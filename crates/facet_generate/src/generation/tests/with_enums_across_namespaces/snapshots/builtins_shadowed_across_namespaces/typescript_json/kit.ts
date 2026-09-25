import * as $json from "./serde/json";
type Seq<T> = T[];
type uint32 = number;
type unit = null;

export class Set {
    constructor (public value: uint32) {
    }

    static toJson(value: Set): $json.JsonValue {
        return {
            "value": value.value,
        };
    }

    static fromJson(json: unknown): Set {
        const obj = $json.readObject(json, "Set");
        return new Set(
            $json.readU32($json.field(obj, "value")),
        );
    }

    static jsonSerialize(value: Set): string {
        return $json.stringify(Set.toJson(value));
    }

    static jsonDeserialize(text: string): Set {
        return Set.fromJson($json.parse(text));
    }
}

export class Tray {
    constructor (public nothing: unit, public ids: Seq<uint32>) {
    }

    static toJson(value: Tray): $json.JsonValue {
        return {
            "nothing": null,
            "ids": value.ids,
        };
    }

    static fromJson(json: unknown): Tray {
        const obj = $json.readObject(json, "Tray");
        return new Tray(
            $json.readUnit($json.field(obj, "nothing")),
            $json.readSeq($json.field(obj, "ids"), $json.readU32),
        );
    }

    static jsonSerialize(value: Tray): string {
        return $json.stringify(Tray.toJson(value));
    }

    static jsonDeserialize(text: string): Tray {
        return Tray.fromJson($json.parse(text));
    }
}
