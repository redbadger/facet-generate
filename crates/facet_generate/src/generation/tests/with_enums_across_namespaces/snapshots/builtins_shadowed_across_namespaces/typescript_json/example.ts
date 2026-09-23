import type { Serializer, Deserializer } from "./serde";
import * as Kit from "./kit";
type Seq<T> = T[];
type uint32 = number;

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

export class Shelf {
    constructor (public set: Kit.Set, public ids: Seq<uint32>, public unit: Unit) {
    }

    public serialize(serializer: Serializer): void {
        this.set.serialize(serializer);
        serializeSet(this.ids, serializer, (item, serializer) => {
            serializer.serializeU32(item);
        });
        this.unit.serialize(serializer);
    }

    static deserialize(deserializer: Deserializer): Shelf {
        const set = Kit.Set.deserialize(deserializer);
        const ids = deserializeSet(deserializer, (deserializer) => {
            return deserializer.deserializeU32();
        });
        const unit = Unit.deserialize(deserializer);
        return new Shelf(set,ids,unit);
    }
}

export class Unit {
    constructor (public value: uint32) {
    }

    public serialize(serializer: Serializer): void {
        serializer.serializeU32(this.value);
    }

    static deserialize(deserializer: Deserializer): Unit {
        const value = deserializer.deserializeU32();
        return new Unit(value);
    }
}
