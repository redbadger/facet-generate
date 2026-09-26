import * as $json from "./serde/json";
import * as Kit from "./kit";
type Seq<T> = T[];
type uint32 = number;

export class Shelf {
    constructor (public set: Kit.Set, public ids: Seq<uint32>, public unit: Unit) {
    }

    static toJson(value: Shelf): $json.JsonValue {
        return {
            "set": Kit.Set.toJson(value.set),
            "ids": value.ids,
            "unit": Unit.toJson(value.unit),
        };
    }

    static fromJson(json: unknown): Shelf {
        const obj = $json.readObject(json, "Shelf");
        return new Shelf(
            Kit.Set.fromJson($json.field(obj, "set")),
            $json.readSeq($json.field(obj, "ids"), $json.readU32),
            Unit.fromJson($json.field(obj, "unit")),
        );
    }

    static jsonSerialize(value: Shelf): string {
        return $json.stringify(Shelf.toJson(value));
    }

    static jsonDeserialize(text: string): Shelf {
        return Shelf.fromJson($json.parse(text));
    }
}

export class Unit {
    constructor (public value: uint32) {
    }

    static toJson(value: Unit): $json.JsonValue {
        return {
            "value": value.value,
        };
    }

    static fromJson(json: unknown): Unit {
        const obj = $json.readObject(json, "Unit");
        return new Unit(
            $json.readU32($json.field(obj, "value")),
        );
    }

    static jsonSerialize(value: Unit): string {
        return $json.stringify(Unit.toJson(value));
    }

    static jsonDeserialize(text: string): Unit {
        return Unit.fromJson($json.parse(text));
    }
}
