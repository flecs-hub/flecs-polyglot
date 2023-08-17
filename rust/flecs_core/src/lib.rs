#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]
use core::ffi::{c_char, c_void};
use std::{mem::MaybeUninit, sync::Mutex};
pub mod bindings {
    include!("./bindings.rs");
}
pub use bindings::*;
use lazy_static::lazy_static;


pub struct World {
    pub world: *mut bindings::ecs_world_t
}
unsafe impl Send for World {}
lazy_static! {
    pub static ref WORLD: Mutex<World> = Mutex::new(World{world: unsafe { ecs_init() }});
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ecs_struct_desc_t {
    pub entity: ecs_entity_t,
    pub members: [ecs_member_t; 32usize],
}

extern "C" {
    pub fn free(ptr: *mut c_void);
    #[allow(clashing_extern_declarations)]
    pub fn ecs_struct_init(world: *mut ecs_world_t, desc: *const ecs_struct_desc_t)
        -> ecs_entity_t;
}

pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Array,
    U32Array,
    F32Array,
}

pub fn init() {
    WORLD.lock().unwrap().world;
}

unsafe fn get_member_type(member_type: u8) -> u64 {
    match member_type {
        0 => FLECS__Eecs_u8_t,
        1 => FLECS__Eecs_u16_t,
        2 => FLECS__Eecs_u32_t,
        3 => FLECS__Eecs_u64_t,
        4 => FLECS__Eecs_i8_t,
        5 => FLECS__Eecs_i16_t,
        6 => FLECS__Eecs_i32_t,
        7 => FLECS__Eecs_i64_t,
        8 => FLECS__Eecs_f32_t,
        9 => FLECS__Eecs_f64_t,
        10 => FLECS__Eecs_bool_t,
        11 => FLECS__Eecs_string_t,
        _ => FLECS__Eecs_uptr_t,
    }
}

#[no_mangle]
pub unsafe fn flecs_component_create(
    component_name: *const c_char,
    member_names: *const *const c_char,
    member_names_count: u32,
    member_types: *const u8,
    member_types_size: u32,
) -> ecs_entity_t {
    let world = WORLD.lock().unwrap().world;

    // Create component entity description
    let mut ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.name = component_name;
    let component_entity: ecs_entity_t = ecs_entity_init(world, &ent_desc);

    // Create runtime component description
    let mut struct_desc: ecs_struct_desc_t = MaybeUninit::zeroed().assume_init();
    struct_desc.entity = component_entity;
    let member: ecs_member_t = MaybeUninit::zeroed().assume_init();
    struct_desc.members = [member; 32usize];

    let member_names =
        std::slice::from_raw_parts(member_names as *const *const c_char, member_names_count as usize);
    let member_types =
        std::slice::from_raw_parts(member_types as *const u8, member_names_count as usize);

    // Iterate through member names
    for (index, member_name) in member_names.iter().enumerate() {
        let member_name = *member_name as *const c_char;
        // Create component member
        let mut member: ecs_member_t = MaybeUninit::zeroed().assume_init();
        member.name = member_name;
        member.type_ = get_member_type(member_types[index]);
        struct_desc.members[index] = member;
    }

    ecs_struct_init(world, &struct_desc)
}

#[no_mangle]
pub unsafe fn flecs_tag_create(tag_name: *const c_char) -> ecs_entity_t {
    let world = WORLD.lock().unwrap().world;

    // Create component entity description
    let mut ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.name = tag_name;
    let component_entity: ecs_entity_t = ecs_entity_init(world, &ent_desc);

    ecs_entity_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_component_get(name: *const c_char) -> ecs_entity_t {
    let world = WORLD.lock().unwrap().world;
    let component_entity: ecs_entity_t = ecs_lookup(world, name);
    component_entity
}

#[no_mangle]
pub unsafe fn flecs_entity_create() -> ecs_entity_t {
    let world = WORLD.lock().unwrap().world;
    let ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    ecs_entity_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_entity_create_named(name: *const c_char) -> ecs_entity_t {
    let world = WORLD.lock().unwrap().world;
    let mut ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.name = name;
    ecs_entity_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_entity_create_bulk(count: i32) -> *const ecs_entity_t {
    let world = WORLD.lock().unwrap().world;
    let mut ent_desc: ecs_bulk_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.count = count;
    ecs_bulk_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_entity_create_bulk_components(
    entity_count: i32,
    component_count: u32,
    components: *const u32,
) -> *const ecs_entity_t {
    let world = WORLD.lock().unwrap().world;
    let components = std::slice::from_raw_parts(components as *const u32, component_count as usize);
    let mut ent_desc: ecs_bulk_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.count = entity_count;
    for (index, component) in components.iter().enumerate() {
        ent_desc.ids[index] = *component as u64;
    }

    ecs_bulk_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_entity_get_component(entity: u32, component: u32) -> *mut c_void {
    let world = WORLD.lock().unwrap().world;
    let entity: ecs_entity_t = entity.try_into().unwrap_unchecked();
    let component: ecs_entity_t = component.try_into().unwrap_unchecked();
    ecs_get_mut_id(world, entity, component)
}

#[no_mangle]
pub unsafe fn flecs_entity_add_component(entity: u32, component: u32) -> *mut c_void {
    let world = WORLD.lock().unwrap().world;
    let entity: ecs_entity_t = entity.try_into().unwrap_unchecked();
    let component: ecs_entity_t = component.try_into().unwrap_unchecked();
    // println!("World, entity, component: {:?}, {:?}, {:?}", world, entity, component);
    let component_ptr = ecs_get_mut_id(world, entity, component);
    component_ptr
}

#[no_mangle]
pub unsafe fn flecs_entity_remove_component(entity: u32, component: u32) {
    let world = WORLD.lock().unwrap().world;
    let entity: ecs_entity_t = entity.try_into().unwrap_unchecked();
    let component: ecs_entity_t = component.try_into().unwrap_unchecked();
    ecs_remove_id(world, entity, component)
}

#[no_mangle]
pub unsafe fn flecs_entity_add_tag(entity: u32, tag: u32) {
    let world = WORLD.lock().unwrap().world;
    let entity: ecs_entity_t = entity.try_into().unwrap_unchecked();
    let tag: ecs_entity_t = tag.try_into().unwrap_unchecked();
    ecs_add_id(world, entity, tag);
}

#[no_mangle]
pub unsafe fn flecs_entity_child_of(entity: u32, parent: u32) {
    let world = WORLD.lock().unwrap().world;
    let entity: ecs_entity_t = entity.try_into().unwrap_unchecked();
    let parent: ecs_entity_t = parent.try_into().unwrap_unchecked();
    let pair = ecs_make_pair(EcsChildOf, parent);
    ecs_add_id(world, entity, pair);
}

#[no_mangle]
pub unsafe fn flecs_entity_children(parent: u32) -> *mut ecs_iter_t {
    let world = WORLD.lock().unwrap().world;
    let parent: ecs_entity_t = parent.try_into().unwrap_unchecked();

    let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
    term.id = ecs_make_pair(EcsChildOf, parent);

    let mut iter = ecs_term_iter(world, &mut term);
    
    // Convert iter to raw pointer
    let iter_ptr: *mut ecs_iter_t = &mut iter;
    iter_ptr
}

#[no_mangle]
pub unsafe fn flecs_term_next(iter: *mut ecs_iter_t) -> bool {
    ecs_term_next(iter)
}

#[no_mangle]
pub unsafe fn flecs_child_entities(iter: *mut ecs_iter_t) -> *mut u64 {
    (*iter).entities
}

#[no_mangle]
pub unsafe fn flecs_query_create(ids: *mut i32, components_count: i32) -> *mut ecs_query_t {
    // Slice from raw parts
    let ids = std::slice::from_raw_parts(ids as *mut i32, components_count as usize);

    let world = WORLD.lock().unwrap().world;
    let mut desc: ecs_query_desc_t = MaybeUninit::zeroed().assume_init();

    // Iterate over ids
    for (index, id) in ids.iter().enumerate() {
        let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
        term.id = (*id).try_into().unwrap();
        desc.filter.terms[index] = term;
    }

    let query: *mut ecs_query_t = ecs_query_init(world, &desc);
    query
}

#[no_mangle]
pub unsafe fn flecs_query_next(iter: *mut ecs_iter_t) -> bool {
    ecs_query_next(iter)
}

#[no_mangle]
pub unsafe fn flecs_query_iter(query: *mut ecs_query_t) -> *mut ecs_iter_t {
    let world = WORLD.lock().unwrap().world;
    let it = ecs_query_iter(world, query);
    let it_ptr = Box::into_raw(Box::new(it));
    it_ptr
}

#[no_mangle]
pub unsafe fn flecs_iter_count(iter: *mut ecs_iter_t) -> i32 {
    (*iter).count
}

// This is for the guest to get the pointers to the components based on the index
// of the component when the query was created
// That's why there is an array of arrays. The first array is the first component type as an array of pointers

#[no_mangle]
pub unsafe fn flecs_iter_ptrs(
    iter: *mut ecs_iter_t,
    component_query_index: u32,
) -> *mut *mut c_void {
    (*iter).ptrs
}

#[no_mangle]
pub unsafe fn flecs_query_iter_component(
    component_array_ptr: *mut u8,
    component_index: u32,
    count: u32,
    component_id: u32,
) -> *const u8 {
    let world = WORLD.lock().unwrap().world;

    // TODO: Have this size value already on the host side in stead of
    // Looking up ecs_get_type_info every time
    let component: ecs_entity_t = component_id.try_into().unwrap_unchecked();
    let type_info = ecs_get_type_info(world, component);
    let component_size = (*type_info).size as usize;

    let ptrs_slice =
        std::slice::from_raw_parts(component_array_ptr, count as usize * component_size);
    let ptr = &ptrs_slice[(component_index as usize) * component_size];
    ptr as *const u8
}

#[no_mangle]
pub unsafe fn flecs_query_field(
    iter: *mut ecs_iter_t,
    term_index: i32,
    count: u32,
    index: u32,
) -> *const c_void {
    let size = ecs_field_size(iter, term_index);
    let field = ecs_field_w_size(iter, size, term_index);

    // Create pointer for an offset in field which is an array of component data
    let ptrs_slice = std::slice::from_raw_parts(field, count as usize * size);
    let ptr = &ptrs_slice[index as usize * size];
    ptr as *const c_void
}

#[no_mangle]
pub unsafe fn flecs_query_field_size(
    iter: *mut ecs_iter_t,
    term_index: i32,
) -> usize {
    ecs_field_size(iter, term_index)
}

#[no_mangle]
pub unsafe fn flecs_query_field_list(
    iter: *mut ecs_iter_t,
    term_index: i32,
    count: u32
) -> &'static mut [*const c_void] {
    let size = ecs_field_size(iter, term_index);
    let field = ecs_field_w_size(iter, size, term_index);
    // Create pointer for an offset in field which is an array of component data
    let ptrs_slice = std::slice::from_raw_parts(field, count as usize * size);
    // Create a new vec and add new pointers to the component
    // to the vector
    let mut component_ptrs: Vec<*const c_void> = Vec::new();
    for i in 0..count {
        let ptr = &ptrs_slice[i as usize * size];
        component_ptrs.push(ptr as *const c_void);
    }
    // Convert to slice and leak the box so it can be used on the guest side
    Box::leak(component_ptrs.into_boxed_slice()) as &'static mut [*const c_void]
}

#[no_mangle]
pub unsafe fn flecs_query_entity(iter: *mut ecs_iter_t, count: u32, index: u32) -> u64 {
    let world = WORLD.lock().unwrap().world;
    let entities = (*iter).entities;
    let entities_slice = std::slice::from_raw_parts(entities, count as usize);
    let entity = entities_slice[index as usize];
    entity
}

#[no_mangle]
pub unsafe fn flecs_query_entity_list(iter: *mut ecs_iter_t) -> *mut u64 {
    let world = WORLD.lock().unwrap().world;
    let entities = (*iter).entities;
    entities
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_u8(component_ptr: *mut c_void, offset: u32, value: u8) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut u8;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_u8(component_ptr: *mut c_void, offset: u32) -> u8 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut u8;
    let member_value: u8 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_u16(component_ptr: *mut c_void, offset: u32, value: u16) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut u16;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_u16(component_ptr: *mut c_void, offset: u32) -> u16 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut u16;
    let member_value: u16 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_u32(component_ptr: *mut c_void, offset: u32, value: u32) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut u32;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_u32(component_ptr: *mut c_void, offset: u32) -> u32 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut u32;
    let member_value: u32 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_u64(component_ptr: *mut c_void, offset: u32, value: u64) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut u64;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_u64(component_ptr: *mut c_void, offset: u32) -> u64 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut u64;
    let member_value: u64 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_i8(component_ptr: *mut c_void, offset: u32, value: i8) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut i8;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_i8(component_ptr: *mut c_void, offset: u32) -> i8 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut i8;
    let member_value: i8 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_i16(component_ptr: *mut c_void, offset: u32, value: i16) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut i16;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_i16(component_ptr: *mut c_void, offset: u32) -> i16 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut i16;
    let member_value: i16 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_i32(component_ptr: *mut c_void, offset: u32, value: i32) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut i32;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_i32(component_ptr: *mut c_void, offset: u32) -> i32 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut i32;
    let member_value: i32 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_i64(component_ptr: *mut c_void, offset: u32, value: i64) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut i64;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_i64(component_ptr: *mut c_void, offset: u32) -> i64 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut i64;
    let member_value: i64 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_f32(component_ptr: *mut c_void, offset: u32, value: f32) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut f32;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_f32(component_ptr: *mut c_void, offset: u32) -> f32 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut f32;
    let member_value: f32 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_f64(component_ptr: *mut c_void, offset: u32, value: f64) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut f64;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_f64(component_ptr: *mut c_void, offset: u32) -> f64 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut f64;
    let member_value: f64 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_bool(
    component_ptr: *mut c_void,
    offset: u32,
    value: bool,
) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut bool;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_bool(component_ptr: *mut c_void, offset: u32) -> bool {
    let member_ptr = component_ptr.offset(offset as isize) as *mut bool;
    let member_value: bool = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_string(
    component_ptr: *mut c_void,
    offset: u32,
    value: *mut c_char,
) {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut c_char;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_string(
    component_ptr: *mut c_void,
    offset: u32,
) -> *mut c_char {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut c_char;
    *member_ptr
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_u32array(
    component_ptr: *mut c_void,
    offset: u32,
    value: *mut u32,
) {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut u32;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_u32array(
    component_ptr: *mut c_void,
    offset: u32,
) -> *mut u32 {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut u32;
    *member_ptr as *mut u32
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_f32array(
    component_ptr: *mut c_void,
    offset: u32,
    value: *mut f32,
) {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut f32;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_f32array(
    component_ptr: *mut c_void,
    offset: u32,
) -> *mut f32 {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut f32;
    *member_ptr as *mut f32
}

#[no_mangle]
pub unsafe fn flecs_progress(delta_time: f32) -> bool {
    let world = WORLD.lock().unwrap().world;
    ecs_progress(world, delta_time)
}

#[no_mangle]
pub unsafe fn flecs_make_pair(relation: u32, object: u32) -> u64 {
    let relation: ecs_entity_t = relation.try_into().unwrap_unchecked();
    let object: ecs_entity_t = object.try_into().unwrap_unchecked();
    ecs_make_pair(relation, object)
}

#[no_mangle]
pub unsafe fn flecs_filter_children_init(id: ecs_entity_t) -> *mut ecs_filter_t {
    let world = WORLD.lock().unwrap().world;
    let mut desc: ecs_filter_desc_t = MaybeUninit::zeroed().assume_init();
    desc.terms[0].id = ecs_make_pair(EcsChildOf, id);
    desc.terms[1].id = EcsPrefab;
    desc.terms[1].oper = ecs_oper_kind_t_EcsOptional;
    ecs_filter_init(world, &desc)
}

#[no_mangle]
pub unsafe fn flecs_filter_iter(filter: *mut ecs_filter_t) -> *mut ecs_iter_t {
    let world = WORLD.lock().unwrap().world;
    let it = ecs_filter_iter(world, filter);
    let it_ptr = Box::into_raw(Box::new(it));
    it_ptr
}

#[no_mangle]
pub unsafe fn flecs_filter_next(iter: *mut ecs_iter_t) -> bool {
    ecs_filter_next(iter)
}


#[no_mangle]
pub unsafe fn flecs_iter_entities(iter: *mut ecs_iter_t) -> &'static [u64] {
    let entities = (*iter).entities;
    let entities_slice = std::slice::from_raw_parts(entities, (*iter).count as usize);
    entities_slice
}

#[no_mangle]
pub unsafe fn flecs_delete_entity(entity: u32) {
    let world = WORLD.lock().unwrap().world;
    let entity: ecs_entity_t = entity.try_into().unwrap_unchecked();
    ecs_delete(world, entity);
}

#[no_mangle]
pub unsafe fn flecs_entity_has_component(entity: u32, component: u32) -> bool {
    let world = WORLD.lock().unwrap().world;
    let entity: ecs_entity_t = entity.try_into().unwrap_unchecked();
    let component: ecs_entity_t = component.try_into().unwrap_unchecked();
    ecs_has_id(world, entity, component)
}

#[no_mangle]
pub unsafe fn flecs_is_valid(entity: u32) -> bool {
    let world = WORLD.lock().unwrap().world;
    let entity: ecs_entity_t = entity.try_into().unwrap_unchecked();
    ecs_is_valid(world, entity)
}

#[no_mangle]
pub unsafe fn m_free(ptr: *mut c_void) {
    free(ptr as *mut c_void)
}
