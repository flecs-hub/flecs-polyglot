import { flecs_core, Pointer } from './emscripten'
import SparseMap from './utils/sparse-map'

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
    Array
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
        case "string":
            return Type.String
        case "number":
            return Type.F64
        case "boolean":
            return Type.Bool
        default:
            throw new Error("Not a primitive type")
    }
}

// ComponentID -> ComponentType
const ComponentsTypeCache = new SparseMap<Component>(10_000)
// ComponentName -> ComponentID
const ComponentIDCache = new Map<ComponentName, ComponentID>()

// TODO: Make a seperate class for ComponentType
// and seperate it from queryable component instances
export class Component {
    public id: EntityID = 0
    public ptr: Pointer = 0
    public types: Types = {}
    public typesInfo: TypesInfo = {}

    static numOfInternalFields = 4
    static isMembers = (key: string) => key !== 'id' && key !== 'ptr' && key !== 'types' && key !== 'typesInfo'

    constructor(types?: Types) {
        this.types = types
    }
}

export class Entity {
    public id: EntityID = 0

    constructor() {
        this.id = flecs_core._flecs_entity_create()
    }

    add(...components: Component[]) {
        for (const component of components) {
            const flecsComponent = ComponentIDCache.has(component.constructor.name) ? 
                // Get component type info from cache
                // Merge it with new component instance
                Object.assign(component, ComponentsTypeCache.get(ComponentIDCache.get(component.constructor.name)))
            :
                // Create new component with type info
                World.registerComponent(component)
            
            // Set component pointer from C API so that we can
            // later get and modify it's struct members
            flecsComponent.ptr = flecs_core._flecs_entity_add_component(this.id, flecsComponent.id)
        }
    }

    get(): Component { return null }

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
                Object.defineProperty(component, key, {
                    get() {
                        if(component.ptr) {
                            const typesInfo = component.typesInfo[key]
                            switch(typesInfo.type) {
                                case Type.F32:
                                    return flecs_core._flecs_component_get_member_float(component.ptr, typesInfo.offset)
                            }
                        }
                        return value
                    },
                    set(value: any) {
                        if(component.ptr) {
                            const typesInfo = component.typesInfo[key]
                            switch(typesInfo.type) {
                                case Type.F32:
                                    flecs_core._flecs_component_set_member_float
                            (component.ptr, typesInfo.offset, value)
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
    static registerComponent<T extends Component>(component: T) {
        // Create C data
        // Name of the component
        const cName = flecs_core.allocateUTF8(component.constructor.name)
        const members = Object.entries(component)
        // Names of component members
        const cNames = new Uint32Array(members.length - Component.numOfInternalFields)

        // Iterate over members and create type info for flecs component metadata
        let i = 0
        let offset = 0
        for (const [key, value] of members) {
            if(Component.isMembers(key)) {    
                // Allocate string for member name, return pointer to string
                const cName = cNames[i - Component.numOfInternalFields] = flecs_core.allocateUTF8(key)
                cNames[i - Component.numOfInternalFields] = cName
                // Create type info for member
                component.typesInfo[key] = {
                    // If types have not been defined, reflect the types from the JavaScript values
                    type: Object.keys(component.types).length === 0 ? checkType(value) : component.types[key],
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
        const buffer = flecs_core._malloc(cNames.length * cNames.BYTES_PER_ELEMENT)
        
        // Write array of string pointers to memory
        flecs_core.HEAPU32.set(cNames, buffer / cNames.BYTES_PER_ELEMENT)
        
        // Create component
        component.id = flecs_core._flecs_component_create(cName, buffer, cNames.length, buffer, cNames.length)
        
        // Update caches
        ComponentIDCache.set(component.constructor.name, component.id)
        ComponentsTypeCache.set(component.id, component)

        return component
    }

    // TODO: Turn this into variadic function
    // and pass in array instead of single
    // component id
    static query(component: typeof Component): Query {
        if(!ComponentIDCache.has(component.name))
            throw new Error(`Component ${component.name} has not been registered`)

        const indexes = new Array<ComponentName>()
        indexes.push(component.name)
        const id = ComponentIDCache.get(component.name)
        return new Query(flecs_core._flecs_query_create(id), indexes)
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
        const count = flecs_core._flecs_query_iter_count(this.iterPtr)
        const componentIndex = this.indexes.indexOf(componentType.name)
        
        // Get iter array ptr which is an array of array of component pointers
        const iterArrayPtr = flecs_core._flecs_query_iter_ptrs(this.iterPtr, componentIndex)

        /*
        TODO: Convert array of component pointers to JS array
        const ptrIndex = iterArrayPtr / 4
        const componentPtrs = flecs_core.HEAPU32.subarray(ptrIndex, ptrIndex + count)
        */

        // Create array of components
        const components = new Array<T>()
        for (let i = 0; i < count; i++) {
            const component = World.createComponent(componentType)
            component.ptr = flecs_core._flecs_query_iter_component(iterArrayPtr, i, count)
            components.push(component)
        }

        return components
    }
}

export class System {
    public query: Pointer = 0
    constructor() {}
    run() {}
}