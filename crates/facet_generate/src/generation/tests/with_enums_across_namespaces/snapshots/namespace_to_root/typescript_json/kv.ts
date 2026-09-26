import * as $json from "./serde/json";
import * as Example from "./example";

export class Entry {
    constructor (public level: Example.Level, public outcome: Example.Outcome) {
    }

    static toJson(value: Entry): $json.JsonValue {
        return {
            "level": Example.toJsonLevel(value.level),
            "outcome": Example.toJsonOutcome(value.outcome),
        };
    }

    static fromJson(json: unknown): Entry {
        const obj = $json.readObject(json, "Entry");
        return new Entry(
            Example.fromJsonLevel($json.field(obj, "level")),
            Example.fromJsonOutcome($json.field(obj, "outcome")),
        );
    }

    static jsonSerialize(value: Entry): string {
        return $json.stringify(Entry.toJson(value));
    }

    static jsonDeserialize(text: string): Entry {
        return Entry.fromJson($json.parse(text));
    }
}
