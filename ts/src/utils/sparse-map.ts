import Iterator from './iterator';
import { getPointerArray } from './typed-arrays';

class SparseMap<T> {
    size: number = 0;
    length: number;
    dense: Uint32Array;
    sparse: Uint32Array;
    vals: T[];
    constructor(length: number) {
        let Values = Array as new(length: number) => T[];
        let ByteArray = getPointerArray(length);

        this.length = length;
        this.dense = new ByteArray(length) as any;
        this.sparse = new ByteArray(length) as any;
        this.vals = new Values(length);
    }

    clear(): void {
        this.size = 0;
    }

    has(member: number): boolean {
        let index = this.sparse[member];

        return (
            index < this.size &&
            this.dense[index] === member
        );
    }

    get(member: number): T | undefined {
        let index = this.sparse[member];

        if (index < this.size && this.dense[index] === member)
            return this.vals[index];

        return;
    }

    set(member: number, value: T): this {
        let index = this.sparse[member];

        if (index < this.size && this.dense[index] === member) {
            this.vals[index] = value;
            return this;
        }

        this.dense[this.size] = member;
        this.sparse[member] = this.size;
        this.vals[this.size] = value;
        this.size++;

        return this;
    }

    delete(member: number): boolean {
        let index = this.sparse[member];

        if (index >= this.size || this.dense[index] !== member)
            return false;

        index = this.dense[this.size - 1];
        this.dense[this.sparse[member]] = index;
        this.sparse[index] = this.sparse[member];
        this.size--;

        return true;
    }

    forEach(callback: (val: T, key: number) => void, scope?: any): void {
        scope = arguments.length > 1 ? scope : this;

        for (let i = 0; i < this.size; i++)
            callback.call(scope, this.vals[i], this.dense[i]);
    }

    keys(): Iterator<number> {
        let size = this.size,
            dense = this.dense,
            i = 0;

        return new Iterator(function () {
            if (i < size) {
                let item = dense[i];
                i++;

                return {
                    value: item
                };
            }

            return {
                done: true
            };
        } as any);
    }
}

export default SparseMap