#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(improper_ctypes)]
// #![feature(thread_id_value)]

pub mod bindings {
    include!("./bindings.rs");
}
pub use bindings::*;
use toxoid_api::make_c_string;

use std::mem::MaybeUninit;
use std::collections::HashMap;
#[cfg(feature = "multithread")]
use std::thread::JoinHandle;
use core::ffi::{c_char, c_void};
use once_cell::sync::Lazy;

pub static mut WORLD: Lazy<*mut bindings::ecs_world_t> = Lazy::new(|| unsafe { ecs_init() });

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ecs_struct_desc_t {
    pub entity: ecs_entity_t,
    pub members: [ecs_member_t; 32usize],
}

extern "C" {
    pub fn free(ptr: *mut c_void);
    #[cfg(feature = "multithread")]
    pub fn pthread_self() -> i32;
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

// Generic function to iterate over an ecs_vector_t
pub unsafe fn ecs_vector_each<T, F>(vector: *const ecs_vec_t, mut f: F)
where
    T: Sized,
    F: FnMut(&T),
{
    let count = ecs_vec_count(vector) as isize;
    let first = ecs_vec_first(vector);
    let mut current = first as *const T;

    for _ in 0..count {
        f(&*current);
        current = current.offset(1);
    }
}

unsafe fn get_member_type(member_type: u8) -> ecs_entity_t {
    match member_type {
        0 => FLECS_IDecs_u8_tID_,
        1 => FLECS_IDecs_u16_tID_,
        2 => FLECS_IDecs_u32_tID_,
        3 => FLECS_IDecs_u64_tID_,
        4 => FLECS_IDecs_i8_tID_,
        5 => FLECS_IDecs_i16_tID_,
        6 => FLECS_IDecs_i32_tID_,
        7 => FLECS_IDecs_i64_tID_,
        8 => FLECS_IDecs_f32_tID_,
        9 => FLECS_IDecs_f64_tID_,
        10 => FLECS_IDecs_bool_tID_,
        11 => FLECS_IDecs_string_tID_,
        _ => FLECS_IDecs_uptr_tID_,
    }
}

pub fn init() {
    #[cfg(feature = "multithread")]
    unsafe {
        // Set ECS threads
        // ecs_set_threads(*WORLD, 12);
        #[cfg(feature = "multithread")] {
            ecs_set_task_threads(*WORLD, 12);
            ecs_os_api.task_new_ = Some(flecs_os_api_task_new);
            ecs_os_api.task_join_ = Some(flecs_os_api_task_join);
        }
        // println!("Pthread ID from flecs init: {}", pthread_self());
        // std::thread::spawn(move || {
        //     println!("Pthread ID from Rust thread: {}", pthread_self());
        // });
    }
}

#[cfg(feature = "multithread")]
#[no_mangle]
pub unsafe extern "C" fn flecs_os_api_task_new(optional_callback: Option<unsafe extern "C" fn(*mut c_void) -> *mut c_void>, _ctx: *mut c_void) -> usize {
    println!("From flecs_os_api_task_new");
    let callback = optional_callback.unwrap();
    let ctx_as_usize: usize = _ctx as usize;
    let handle = std::thread::spawn(move || {
        // println!("This system runs on this thread from flecs_os_api_task_new: {}", std::thread::ThreadId::as_u64(&std::thread::current().id()));
        // println!("Pthread ID from flecs_os_api_task_new: {}", pthread_self());
        // println!("From flecs_os_api_task_new thread");
        // Back inside the new thread, cast it back to a raw pointer.
        let _ctx_back: *mut c_void = ctx_as_usize as *mut c_void;
        callback(_ctx_back);
    });
    Box::into_raw(Box::new(handle)) as *mut _ as usize
}

#[cfg(feature = "multithread")]
#[no_mangle]
pub unsafe extern "C" fn flecs_os_api_task_join(handle: usize) -> *mut c_void {
    println!("From flecs_os_api_task_join");
     // Convert back to the original Rust JoinHandle type
     let handle: Box<JoinHandle<()>> = Box::from_raw(handle as *mut JoinHandle<()>);

     // Wait for the thread to complete
     handle.join().unwrap();

    // Return a null pointer
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe fn flecs_component_create(
    component_name: *const c_char,
    member_names: *const *const c_char,
    member_names_count: u32,
    member_types: *const u8,
    member_types_size: u32,
) -> ecs_entity_t {
    let world = *WORLD;

    // Create component entity description
    let mut ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.name = component_name;
    let component_entity: ecs_entity_t = ecs_entity_init(world, &ent_desc);
    // println!("Component name: {:?} \n", std::ffi::CStr::from_ptr(component_name).to_str().unwrap());

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
        // print!("Member name: {:?} \n", std::ffi::CStr::from_ptr(member_name).to_str().unwrap());
        member.type_ = get_member_type(member_types[index]);
        struct_desc.members[index] = member;
    }

    ecs_struct_init(world, &struct_desc)
}

#[no_mangle]
pub unsafe fn flecs_tag_create(tag_name: *const c_char) -> ecs_entity_t {
    let world = *WORLD;

    // Create component entity description
    let mut ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.name = tag_name;
    let component_entity: ecs_entity_t = ecs_entity_init(world, &ent_desc);

    ecs_entity_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_component_get(name: *const c_char) -> ecs_entity_t {
    let world = *WORLD;
    let component_entity: ecs_entity_t = ecs_lookup(world, name);
    component_entity
}

#[no_mangle]
pub unsafe fn flecs_entity_create() -> ecs_entity_t {
    let world = *WORLD;
    let ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    ecs_entity_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_prefab_create() -> ecs_entity_t {
    let world = *WORLD;
    ecs_new_w_id(world, EcsPrefab)
}

#[no_mangle]
pub unsafe fn flecs_prefab_instance(prefab: ecs_entity_t) -> ecs_entity_t {
    let world = *WORLD;
    let ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    let pair = ecs_make_pair(EcsIsA, prefab);
    ecs_new_w_id(world, pair)
}

#[no_mangle]
pub unsafe fn flecs_entity_create_named(name: *const c_char) -> ecs_entity_t {
    let world = *WORLD;
    let mut ent_desc: ecs_entity_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.name = name;
    ecs_entity_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_entity_create_bulk(count: i32) -> *const ecs_entity_t {
    let world = *WORLD;
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
    let world = *WORLD;
    let components = std::slice::from_raw_parts(components as *const ecs_entity_t, component_count as usize);
    let mut ent_desc: ecs_bulk_desc_t = MaybeUninit::zeroed().assume_init();
    ent_desc.count = entity_count;
    for (index, component) in components.iter().enumerate() {
        ent_desc.ids[index] = *component as ecs_entity_t;
    }

    ecs_bulk_init(world, &ent_desc)
}

#[no_mangle]
pub unsafe fn flecs_singleton_add(component: ecs_entity_t) {
    let world = *WORLD;
    ecs_add_id(world, EcsWorld, component);
}

#[no_mangle]
pub unsafe fn flecs_singleton_get(component: ecs_entity_t) -> *mut c_void {
    let world = *WORLD;
    ecs_get_mut_id(world, EcsWorld, component)
}

#[no_mangle]
pub unsafe fn flecs_singleton_remove(component: ecs_entity_t) {
    let world = *WORLD;
    ecs_remove_id(world, EcsWorld, component);
}

#[no_mangle]
pub unsafe fn flecs_entity_get_component(entity: ecs_entity_t, component: ecs_entity_t) -> *mut c_void {
    let world = *WORLD;
    ecs_get_mut_id(world, entity, component)
}

#[no_mangle]
pub unsafe fn flecs_entity_add_component(entity: ecs_entity_t, component: ecs_entity_t) {
    let world = *WORLD;
    ecs_add_id(world, entity, component);
}

#[no_mangle]
pub unsafe fn flecs_entity_remove_component(entity: ecs_entity_t, component: ecs_entity_t) {
    let world = *WORLD;
    ecs_remove_id(world, entity, component)
}

#[no_mangle]
pub unsafe fn flecs_entity_add_tag(entity: ecs_entity_t, tag: ecs_entity_t) {
    let world = *WORLD;
    ecs_add_id(world, entity, tag);
}

#[no_mangle]
pub unsafe fn flecs_entity_child_of(entity: ecs_entity_t, parent: ecs_entity_t) {
    let world = *WORLD;
    let pair = ecs_make_pair(EcsChildOf, parent);
    ecs_add_id(world, entity, pair);
}

#[no_mangle]
pub unsafe fn flecs_entity_children(parent: ecs_entity_t) -> *mut ecs_iter_t {
    let world = *WORLD;
    let parent: ecs_entity_t = parent;

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
pub unsafe fn flecs_child_entities(iter: *mut ecs_iter_t) -> *mut ecs_entity_t {
    (*iter).entities
}

#[no_mangle]
pub unsafe fn flecs_query_create() -> *mut ecs_query_desc_t {
    let desc: ecs_query_desc_t = MaybeUninit::zeroed().assume_init();
    Box::into_raw(Box::new(desc))
}

#[no_mangle]
pub unsafe fn flecs_query_with(query_desc: *mut ecs_query_desc_t, filter_index: u8, ids: *mut ecs_entity_t, components_count: i32) -> u8 {
    // Slice from raw parts
    let ids = std::slice::from_raw_parts(ids as *mut u64, components_count as usize);

    let world = *WORLD;

    // Iterate over ids
    /*
        The new filter index in the flecs_query_with function is used to keep track of the position where the next term should be added to the query's filter terms array. Each time a new term is added, the filter index is updated to the current index of the loop, which represents the last position used in the filter terms array. This is necessary because the function may be called multiple times to add multiple terms, and each call needs to know where to insert the next term without overwriting the existing ones.

        The final value of new_filter_index is returned as a u8 so that the caller knows the next available index for adding additional terms if needed. This is important for building complex queries with multiple terms.
    */
    let mut new_filter_index = filter_index;
    for (index, id) in ids.iter().enumerate() {
        let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
        term.id = *id;
        term.oper = ecs_oper_kind_t_EcsAnd;
        (*query_desc).filter.terms[filter_index as usize + index] = term;
        new_filter_index = index as u8;
    }
    new_filter_index as u8
}


#[no_mangle]
pub unsafe fn flecs_query_without(query_desc: *mut ecs_query_desc_t, filter_index: u8, ids: *mut ecs_entity_t, components_count: i32) -> u8 {
    // Slice from raw parts
    let ids = std::slice::from_raw_parts(ids as *mut u64, components_count as usize);

    let world = *WORLD;

    // Iterate over ids
    let mut new_filter_index = filter_index;
    for (index, id) in ids.iter().enumerate() {
        let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
        term.id = *id;
        term.oper = ecs_oper_kind_t_EcsNot;
        (*query_desc).filter.terms[filter_index as usize + index] = term;
        new_filter_index = index as u8;
    }
    new_filter_index 
}

#[no_mangle]
pub unsafe fn flecs_query_with_or(query_desc: *mut ecs_query_desc_t, filter_index: u8, ids: *mut ecs_entity_t, components_count: i32) -> u8 {
    // Slice from raw parts
    let ids = std::slice::from_raw_parts(ids as *mut u64, components_count as usize);

    let world = *WORLD;

    let mut new_filter_index = filter_index;
    // Iterate over ids
    for (index, id) in ids.iter().enumerate() {
        let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
        term.id = *id;
        term.oper = ecs_oper_kind_t_EcsOr;
        (*query_desc).filter.terms[filter_index as usize + index] = term;
        new_filter_index = index as u8;
    }
    new_filter_index
}

#[no_mangle]
pub unsafe fn flecs_query_build(desc: *mut ecs_query_desc_t) -> *mut ecs_query_t {
    let world = *WORLD;
    let query: *mut ecs_query_t = ecs_query_init(world, desc);
    query
}

#[no_mangle]
pub unsafe fn flecs_query_next(iter: *mut ecs_iter_t) -> bool {
    ecs_query_next(iter)
}

#[no_mangle]
pub unsafe fn flecs_query_iter(query: *mut ecs_query_t) -> *mut ecs_iter_t {
    let world = *WORLD;
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
    component_id: ecs_entity_t,
) -> *const u8 {
    let world = *WORLD;

    // TODO: Have this size value already on the host side in stead of
    // Looking up ecs_get_type_info every time
    let component: ecs_entity_t = component_id;
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
) -> *mut [*const c_void] {
    let size = ecs_field_size(iter, term_index);
    let field = ecs_field_w_size(iter, size, term_index);

    let ptrs_slice = std::slice::from_raw_parts(field, count as usize * size);
    let mut component_ptrs: Vec<*const c_void> = Vec::new();

    for i in 0..count {
        let ptr = &ptrs_slice[i as usize * size];
        component_ptrs.push(ptr as *const c_void);
    }

    let boxed_slice = component_ptrs.into_boxed_slice();
    let raw_ptr = Box::into_raw(boxed_slice);

    raw_ptr
}

#[no_mangle]
pub unsafe fn flecs_query_entity(iter: *mut ecs_iter_t, count: u32, index: u32) -> ecs_entity_t {
    let world = *WORLD;
    let entities = (*iter).entities;
    let entities_slice = std::slice::from_raw_parts(entities, count as usize);
    let entity = entities_slice[index as usize];
    entity
}

#[no_mangle]
pub unsafe fn flecs_query_entity_list(iter: *mut ecs_iter_t) -> *mut ecs_entity_t {
    let world = *WORLD;
    let entities = (*iter).entities;
    entities
}

// TODO: Take another look at whether this is nessecary, because we don't want to copy data
/*
#[no_mangle]
pub unsafe fn flecs_query_entity_list(iter: *mut ecs_iter_t) -> *mut ecs_entity_t {
    let world = *WORLD;
    let entities_ptr = (*iter).entities;

    // Copy data into a new Vec
    let entities = std::slice::from_raw_parts(entities_ptr, (*iter).count as usize).to_vec();

    // Convert Vec into a boxed slice
    let boxed_entities = entities.into_boxed_slice();

    // Prevent the boxed slice from being deallocated automatically
    Box::into_raw(boxed_entities) as *mut ecs_entity_t
}
*/

#[no_mangle]
pub unsafe fn flecs_filter_create() -> *mut ecs_filter_desc_t {
    let desc: ecs_filter_desc_t = MaybeUninit::zeroed().assume_init();
    Box::into_raw(Box::new(desc))
}

#[no_mangle]
pub unsafe fn flecs_filter_with(filter_desc: *mut ecs_filter_desc_t, filter_index: u8, ids: *mut ecs_entity_t, components_count: i32) -> u8 {
    // Slice from raw parts
    let ids = std::slice::from_raw_parts(ids as *mut u64, components_count as usize);

    let world = *WORLD;

    // Iterate over ids
    /*
        The new filter index in the flecs_query_with function is used to keep track of the position where the next term should be added to the query's filter terms array. Each time a new term is added, the filter index is updated to the current index of the loop, which represents the last position used in the filter terms array. This is necessary because the function may be called multiple times to add multiple terms, and each call needs to know where to insert the next term without overwriting the existing ones.

        The final value of new_filter_index is returned as a u8 so that the caller knows the next available index for adding additional terms if needed. This is important for building complex queries with multiple terms.
    */
    let mut new_filter_index = 0;
    for (index, id) in ids.iter().enumerate() {
        let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
        term.id = *id;
        term.oper = ecs_oper_kind_t_EcsAnd;
        (*filter_desc).terms[filter_index as usize + index] = term;
        new_filter_index = index;
    }
    new_filter_index as u8
}

#[no_mangle]
pub unsafe fn flecs_filter_without(filter_desc: *mut ecs_filter_desc_t, filter_index: u8, ids: *mut ecs_entity_t, components_count: i32) -> u8 {
    // Slice from raw parts
    let ids = std::slice::from_raw_parts(ids as *mut u64, components_count as usize);

    let world = *WORLD;

    // Iterate over ids
    let mut new_filter_index = 0;
    for (index, id) in ids.iter().enumerate() {
        let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
        term.id = *id;
        term.oper = ecs_oper_kind_t_EcsNot;
        (*filter_desc).terms[filter_index as usize + index] = term;
        new_filter_index = index;
    }
    new_filter_index as u8
}

#[no_mangle]
pub unsafe fn flecs_filter_with_or(filter_desc: *mut ecs_filter_desc_t, filter_index: u8, ids: *mut ecs_entity_t, components_count: i32) -> u8 {
    // Slice from raw parts
    let ids = std::slice::from_raw_parts(ids as *mut u64, components_count as usize);

    let world = *WORLD;

    // Iterate over ids
    let mut new_filter_index = 0;
    for (index, id) in ids.iter().enumerate() {
        let mut term: ecs_term_t = MaybeUninit::zeroed().assume_init();
        term.id = *id;
        term.oper = ecs_oper_kind_t_EcsOr;
        (*filter_desc).terms[filter_index as usize + index] = term;
        new_filter_index = index;
    }
    new_filter_index as u8
}

#[no_mangle]
pub unsafe fn flecs_filter_build(desc: *mut ecs_filter_desc_t) -> *mut ecs_filter_t {
    let world = *WORLD;
    let filter: *mut ecs_filter_t = ecs_filter_init(world, desc);
    filter
}

#[no_mangle]
pub unsafe fn flecs_filter_next(iter: *mut ecs_iter_t) -> bool {
    ecs_filter_next(iter)
}

#[no_mangle]
pub unsafe fn flecs_filter_iter_component(
    component_array_ptr: *mut u8,
    component_index: u32,
    count: u32,
    component_id: ecs_entity_t,
) -> *const u8 {
    let world = *WORLD;

    // TODO: Have this size value already on the host side in stead of
    // Looking up ecs_get_type_info every time
    let component: ecs_entity_t = component_id;
    let type_info = ecs_get_type_info(world, component);
    let component_size = (*type_info).size as usize;

    let ptrs_slice =
        std::slice::from_raw_parts(component_array_ptr, count as usize * component_size);
    let ptr = &ptrs_slice[(component_index as usize) * component_size];
    ptr as *const u8
}

#[no_mangle]
pub unsafe fn flecs_filter_field(
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
pub unsafe fn flecs_filter_field_size(
    iter: *mut ecs_iter_t,
    term_index: i32,
) -> usize {
    ecs_field_size(iter, term_index)
}

#[no_mangle]
pub unsafe fn flecs_filter_field_list(
    iter: *mut ecs_iter_t,
    term_index: i32,
    count: u32
) -> *mut [*const c_void] {
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

    let boxed_slice = component_ptrs.into_boxed_slice();
    let raw_ptr = Box::into_raw(boxed_slice);

    raw_ptr
}

#[no_mangle]
pub unsafe fn flecs_filter_entity(iter: *mut ecs_iter_t, count: u32, index: u32) -> ecs_entity_t {
    let world = *WORLD;
    let entities = (*iter).entities;
    let entities_slice = std::slice::from_raw_parts(entities, count as usize);
    let entity = entities_slice[index as usize];
    entity
}

#[no_mangle]
pub unsafe fn flecs_filter_entity_list(iter: *mut ecs_iter_t) -> *mut ecs_entity_t {
    let world = *WORLD;
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
    let world = *WORLD;
    ecs_progress(world, delta_time)
}

#[no_mangle]
pub unsafe fn flecs_make_pair(relation: ecs_entity_t, object: ecs_entity_t) -> ecs_entity_t {
    let relation: ecs_entity_t = relation;
    let object: ecs_entity_t = object;
    ecs_make_pair(relation, object)
}

#[no_mangle]
pub unsafe fn flecs_filter_children_init(id: ecs_entity_t) -> *mut ecs_filter_t {
    let world = *WORLD;
    let mut desc: ecs_filter_desc_t = MaybeUninit::zeroed().assume_init();
    desc.terms[0].id = ecs_make_pair(EcsChildOf, id);
    desc.terms[1].id = EcsPrefab;
    desc.terms[1].oper = ecs_oper_kind_t_EcsOptional;
    ecs_filter_init(world, &desc)
}

#[no_mangle]
pub unsafe fn flecs_filter_iter(filter: *mut ecs_filter_t) -> *mut ecs_iter_t {
    let world = *WORLD;
    let it = ecs_filter_iter(world, filter);
    let it_ptr = Box::into_raw(Box::new(it));
    it_ptr
}

#[no_mangle]
pub unsafe fn flecs_iter_entities(iter: *mut ecs_iter_t) -> &'static [ecs_entity_t] {
    let entities = (*iter).entities;
    if (*iter).count > 0 {
        let entities_slice = std::slice::from_raw_parts(entities, (*iter).count as usize);
        entities_slice
    } else {
        &[]
    }
}

#[no_mangle]
pub unsafe fn flecs_delete_entity(entity: ecs_entity_t) {
    let world = *WORLD;
    let entity: ecs_entity_t = entity;
    ecs_delete(world, entity);
}

#[no_mangle]
pub unsafe fn flecs_entity_has_component(entity: ecs_entity_t, component: ecs_entity_t) -> bool {
    let world = *WORLD;
    ecs_has_id(world, entity, component)
}

#[no_mangle]
pub unsafe fn flecs_is_valid(entity: ecs_entity_t) -> bool {
    let world = *WORLD;
    let entity: ecs_entity_t = entity;
    ecs_is_valid(world, entity)
}

#[no_mangle]
pub unsafe fn m_free(ptr: *mut c_void) {
    free(ptr as *mut c_void)
}

#[no_mangle]
pub unsafe fn flecs_component_set_member_ptr(
    component_ptr: *mut c_void,
    offset: u32,
    value: *mut c_void,
) {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut c_void;
    *member_ptr = value;
}

#[no_mangle]
pub unsafe fn flecs_component_get_member_ptr(
    component_ptr: *mut c_void,
    offset: u32,
) -> *mut c_void {
    let member_ptr = (component_ptr as *mut u8).add(offset as usize) as *mut *mut c_void;
    *member_ptr as *mut c_void
}

#[no_mangle]
// Trampoline closure from Rust using C callback and binding_ctx field to call a Rust closure
pub unsafe extern "C" fn query_trampoline(iter: *mut ecs_iter_t) {
    // println!("This system runs on this thread from trampoline: {}", std::thread::ThreadId::as_u64(&std::thread::current().id()));
    // println!("Pthread ID from trampoline: {}", pthread_self());
    let world = *WORLD;
    let callback = (*iter).binding_ctx as *mut c_void;
    if callback.is_null() {
        return;
    }
    let iter = toxoid_api::Iter::from(iter as *mut c_void);
    let callback_fn: fn(&toxoid_api::Iter) = std::mem::transmute(callback);
    callback_fn(&iter); // Call the callback through the reference
}

#[no_mangle]
pub unsafe fn flecs_system_create(
    callback: fn(*mut c_void)
) -> *mut ecs_system_desc_t {
    let mut system_desc: ecs_system_desc_t = MaybeUninit::zeroed().assume_init();
    system_desc.binding_ctx = callback as *mut c_void;
    system_desc.callback = Some(query_trampoline);
    #[cfg(feature = "multithread")] {
        system_desc.multi_threaded = true;
    }
    Box::into_raw(Box::new(system_desc))
}

#[no_mangle]
pub unsafe fn flecs_system_build(
    system_desc: *mut ecs_system_desc_t,
) -> ecs_entity_t {
    let world = *WORLD;
    let mut entity_desc: ecs_entity_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
    // We have to add this pair so that the system is part of standard progress stage
    entity_desc.add[0] = unsafe { ecs_make_pair(EcsDependsOn, EcsOnUpdate) };
    (*system_desc).entity = ecs_entity_init(world, &entity_desc);
    ecs_system_init(world, system_desc)
}

#[no_mangle]
pub unsafe fn flecs_query_from_system_desc(
    system_desc: *mut ecs_system_desc_t
) -> *mut ecs_query_desc_t {
    &mut (*system_desc).query as *mut ecs_query_desc_t
}

#[no_mangle]
pub unsafe fn flecs_component_lookup(name: *mut c_char) -> ecs_entity_t {
    let world = *WORLD;
    let component_id: ecs_entity_t = ecs_lookup(world, name);
    component_id
}

#[no_mangle]
pub unsafe fn flecs_entity_to_json(entity: ecs_entity_t) -> *mut c_char {
    let world = *WORLD;
    let mut json_desc: ecs_entity_to_json_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
    json_desc.serialize_base = true;
    json_desc.serialize_ids = true;
    json_desc.serialize_values = true;
    json_desc.serialize_refs = EcsChildOf;
    json_desc.serialize_path = true;
    json_desc.serialize_label = true;
    // json_desc.serialize_id_labels = true;
    // json_desc.serialize_type_info = true;
    // json_desc.serialize_hidden = true;
    // json_desc.serialize_matches = true;
    // json_desc.serialize_private = true;
    let json: *mut c_char = ecs_entity_to_json(world, entity, &json_desc);
    json
}

#[no_mangle]
pub unsafe fn flecs_json_to_entity(json: *mut c_char) {
    let world = *WORLD;
    // let mut json_desc: ecs_from_json_desc_t = unsafe { MaybeUninit::zeroed().assume_init() };
    // json_desc.strict = false;
    // // // New Cstring from &str
    // // let json = make_c_string("{\"ids\":[[\"Platypus\"]],\"values\":[{\"x\":10, \"y\":20}]}");
    // let entity = flecs_entity_create();
    // let entity = toxoid_api::Entity::from_id(entity as u64);
    // let result = ecs_entity_from_json(world, entity.get_id(), json, std::ptr::null());
    // println!("Result: {}", *result);
    
    // // println!("Entity has Position? {}", entity.has::<toxoid_api::components::Position>());
    // let position = entity.get::<toxoid_api::components::Position>();
    // println!("x: {}, y: {}", position.get_x(), position.get_y());
}

#[no_mangle]
pub unsafe fn flecs_entity_set_name(entity: ecs_entity_t, name: *mut c_char) {
    let world = *WORLD;
    ecs_set_name(world, entity, name);
}