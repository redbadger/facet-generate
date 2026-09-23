import type { Serializer, Deserializer } from "./serde";
type Seq<T> = T[];
type uint32 = number;
type unit = null;

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

export class Set {
    constructor (public value: uint32) {
    }

    public serialize(serializer: Serializer): void {
        serializer.serializeU32(this.value);
    }

    static deserialize(deserializer: Deserializer): Set {
        const value = deserializer.deserializeU32();
        return new Set(value);
    }
}

export class Tray {
    constructor (public nothing: unit, public ids: Seq<uint32>) {
    }

    public serialize(serializer: Serializer): void {
        serializer.serializeUnit(this.nothing);
        serializeSet(this.ids, serializer, (item, serializer) => {
            serializer.serializeU32(item);
        });
    }

    static deserialize(deserializer: Deserializer): Tray {
        const nothing = deserializer.deserializeUnit();
        const ids = deserializeSet(deserializer, (deserializer) => {
            return deserializer.deserializeU32();
        });
        return new Tray(nothing,ids);
    }
}
