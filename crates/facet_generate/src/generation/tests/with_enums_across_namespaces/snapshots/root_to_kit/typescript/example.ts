import type { Serializer, Deserializer } from "./serde";
import * as Kit from "./kit";
type Optional<T> = T | null;
type Seq<T> = T[];
type uint64 = bigint;

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

export class Card {
    constructor (public presence: Kit.Presence, public shape: Kit.Shape, public shapes: Seq<Optional<Kit.Shape>>, public badge: Kit.Badge) {
    }

    public serialize(serializer: Serializer): void {
        Kit.serializePresence(this.presence, serializer);
        Kit.serializeShape(this.shape, serializer);
        serializeArray(this.shapes, serializer, (item, serializer) => {
            serializeOption(item, serializer, (value, serializer) => {
                Kit.serializeShape(value, serializer);
            });
        });
        this.badge.serialize(serializer);
    }

    static deserialize(deserializer: Deserializer): Card {
        const presence = Kit.deserializePresence(deserializer);
        const shape = Kit.deserializeShape(deserializer);
        const shapes = deserializeArray(deserializer, (deserializer) => {
            return deserializeOption(deserializer, (deserializer) => {
                return Kit.deserializeShape(deserializer);
            });
        });
        const badge = Kit.Badge.deserialize(deserializer);
        return new Card(presence,shape,shapes,badge);
    }
}

export class Presence {
    constructor (public since: uint64) {
    }

    public serialize(serializer: Serializer): void {
        serializer.serializeU64(this.since);
    }

    static deserialize(deserializer: Deserializer): Presence {
        const since = deserializer.deserializeU64();
        return new Presence(since);
    }
}

export class Sighting {
    constructor (public last_seen: Presence) {
    }

    public serialize(serializer: Serializer): void {
        this.last_seen.serialize(serializer);
    }

    static deserialize(deserializer: Deserializer): Sighting {
        const last_seen = Presence.deserialize(deserializer);
        return new Sighting(last_seen);
    }
}
