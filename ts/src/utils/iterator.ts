class Iterator<T> implements IterableIterator<T> {
    constructor(next: () => {done: boolean, value: T}) {
      if (typeof next !== 'function') {
        throw new Error('obliterator/iterator: expecting a function!');
      }
    }
  
    [Symbol.iterator](): IterableIterator<T> {
      return this;
    }
  
    next(): IteratorResult<T> {
      return this.next();
    }
  
    static of<T>(...values: T[]): Iterator<T> {
      let i = 0;
      return new Iterator(() => {
        if (i >= values.length) return {done: true};
        return {done: false, value: values[i++]} as any;
      });
    }
  
    static empty(): Iterator<any> {
      return new Iterator(() => {
        return {done: true} as any;
      });
    }
  
    static fromSequence<T>(sequence: T[]): Iterator<T> {
      let i = 0;
      return new Iterator(() => {
        if (i >= sequence.length) return {done: true};
        return {done: false, value: sequence[i++]};
      });
    }
  
    static is(value: any): value is Iterator<any> {
      return value instanceof Iterator ||
        (typeof value === 'object' && value !== null && typeof value.next === 'function');
    }
  }
  
  export default Iterator;
  