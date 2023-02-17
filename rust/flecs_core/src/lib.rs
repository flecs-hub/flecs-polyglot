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
    pub fn free(ptr: *mut c_void);
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
    F32Array
}

static mut WORLD: Option<*mut ecs_world_t> = None;

pub fn init() {
    // Create a flecs world
    unsafe { WORLD = Some(ecs_init()) }
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
        _ => FLECS__Eecs_f32_t
    }
}


#[no_mangle]
pub unsafe fn flecs_component_create(component_name: *const c_char, member_names: *const *const c_char, member_names_size: u32, member_types: *const *const u8, member_types_size: u32) -> ecs_entity_t  {
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
    let member_types = std::slice::from_raw_parts(member_types as *const u8, member_names_size as usize);
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
pub unsafe fn flecs_component_get(name: *const c_char) -> ecs_entity_t {
    let world = *WORLD.as_mut().unwrap_unchecked();
    let component_entity: ecs_entity_t = ecs_lookup(world, name);
    component_entity
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
pub unsafe fn flecs_query_create(id: *mut i32, length: i32) -> *mut ecs_query_t {
    // Slice from raw parts
    let ids = std::slice::from_raw_parts(id as *mut i32, length as usize);

    let world = *WORLD.as_mut().unwrap_unchecked();
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

#[no_mangle]
pub unsafe fn flecs_query_iter_ptrs(iter: *mut ecs_iter_t, component_query_index: u32) -> *mut *mut c_void {
    (*iter).ptrs
}

#[no_mangle]
pub unsafe fn flecs_query_iter_component(component_array_ptr: *mut u8, component_index: u32, count: u32) -> *const u8 {
    let ptrs_slice = std::slice::from_raw_parts(component_array_ptr, count as usize * 8);
    let ptr = &ptrs_slice[(component_index as usize) * 8];
    ptr as *const u8
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
pub unsafe fn flecs_component_set_member_string(component_ptr: *mut c_void, offset: u32, value: *mut c_char) {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut c_char;
    // *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_string(component_ptr: *mut c_void, offset: u32) -> *mut c_char {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut c_char;
    *member_ptr as *mut c_char
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_f32array(component_ptr: *mut c_void, offset: u32, value: *mut f32) {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut f32;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_f32array(component_ptr: *mut c_void, offset: u32) -> *mut f32 {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut f32;
    *member_ptr as *mut f32
}

#[no_mangle]
pub unsafe fn m_free(ptr: *mut c_void) {
    free(ptr as *mut c_void)
}