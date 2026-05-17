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

const HEX = '0123456789abcdef';

function uuidStringToBytes(value: Uuid): Uint8Array {
    const hex = (value as string).replace(/-/g, '');
    if (hex.length !== 32) {
        throw new Error(`Invalid UUID: ${value}`);
    }
    const bytes = new Uint8Array(16);
    for (let i = 0; i < 16; i++) {
        const hi = HEX.indexOf(hex[i * 2].toLowerCase());
        const lo = HEX.indexOf(hex[i * 2 + 1].toLowerCase());
        if (hi < 0 || lo < 0) {
            throw new Error(`Invalid UUID: ${value}`);
        }
        bytes[i] = (hi << 4) | lo;
    }
    return bytes;
}

function bytesToUuidString(bytes: Uint8Array): Uuid {
    if (bytes.length !== 16) {
        throw new Error(`UUID must be 16 bytes, got ${bytes.length}`);
    }
    let s = '';
    for (let i = 0; i < 16; i++) {
        s += HEX[(bytes[i] >> 4) & 0xf] + HEX[bytes[i] & 0xf];
        if (i === 3 || i === 5 || i === 7 || i === 9) {
            s += '-';
        }
    }
    return s as Uuid;
}

function serializeUuid(value: Uuid, serializer: Serializer): void {
    serializer.serializeBytes(uuidStringToBytes(value));
}

function deserializeUuid(deserializer: Deserializer): Uuid {
    return bytesToUuidString(deserializer.deserializeBytes());
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
