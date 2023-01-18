#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]

use std::ffi::{c_char, c_void};
use std::mem::MaybeUninit;
mod bindings { include!("./bindings.rs"); }
use bindings::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ecs_struct_desc_t {
    pub entity: ecs_entity_t,
    pub members: [ecs_member_t; 32usize],
}

extern "C" {
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
    Array
}

static mut WORLD: Option<*mut ecs_world_t> = None;

pub fn init() {
    // Create a flecs world
    unsafe { WORLD = Some(ecs_init()) }
}

#[no_mangle]
pub unsafe fn flecs_component_create(component_name: *const c_char, member_names: *const *const c_char, member_names_size: u32, member_types: *const *const c_char, member_types_size: u32) -> ecs_entity_t  {
    let world = *WORLD.as_mut().unwrap_unchecked();

    // Create component entity description
    let mut ent_desc: ecs_entity_desc_t  = MaybeUninit::zeroed().assume_init();
    ent_desc.name = component_name;
    let component_entity: ecs_entity_t = ecs_entity_init(world, &ent_desc);

    // Create runtime component description
    let mut struct_desc: ecs_struct_desc_t = MaybeUninit::zeroed().assume_init();
    struct_desc.entity = component_entity;
    let member: ecs_member_t = MaybeUninit::zeroed().assume_init();
    struct_desc.members = [member; 32usize];

    let member_names = std::slice::from_raw_parts(member_names as *const u32, member_names_size as usize);
    // Iterate through member names
    for (index, member_name) in member_names.iter().enumerate() {
        let member_name = *member_name as *const c_char;
        // Create component member
        let mut member: ecs_member_t = MaybeUninit::zeroed().assume_init();
        member.name = member_name;
        member.type_ = FLECS__Eecs_f32_t;
        struct_desc.members[index] = member;
    }
    
    ecs_struct_init(world, &struct_desc)
}

#[no_mangle]
pub unsafe fn flecs_entity_create(name: *const c_char) -> ecs_entity_t {
    let world = *WORLD.as_mut().unwrap_unchecked();
    let mut ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.name = name;
    ecs_entity_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_entity_add_component(entity: u32, component: u32) -> *mut c_void {
    let world = *WORLD.as_mut().unwrap_unchecked();
    let entity: ecs_entity_t = entity.try_into().unwrap_unchecked();
    let component: ecs_entity_t = component.try_into().unwrap_unchecked();
    let component_ptr = ecs_get_mut_id(world, entity, component);
    component_ptr
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_float(component_ptr: *mut c_void, offset: u32, value: f32) {
    let member_ptr = component_ptr.offset(offset as isize) as *mut f32;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_float(component_ptr: *mut c_void, offset: u32) -> f32 {
    let member_ptr = component_ptr.offset(offset as isize) as *mut f32;
    let member_value: f32 = *member_ptr;
    member_value
}

#[no_mangle]
pub unsafe fn flecs_query_create(id: u32) -> *mut ecs_query_t {
    let world = *WORLD.as_mut().unwrap_unchecked();
    let mut desc: ecs_query_desc_t = MaybeUninit::zeroed().assume_init();
    let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
    term.id = id.try_into().unwrap_unchecked();
    desc.filter.terms[0] = term;
    let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
    // TODO: Remove this hardcoded value
    term.id = 485;
    desc.filter.terms[1] = term;
    let query: *mut ecs_query_t = ecs_query_init(world, &desc);
    query
}

#[no_mangle]
pub unsafe fn flecs_query_next(iter: *mut ecs_iter_t) -> bool {
    ecs_query_next(iter)
}

#[no_mangle]
pub unsafe fn flecs_query_iter(query: *mut ecs_query_t) -> *mut ecs_iter_t {
    let world = *WORLD.as_mut().unwrap_unchecked();
    let it = ecs_query_iter(world, query);
    let it_ptr = Box::into_raw(Box::new(it));
    it_ptr
}

#[no_mangle]
pub unsafe fn flecs_query_iter_count(iter: *mut ecs_iter_t) -> i32 {
    (*iter).count
}

// This is for the guest to get the pointers to the components based on the index 
// of the component when the query was created
// That's why there is an array of arrays. The first array is the first component type as an array of pointers
/* 
#[no_mangle]
pub unsafe fn flecs_query_iter_ptrs(iter: *mut ecs_iter_t, component_query_index: u32) -> *mut c_void {
    *(*iter).ptrs
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Position {
    x: f32,
    y: f32
}

#[no_mangle]
pub unsafe fn flecs_query_iter_component(component_array_ptr: *mut Position, component_index: u32, count: u32) -> &'static Position {
    let ptrs_slice = std::slice::from_raw_parts(component_array_ptr, count as usize);
    &ptrs_slice[component_index as usize]
}
*/

// Temporary workaround to getting component pointers, has more overhead
#[no_mangle]
pub unsafe fn flecs_query_iter_field(iter: *mut ecs_iter_t, component_ptr_index: ecs_entity_t, term_index: ecs_entity_t) -> *mut c_void {
    let world = *WORLD.as_mut().unwrap_unchecked();
    let entity = (*iter).entities.offset((component_ptr_index as usize * std::mem::size_of::<u8>()).try_into().unwrap());
    let component = (*iter).ids.offset((term_index as usize * std::mem::size_of::<ecs_entity_t>()).try_into().unwrap());
    ecs_get_mut_id(world, *entity, *component)
}