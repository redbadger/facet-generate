import * as $json from "./serde/json";
import * as Kit from "./kit";
type Optional<T> = T | null;
type Seq<T> = T[];
type uint64 = bigint;

export class Card {
    constructor (public presence: Kit.Presence, public shape: Kit.Shape, public shapes: Seq<Optional<Kit.Shape>>, public badge: Kit.Badge) {
    }

    static toJson(value: Card): $json.JsonValue {
        return {
            "presence": Kit.toJsonPresence(value.presence),
            "shape": Kit.toJsonShape(value.shape),
            "shapes": value.shapes.map((v0) => (v0 === null ? null : Kit.toJsonShape(v0))),
            "badge": Kit.Badge.toJson(value.badge),
        };
    }

    static fromJson(json: unknown): Card {
        const obj = $json.readObject(json, "Card");
        return new Card(
            Kit.fromJsonPresence($json.field(obj, "presence")),
            Kit.fromJsonShape($json.field(obj, "shape")),
            $json.readSeq($json.field(obj, "shapes"), (j0) => $json.readOption(j0, Kit.fromJsonShape)),
            Kit.Badge.fromJson($json.field(obj, "badge")),
        );
    }

    static jsonSerialize(value: Card): string {
        return $json.stringify(Card.toJson(value));
    }

    static jsonDeserialize(text: string): Card {
        return Card.fromJson($json.parse(text));
    }
}

export class Presence {
    constructor (public since: uint64) {
    }

    static toJson(value: Presence): $json.JsonValue {
        return {
            "since": $json.writeBigInt(value.since),
        };
    }

    static fromJson(json: unknown): Presence {
        const obj = $json.readObject(json, "Presence");
        return new Presence(
            $json.readU64($json.field(obj, "since")),
        );
    }

    static jsonSerialize(value: Presence): string {
        return $json.stringify(Presence.toJson(value));
    }

    static jsonDeserialize(text: string): Presence {
        return Presence.fromJson($json.parse(text));
    }
}

export class Sighting {
    constructor (public last_seen: Presence) {
    }

    static toJson(value: Sighting): $json.JsonValue {
        return {
            "last_seen": Presence.toJson(value.last_seen),
        };
    }

    static fromJson(json: unknown): Sighting {
        const obj = $json.readObject(json, "Sighting");
        return new Sighting(
            Presence.fromJson($json.field(obj, "last_seen")),
        );
    }

    static jsonSerialize(value: Sighting): string {
        return $json.stringify(Sighting.toJson(value));
    }

    static jsonDeserialize(text: string): Sighting {
        return Sighting.fromJson($json.parse(text));
    }
}
