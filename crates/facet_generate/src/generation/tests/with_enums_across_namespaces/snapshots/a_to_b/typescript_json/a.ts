import * as $json from "./serde/json";
import * as B from "./b";

export class Row {
    constructor (public status: B.Status, public signal: B.Signal) {
    }

    static toJson(value: Row): $json.JsonValue {
        return {
            "status": B.toJsonStatus(value.status),
            "signal": B.toJsonSignal(value.signal),
        };
    }

    static fromJson(json: unknown): Row {
        const obj = $json.readObject(json, "Row");
        return new Row(
            B.fromJsonStatus($json.field(obj, "status")),
            B.fromJsonSignal($json.field(obj, "signal")),
        );
    }

    static jsonSerialize(value: Row): string {
        return $json.stringify(Row.toJson(value));
    }

    static jsonDeserialize(text: string): Row {
        return Row.fromJson($json.parse(text));
    }
}
