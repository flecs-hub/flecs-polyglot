/// <reference types="emscripten" />

import { EntityID, Type } from 'ecs'

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
}

export interface CoreAPI {
    _flecs_component_create: (name: Pointer, memberNames: Pointer, memberNamesSize: u32, memberValues: Pointer, memberValuesSize: u32) => Pointer,
    _flecs_component_set_member_float: (component_ptr: Pointer, offset: u32, value: f32) => void,
    _flecs_component_get_member_float: (component_ptr: Pointer, offset: u32) => f64,
    _flecs_entity_create: () => Pointer,
    _flecs_entity_add_component: (entity: EntityID, component: EntityID) => Pointer,
    _flecs_query_create: (component: EntityID) => Pointer,
    _flecs_query_next: (iter: Pointer) => boolean,
    _flecs_query_iter: (query: Pointer) => Pointer,
    _flecs_query_iter_ptrs: (iter: Pointer) => Pointer,
    _flecs_query_iter_count: (iter: Pointer) => i32,
}

export const flecs_core: EmscriptenModuleExtended & CoreAPI = window['flecs_core']
