import { Serializer, Deserializer } from "./serde";
type Optional<T> = T | null;
type str = string;

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

export type Uuid = string & { readonly __uuid: unique symbol };

const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

function serializeUuid(value: Uuid, serializer: Serializer): void {
    serializer.serializeStr(value as string);
}

function deserializeUuid(deserializer: Deserializer): Uuid {
    const s = deserializer.deserializeStr();
    if (!UUID_RE.test(s)) {
        throw new Error(`Invalid UUID string: ${s}`);
    }
    return s.toLowerCase() as Uuid;
}

export class StructWithUuid {
    constructor (public id: Uuid, public parent_id: Optional<Uuid>, public name: str) {
    }

    public serialize(serializer: Serializer): void {
        serializeUuid(this.id, serializer);
        serializeOption(this.parent_id, serializer, (value, serializer) => {
            serializeUuid(value, serializer);
        });
        serializer.serializeStr(this.name);
    }

    static deserialize(deserializer: Deserializer): StructWithUuid {
        const id = deserializeUuid(deserializer);
        const parent_id = deserializeOption(deserializer, (deserializer) => {
            return deserializeUuid(deserializer);
        });
        const name = deserializer.deserializeStr();
        return new StructWithUuid(id,parent_id,name);
    }
}
