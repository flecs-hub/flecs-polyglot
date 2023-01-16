export const MAX_8BIT_INTEGER = Math.pow(2, 8) - 1;
export const MAX_16BIT_INTEGER = Math.pow(2, 16) - 1;
export const MAX_32BIT_INTEGER = Math.pow(2, 32) - 1;
export const MAX_SIGNED_8BIT_INTEGER = Math.pow(2, 7) - 1;
export const MAX_SIGNED_16BIT_INTEGER = Math.pow(2, 15) - 1;
export const MAX_SIGNED_32BIT_INTEGER = Math.pow(2, 31) - 1;

export function getPointerArray(size: number): typeof Uint8Array | typeof Uint16Array | typeof Uint32Array {
const maxIndex = size - 1;

if (maxIndex <= MAX_8BIT_INTEGER)
    return Uint8Array;

if (maxIndex <= MAX_16BIT_INTEGER)
    return Uint16Array;

if (maxIndex <= MAX_32BIT_INTEGER)
    return Uint32Array;

throw new Error('mnemonist: Pointer Array of size > 4294967295 is not supported.');
}

export function getSignedPointerArray(size: number): typeof Int8Array | typeof Int16Array | typeof Int32Array | typeof Float64Array {
const maxIndex = size - 1;

if (maxIndex <= MAX_SIGNED_8BIT_INTEGER)
    return Int8Array;

if (maxIndex <= MAX_SIGNED_16BIT_INTEGER)
    return Int16Array;

if (maxIndex <= MAX_SIGNED_32BIT_INTEGER)
    return Int32Array;

return Float64Array;
}

export function getNumberType(value: number): typeof Uint8Array | typeof Int8Array | typeof Uint16Array | typeof Int16Array | typeof Uint32Array | typeof Int32Array | typeof Float64Array {
if (value === (value | 0)) {
if (Math.sign(value) === -1) {
if (value <= 127 && value >= -128)
return Int8Array;
        if (value <= 32767 && value >= -32768)
            return Int16Array;

        return Int32Array;
    } else {
        if (value <= 255)
            return Uint8Array;

        if (value <= 65535)
            return Uint16Array;

        return Uint32Array;
    }
}

return Float64Array;
}

const TYPE_PRIORITY = {
    Uint8Array: 1,
    Int8Array: 2,
    Uint16Array: 3,
    Int16Array: 4,
    Uint32Array: 5,
    Int32Array: 6,
    Float32Array: 7,
    Float64Array: 8
};

/*
It's a function that takes an array of JavaScript numbers and an optional getter function as arguments, and returns the minimal TypedArray that can represent all the numbers in the array. The function iterates through the array and uses the getNumberType function to determine the type of each number, and then uses a TYPE_PRIORITY object to keep track of the highest priority type encountered so far. Once it's done iterating, it returns the highest priority type.
*/
export function getMinimalRepresentation<T>(array: T[], getter: (val: T) => number): typeof Uint8Array | typeof Int8Array | typeof Uint16Array | typeof Int16Array | typeof Uint32Array | typeof Int32Array | typeof Float32Array | typeof Float64Array {
    let maxType = null,
        maxPriority = 0,
        p,
        t,
        v,
        i,
        l;

    for (i = 0, l = array.length; i < l; i++) {
        v = getter ? getter(array[i]) : array[i];
        t = getNumberType(v);
        p = TYPE_PRIORITY[t.name];

        if (p > maxPriority) {
            maxPriority = p;
            maxType = t;
        }
    }

    return maxType;
}

/*
takes any value as an argument and returns a boolean indicating whether the value is a typed array
*/
export function isTypedArray(value: any): boolean {
  return typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView(value);
}

/*
takes any number of ArrayBufferView arrays and returns a new ArrayBufferView that is the concatenation of the input arrays
*/
export function concat(...arrays: ArrayBufferView[]): ArrayBufferView {
  let length = 0;
  for (const array of arrays) {
    length += array.length;
  }
  const result = new (arrays[0].constructor)(length);
  let offset = 0;
  for (const array of arrays) {
    result.set(array, offset);
    offset += array.length;
  }
  return result;
}

/*
takes a number as an argument and returns a new Uint8Array, Uint16Array or Uint32Array, depending on the length of the array, filled with the integers from 0 to length-1
*/
export function indices(length: number): Uint8Array | Uint16Array | Uint32Array {
  const PointerArray = getPointerArray(length);
  const array = new PointerArray(length);
  for (let i = 0; i < length; i++) {
    array[i] = i;
  }
  return array;
}