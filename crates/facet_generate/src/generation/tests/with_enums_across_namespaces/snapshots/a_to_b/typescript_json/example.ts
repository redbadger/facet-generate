import * as $json from "./serde/json";
import * as A from "./a";

export class App {
    constructor (public row: A.Row) {
    }

    static toJson(value: App): $json.JsonValue {
        return {
            "row": A.Row.toJson(value.row),
        };
    }

    static fromJson(json: unknown): App {
        const obj = $json.readObject(json, "App");
        return new App(
            A.Row.fromJson($json.field(obj, "row")),
        );
    }

    static jsonSerialize(value: App): string {
        return $json.stringify(App.toJson(value));
    }

    static jsonDeserialize(text: string): App {
        return App.fromJson($json.parse(text));
    }
}
