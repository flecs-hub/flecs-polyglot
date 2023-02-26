import { flecs_core, Pointer } from './emscripten'
import SparseMap from './utils/sparse-map'
import { v4 as uuidv4 } from 'uuid'

export enum Type {
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
export type JsPrimitive = string | number | boolean 
export type Types = { [key: string]: Type }
export type TypesInfo = { 
    [key: string]: {
        type: Type,
        cName: Pointer,
        index: number,
        offset: number
    }
}
export type ComponentValues<I> = { [K in keyof I]: I[K] | JsPrimitive }
export type FlecsComponent<I, T> = {
    [key in keyof ComponentValues<I>]: ComponentValues<I>[keyof ComponentValues<I>] | Types[keyof Types]
} 
export type EntityID = number
export type ComponentID = number
export type ComponentName = string
export type TagName = string

export const TypeSizes = {
    [Type.U8]: 1,
    [Type.U16]: 2,
    [Type.U32]: 4,
    [Type.U64]: 8,
    [Type.I8]: 1,
    [Type.I16]: 2,
    [Type.I32]: 4,
    [Type.I64]: 8,
    [Type.F32]: 4,
    [Type.F64]: 8,
    [Type.Bool]: 1,
    [Type.String]: 4,
    [Type.Array]: 4
}

export const checkType = (value: JsPrimitive): Type => {
    switch (typeof value) {
        case 'string':
            return Type.String
        case 'number':
            return Type.F64
        case 'boolean':
            return Type.Bool
        default:
            throw new Error("Not a primitive type")
    }
}

// ComponentID -> ComponentType
const ComponentsTypeCache = new SparseMap<ComponentType>(10_000)
// ComponentName -> ComponentID
const ComponentIDCache = new Map<ComponentName, ComponentID>()

export class ComponentType {
    public id: EntityID = 0
    public types: Types = {}
}

export class Component {
    public id: EntityID = 0
    public ptr: Pointer = 0
    public typesInfo: TypesInfo = {}

    static numOfInternalFields = 4
    static isMembers = (key: string) => key !== 'id' && key !== 'ptr' && key !== 'typesInfo' && key !== 'types'
}

export class Tag {
    public id: EntityID = 0
}

export class Entity {
    public id: EntityID = 0

    constructor(name?: string) {
        const cName = flecs_core.allocateUTF8(name ? name : uuidv4())
        this.id = flecs_core._flecs_entity_create(cName)
        flecs_core._m_free(cName)
    }

    add(...components: Component[]): Entity {
        for (const component of components) {
            const flecsComponent = ComponentIDCache.has(component.constructor.name) ? 
                // Get component type info from cache
                // Merge it with new component instance
                Object.assign(component, ComponentsTypeCache.get(ComponentIDCache.get(component.constructor.name)))
            :
                // Create new component with type info
                World.registerComponent(Object.getPrototypeOf(component).constructor) && Object.assign(component, ComponentsTypeCache.get(ComponentIDCache.get(component.constructor.name)))
            
            // Set component pointer from C API so that we can
            // later get and modify it's struct members
            flecsComponent.ptr = flecs_core._flecs_entity_add_component(this.id, flecsComponent.id)
        }
        return this
    }
    
    addTags(...tags: Tag[]): Entity {
        for (const tag of tags) {
            const tagID = ComponentIDCache.has(tag.constructor.name) ? 
                ComponentIDCache.get(tag.constructor.name)
            :
                World.registerTag(Object.getPrototypeOf(tag).constructor).id
                
            flecs_core._flecs_entity_add_tag(this.id, tagID)
        }
        return this
    }

    childOf(parent: Entity): Entity {
        flecs_core._flecs_entity_childof(this.id, parent.id)
        return this
    }

    children(): Array<Entity> { 
        const iterPtr = flecs_core._flecs_entity_children(this.id)
        flecs_core._flecs_term_next(iterPtr)

        const count = flecs_core._flecs_iter_count(iterPtr)
        const childrenPtr = flecs_core._flecs_child_entities(iterPtr)
        const entities = new Array<Entity>()
        // Iterate over HEAPU32 and get the children
        for (let i = 0; i < count; i++) {
            const child = flecs_core.HEAPU32[childrenPtr / 4 + i]
            const entity = new Entity()
            entity.id = child
            entities.push(entity)
        }

        return entities
    }

    get<T extends Component>(componentType: typeof Component): T {
        const component = World.createComponent(componentType)
        component.ptr = flecs_core._flecs_entity_get_component(this.id, component.id)
        return component as T
     }

    remove(component: Component) {}
}

export class World {
    static createComponent<T extends Component>(componentType: new() => T): T {
        const component = new componentType()
        const cachedComponentType = ComponentsTypeCache.get(ComponentIDCache.get(componentType.name)) 
        Object.assign(component, cachedComponentType)

        // Iterate through component and map getters and setters
        // to change the values of thes struct in the C API
        for (const [key, value] of Object.entries(component)) {
            if(Component.isMembers(key)) {
                // Getters and setters for component members
                Object.defineProperty(component, key, {
                    get() {
                        if(component.ptr) {
                            const typesInfo = component.typesInfo[key]
                            switch(typesInfo.type) {
                                case Type.U8:
                                    return flecs_core._flecs_component_get_member_u8(component.ptr, typesInfo.offset)
                                case Type.U16:
                                    return flecs_core._flecs_component_get_member_u16(component.ptr, typesInfo.offset)
                                case Type.U32:
                                    return flecs_core._flecs_component_get_member_u32(component.ptr, typesInfo.offset)
                                case Type.U64:
                                    return flecs_core._flecs_component_get_member_u64(component.ptr, typesInfo.offset)
                                case Type.I8:
                                    return flecs_core._flecs_component_get_member_i8(component.ptr, typesInfo.offset)
                                case Type.I16:
                                    return flecs_core._flecs_component_get_member_i16(component.ptr, typesInfo.offset)
                                case Type.I32:
                                    return flecs_core._flecs_component_get_member_i32(component.ptr, typesInfo.offset)
                                case Type.I64:
                                    return flecs_core._flecs_component_get_member_i64(component.ptr, typesInfo.offset)
                                case Type.F32:
                                    return flecs_core._flecs_component_get_member_f32(component.ptr, typesInfo.offset)
                                case Type.F64:
                                    return flecs_core._flecs_component_get_member_f64(component.ptr, typesInfo.offset)
                                case Type.String:
                                {
                                    const stringPtr = flecs_core._flecs_component_get_member_string(component.ptr, typesInfo.offset)
                                    return flecs_core.UTF8ToString(stringPtr)
                                }
                                case Type.U32Array:
                                {
                                    const arrayPtr = flecs_core._flecs_component_get_member_f32array(component.ptr, typesInfo.offset)
                                    let length = new Uint32Array(flecs_core.HEAPU32.buffer, arrayPtr, 1)[0]

                                    // TODO: Memory in the heap is a floating point value
                                    // until the pointer to the array is first set.
                                    // Refactor this workaround later.
                                    if(!Number.isSafeInteger(length)) 
                                        length = 0

                                    // Get array from emscripten heap
                                    return new Uint32Array(flecs_core.HEAPF32.buffer, arrayPtr + TypeSizes[Type.F32], length)
                                }
                                case Type.F32Array:
                                {
                                    const arrayPtr = flecs_core._flecs_component_get_member_f32array(component.ptr, typesInfo.offset)
                                    let length = new Float32Array(flecs_core.HEAPF32.buffer, arrayPtr, 1)[0]

                                    // TODO: Memory in the heap is a floating point value
                                    // until the pointer to the array is first set.
                                    // Refactor this workaround later.
                                    if(!Number.isSafeInteger(length)) 
                                        length = 0

                                    // Get array from emscripten heap
                                    return new Float32Array(flecs_core.HEAPF32.buffer, arrayPtr + TypeSizes[Type.F32], length)
                                }
                            }
                        }
                        return value
                    },
                    set(value: any) {
                        if(component.ptr) {
                            const typesInfo = component.typesInfo[key]
                            switch(typesInfo.type) {
                                case Type.U8:
                                    flecs_core._flecs_component_set_member_u8(component.ptr, typesInfo.offset, value)
                                    break
                                case Type.U16:
                                    flecs_core._flecs_component_set_member_u16(component.ptr, typesInfo.offset, value)
                                    break
                                case Type.U32:
                                    flecs_core._flecs_component_set_member_u32(component.ptr, typesInfo.offset, value)
                                    break
                                case Type.U64:
                                    flecs_core._flecs_component_set_member_u64(component.ptr, typesInfo.offset, value)
                                    break
                                case Type.I8:
                                    flecs_core._flecs_component_set_member_i8(component.ptr, typesInfo.offset, value)
                                    break
                                case Type.I16:
                                    flecs_core._flecs_component_set_member_i16(component.ptr, typesInfo.offset, value)
                                    break
                                case Type.I32:
                                    flecs_core._flecs_component_set_member_i32(component.ptr, typesInfo.offset, value)
                                    break
                                case Type.I64:
                                    flecs_core._flecs_component_set_member_i64(component.ptr, typesInfo.offset, value)
                                    break
                                case Type.F32:
                                    flecs_core._flecs_component_set_member_f32
                            (component.ptr, typesInfo.offset, value)
                                break
                                case Type.F64:
                                    flecs_core._flecs_component_set_member_f32(component.ptr, typesInfo.offset, value)
                                    break
                                case Type.String:
                                {
                                    const stringPtr = flecs_core.allocateUTF8(value)
                                    // TODO: Free memory
                                    flecs_core._flecs_component_set_member_string(component.ptr, typesInfo.offset, stringPtr)
                                   break
                                }
                                case Type.U32Array:
                                {
                                    // Allocate array of uint32 values in memory'
                                    // Add length of array to the first element of the array
                                    const valueWithLength = new Uint32Array(value.length + 1)
                                    valueWithLength[0] = value.length
                                    // Append rest of values to the array
                                    valueWithLength.set(value, 1)

                                    // Allocate memory for the array
                                    const arrayBuffer = flecs_core._malloc((valueWithLength as Uint32Array).length * (valueWithLength as Uint32Array).BYTES_PER_ELEMENT)
                                    // TODO: Free memory
                                    // Write array of string pointers to memory
                                    flecs_core.HEAPF32.set((valueWithLength as Uint32Array), arrayBuffer / (valueWithLength as Uint32Array).BYTES_PER_ELEMENT)

                                    // Set component member
                                    flecs_core._flecs_component_set_member_f32array(component.ptr, typesInfo.offset, arrayBuffer)
                                    break
                                }
                                case Type.F32Array:
                                {
                                    // Allocate array of float32 values in memory'
                                    // Add length of array to the first element of the array
                                    const valueWithLength = new Float32Array(value.length + 1)
                                    valueWithLength[0] = value.length
                                    // Append rest of values to the array
                                    valueWithLength.set(value, 1)

                                    // Allocate memory for the array
                                    const arrayBuffer = flecs_core._malloc((valueWithLength as Float32Array).length * (valueWithLength as Float32Array).BYTES_PER_ELEMENT)
                                    // TODO: Free memory
                                    // Write array of string pointers to memory
                                    flecs_core.HEAPF32.set((valueWithLength as Float32Array), arrayBuffer / (valueWithLength as Float32Array).BYTES_PER_ELEMENT)

                                    // Set component member
                                    flecs_core._flecs_component_set_member_f32array(component.ptr, typesInfo.offset, arrayBuffer)
                                   break
                                }
                                default:
                                    break
                            }
                        }
                    }
                })
            }
        }
        return component
    }

    static registerComponent(_component: typeof Component, types: Types = {}) {
        const component = Object.assign(new ComponentType(), new _component())
        // Create C data
        // Name of the component
        const cName = flecs_core.allocateUTF8(_component.name)
        const members = Object.entries(component)
        // Names of component members
        const cNames = new Uint32Array(members.length - Component.numOfInternalFields)
        const cTypes = new Uint8Array(members.length - Component.numOfInternalFields)

        // Iterate over members and create type info for flecs component metadata
        let i = 0
        let offset = 0
        for (const [key, value] of members) {
            if(Component.isMembers(key)) {    
                // Allocate string for member name, return pointer to string
                const cName = cNames[i - Component.numOfInternalFields] = flecs_core.allocateUTF8(key)
                const cType = types[key] ? types[key] : checkType(value)
                cNames[i - Component.numOfInternalFields] = cName
                cTypes[i - Component.numOfInternalFields] = cType
                // Create type info for member
                component.typesInfo[key] = {
                    // If types have not been defined, reflect the types from the JavaScript values
                    type: cType,
                    cName,
                    index: i - Component.numOfInternalFields,
                    offset
                }
                // Increment field offset by size of type in bytes
                offset += TypeSizes[component.typesInfo[key].type]
            }
            i++
        }

        // Allocate array of string pointers
        const cNamesBuffer = flecs_core._malloc(cNames.length * cNames.BYTES_PER_ELEMENT)
        
        // Write array of string pointers to memory
        flecs_core.HEAPU32.set(cNames, cNamesBuffer / cNames.BYTES_PER_ELEMENT)

        // Allocate array of component types (u8s)
        const cTypesBuffer = flecs_core._malloc(cTypes.length * cTypes.BYTES_PER_ELEMENT)

        // Write array of uint8s to memory
        flecs_core.HEAPU8.set(cTypes, cTypesBuffer / cTypes.BYTES_PER_ELEMENT)
        
        // Create component
        component.id = flecs_core._flecs_component_create(cName, cNamesBuffer, cNames.length, cTypesBuffer, cTypes.length)
        
        // Update caches
        ComponentIDCache.set(_component.name, component.id)
        ComponentsTypeCache.set(component.id, component)

        // Free memory
        flecs_core._m_free(cName)
        flecs_core._m_free(cNamesBuffer)
        flecs_core._m_free(cTypesBuffer)

        return component
    }
    
    static registerTag(_tag: typeof Tag) {
        const tag = new _tag()

        // Name of the tag
        const tName = flecs_core.allocateUTF8(_tag.name)

        // Create component
        tag.id = flecs_core._flecs_tag_create(tName)
        
        // Update caches
        ComponentIDCache.set(_tag.name, tag.id)
        // ComponentsTypeCache.set(component.id, component)

        // Free memory
        flecs_core._m_free(tName)

        return tag
    }

    // TODO: Turn this into variadic function
    // and pass in array instead of single
    // component id
    static query(...components: (typeof Component)[]): Query {
        const componentIds = new Array<number>()
        const indexes = new Array<ComponentName>()

        for (const component  of components) {
            if(!ComponentIDCache.has(component.name))
                throw new Error(`Component ${component.name} has not been registered`)
    
            const id = ComponentIDCache.get(component.name)
            componentIds.push(id)
            indexes.push(component.name)
        }

        const BYTES_PER_ELEMENT = 4
        // Allocate array of component ids
        const buffer = flecs_core._malloc(componentIds.length * BYTES_PER_ELEMENT)
        // Write array of component ids to memory
        flecs_core.HEAPU32.set(componentIds, buffer / BYTES_PER_ELEMENT)

        // Create query
        const query = new Query(flecs_core._flecs_query_create(buffer, componentIds.length), indexes)

        // Free memory
        flecs_core._m_free(buffer)
        
        return query
    }
}

export class Query {
    public ptr: Pointer = 0
    public iterPtr: Pointer = 0
    public indexes: Array<ComponentName> = new Array<ComponentName>()
    
    constructor(ptr: Pointer, indexes: Array<ComponentName>) {
        this.ptr = ptr
        this.indexes = indexes
    }

    iter(): Pointer {
        this.iterPtr = flecs_core._flecs_query_iter(this.ptr)
        return this.iterPtr
    }

    next(): boolean {
       return flecs_core._flecs_query_next(this.iterPtr)
    }

    field<T extends Component>(componentType: new() => T): Array<T> {
        const count = flecs_core._flecs_iter_count(this.iterPtr)
        const termIndex = this.indexes.indexOf(componentType.name)

        /// Get iter array ptr which is an array of array of component pointers
        const iterPtrsPtr = flecs_core._flecs_query_iter_ptrs(this.iterPtr, termIndex)
        const ptrIndex = (iterPtrsPtr / 4)
        // Convert to JS array
        const iterArrayPtr = flecs_core.HEAPU32[ptrIndex]

        // Create array of components
        const components = new Array<T>()
        for (let i = 0; i < count; i++) {
            const component = World.createComponent(componentType)
            component.ptr = flecs_core._flecs_query_iter_component(iterArrayPtr, i, count, component.id)
            components.push(component)
        }

        return components
    }
}

export class System {
    public query: Query
    constructor() {}
    update(deltaM: number) {}
}