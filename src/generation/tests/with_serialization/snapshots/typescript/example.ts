import { Serializer, Deserializer } from "./serde";
type int32 = number;
type Optional<T> = T | null;
type Seq<T> = T[];
type str = string;

function serializeArray<T>(
    value: T[],
    serializer: Serializer,
    serializeElement: (item: T, serializer: Serializer) => void,
): void {
    serializer.serializeLen(value.length);
    value.forEach((item) => {
        serializeElement(item, serializer);
    });
}

function deserializeArray<T>(
    deserializer: Deserializer,
    deserializeElement: (deserializer: Deserializer) => T,
): T[] {
    const length = deserializer.deserializeLen();
    const list: T[] = [];
    for (let i = 0; i < length; i++) {
        list.push(deserializeElement(deserializer));
    }
    return list;
}

function serializeMap<K, V>(
    value: Map<K, V>,
    serializer: Serializer,
    serializeEntry: (key: K, value: V, serializer: Serializer) => void,
): void {
    serializer.serializeLen(value.size);
    const offsets: number[] = [];
    for (const [k, v] of value.entries()) {
        offsets.push(serializer.getBufferOffset());
        serializeEntry(k, v, serializer);
    }
    serializer.sortMapEntries(offsets);
}

function deserializeMap<K, V>(
    deserializer: Deserializer,
    deserializeEntry: (deserializer: Deserializer) => [K, V],
): Map<K, V> {
    const length = deserializer.deserializeLen();
    const obj = new Map<K, V>();
    for (let i = 0; i < length; i++) {
        const [key, value] = deserializeEntry(deserializer);
        obj.set(key, value);
    }
    return obj;
}

function serializeOption<T>(
    value: T | null,
    serializer: Serializer,
    serializeElement: (value: T, serializer: Serializer) => void,
): void {
    if (value !== null) {
        serializer.serializeOptionTag(true);
        serializeElement(value, serializer);
    } else {
        serializer.serializeOptionTag(false);
    }
}

function deserializeOption<T>(
    deserializer: Deserializer,
    deserializeElement: (deserializer: Deserializer) => T,
): T | null {
    const tag = deserializer.deserializeOptionTag();
    if (!tag) {
        return null;
    } else {
        return deserializeElement(deserializer);
    }
}

function serializeSet<T>(
    value: T[],
    serializer: Serializer,
    serializeElement: (item: T, serializer: Serializer) => void,
): void {
    serializer.serializeLen(value.length);
    value.forEach((item) => {
        serializeElement(item, serializer);
    });
}

function deserializeSet<T>(
    deserializer: Deserializer,
    deserializeElement: (deserializer: Deserializer) => T,
): T[] {
    const length = deserializer.deserializeLen();
    const list: T[] = [];
    for (let i = 0; i < length; i++) {
        list.push(deserializeElement(deserializer));
    }
    return list;
}

export class Child {
    constructor (public name: str) {
    }

    public serialize(serializer: Serializer): void {
        serializer.serializeStr(this.name);
    }

    static deserialize(deserializer: Deserializer): Child {
        const name = deserializer.deserializeStr();
        return new Child(name);
    }
}

export class MyStruct {
    constructor (public string_to_int: Map<str,int32>, public map_to_list: Map<str,Seq<int32>>, public option_of_vec_of_set: Optional<Seq<Seq<str>>>, public parent: Parent) {
    }

    public serialize(serializer: Serializer): void {
        serializeMap(this.string_to_int, serializer, (key, value, serializer) => {
            serializer.serializeStr(key);
            serializer.serializeI32(value);
        });
        serializeMap(this.map_to_list, serializer, (key, value, serializer) => {
            serializer.serializeStr(key);
            serializeArray(value, serializer, (item, serializer) => {
                serializer.serializeI32(item);
            });
        });
        serializeOption(this.option_of_vec_of_set, serializer, (value, serializer) => {
            serializeArray(value, serializer, (item, serializer) => {
                serializeSet(item, serializer, (item, serializer) => {
                    serializer.serializeStr(item);
                });
            });
        });
        this.parent.serialize(serializer);
    }

    static deserialize(deserializer: Deserializer): MyStruct {
        const string_to_int = deserializeMap(deserializer, (deserializer) => {
            const key = deserializer.deserializeStr();
            const value = deserializer.deserializeI32();
            return [key, value];
        });
        const map_to_list = deserializeMap(deserializer, (deserializer) => {
            const key = deserializer.deserializeStr();
            const value = deserializeArray(deserializer, (deserializer) => {
                return deserializer.deserializeI32();
            });
            return [key, value];
        });
        const option_of_vec_of_set = deserializeOption(deserializer, (deserializer) => {
            return deserializeArray(deserializer, (deserializer) => {
                return deserializeSet(deserializer, (deserializer) => {
                    return deserializer.deserializeStr();
                });
            });
        });
        const parent = Parent.deserialize(deserializer);
        return new MyStruct(string_to_int,map_to_list,option_of_vec_of_set,parent);
    }
}

export abstract class Parent {
    abstract serialize(serializer: Serializer): void;

    static deserialize(deserializer: Deserializer): Parent {
        const index = deserializer.deserializeVariantIndex();
        switch (index) {
            case 0: return ParentVariantChild.load(deserializer);
            default: throw new Error("Unknown variant index for Parent: " + index);
        }
    }
}

export class ParentVariantChild extends Parent {
    constructor (public value: Child) {
        super();
    }

    public serialize(serializer: Serializer): void {
        serializer.serializeVariantIndex(0);
        this.value.serialize(serializer);
    }

    static load(deserializer: Deserializer): ParentVariantChild {
        const value = Child.deserialize(deserializer);
        return new ParentVariantChild(value);
    }
}
