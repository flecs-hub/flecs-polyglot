/// <reference types="emscripten" />

import { EntityID } from 'ecs'

export type Pointer = number

export type u8 = number
export type u16 = number
export type u32 = number
export type u64 = number
export type i8 = number
export type i16 = number
export type i32 = number
export type i64 = number
export type f32 = number
export type f64 = number
export type Bool = boolean
export type String = Pointer
export type Array = Pointer

export interface EmscriptenModuleExtended extends EmscriptenModule {
	allocateUTF8: typeof allocateUTF8,
    writeArrayToMemory: typeof writeArrayToMemory,
    UTF8ToString: typeof UTF8ToString,
    _m_free: (ptr: Pointer) => void,
}

export interface CoreAPI {
    _flecs_component_create: (name: Pointer, memberNames: Pointer, memberNamesSize: u32, memberValues: Pointer, memberValuesSize: u32) => Pointer,
    _flecs_entity_create: () => Pointer,
    _flecs_entity_add_component: (entity: EntityID, component: EntityID) => Pointer,
    _flecs_query_create: (components: Pointer, componentsSize: i32) => Pointer,
    _flecs_query_next: (iter: Pointer) => boolean,
    _flecs_query_iter: (query: Pointer) => Pointer,
    _flecs_query_iter_count: (iter: Pointer) => i32,
    _flecs_query_iter_ptrs: (iter: Pointer, component_query_index: u32) => Pointer,
    _flecs_query_iter_component: (component_array_ptr: Pointer, component_index: u32, count: u32, component_id: EntityID) => Pointer,
    _flecs_component_set_member_u8: (component_ptr: Pointer, offset: u32, value: u8) => void,
    _flecs_component_get_member_u8: (component_ptr: Pointer, offset: u32) => u8,
    _flecs_component_set_member_u16: (component_ptr: Pointer, offset: u32, value: u16) => void,
    _flecs_component_get_member_u16: (component_ptr: Pointer, offset: u32) => u8,
    _flecs_component_set_member_u32: (component_ptr: Pointer, offset: u32, value: u32) => void,
    _flecs_component_get_member_u32: (component_ptr: Pointer, offset: u32) => u8,
    _flecs_component_set_member_u64: (component_ptr: Pointer, offset: u32, value: u64) => void,
    _flecs_component_get_member_u64: (component_ptr: Pointer, offset: u32) => u8,
    _flecs_component_set_member_i8: (component_ptr: Pointer, offset: u32, value: i8) => void,
    _flecs_component_get_member_i8: (component_ptr: Pointer, offset: u32) => i8,
    _flecs_component_set_member_i16: (component_ptr: Pointer, offset: u32, value: i16) => void,
    _flecs_component_get_member_i16: (component_ptr: Pointer, offset: u32) => i8,
    _flecs_component_set_member_i32: (component_ptr: Pointer, offset: u32, value: i32) => void,
    _flecs_component_get_member_i32: (component_ptr: Pointer, offset: u32) => i8,
    _flecs_component_set_member_i64: (component_ptr: Pointer, offset: u32, value: i64) => void,
    _flecs_component_get_member_i64: (component_ptr: Pointer, offset: u32) => i8,
    _flecs_component_set_member_f32: (component_ptr: Pointer, offset: u32, value: f32) => void,
    _flecs_component_get_member_f32: (component_ptr: Pointer, offset: u32) => f32,
    _flecs_component_set_member_f64: (component_ptr: Pointer, offset: u32, value: f64) => void,
    _flecs_component_get_member_f64: (component_ptr: Pointer, offset: u32) => f64,
    _flecs_component_set_member_string: (component_ptr: Pointer, offset: u32, value: u32) => f64,
    _flecs_component_get_member_string: (component_ptr: Pointer, offset: u32) => Pointer,
    _flecs_component_set_member_f32array: (component_ptr: Pointer, offset: u32, value: u32) => f64,
    _flecs_component_get_member_f32array: (component_ptr: Pointer, offset: u32) => Pointer,
}

export const flecs_core: EmscriptenModuleExtended & CoreAPI = window['flecs_core']
