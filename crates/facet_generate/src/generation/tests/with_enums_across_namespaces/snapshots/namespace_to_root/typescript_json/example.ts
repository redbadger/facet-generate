import * as $json from "./serde/json";
import * as Kv from "./kv";
type uint32 = number;

export class App {
    constructor (public entry: Kv.Entry) {
    }

    static toJson(value: App): $json.JsonValue {
        return {
            "entry": Kv.Entry.toJson(value.entry),
        };
    }

    static fromJson(json: unknown): App {
        const obj = $json.readObject(json, "App");
        return new App(
            Kv.Entry.fromJson($json.field(obj, "entry")),
        );
    }

    static jsonSerialize(value: App): string {
        return $json.stringify(App.toJson(value));
    }

    static jsonDeserialize(text: string): App {
        return App.fromJson($json.parse(text));
    }
}

export type Level =
    | { kind: "Low" }
    | { kind: "High" };

export const levelLow = (): Level => ({ kind: "Low" });

export const levelHigh = (): Level => ({ kind: "High" });

export function matchLevel<R>(value: Level, cases: {
    Low: (v: Extract<Level, { kind: "Low" }>) => R;
    High: (v: Extract<Level, { kind: "High" }>) => R;
}): R {
    return cases[value.kind as Level["kind"]](value as never);
}

export function toJsonLevel(value: Level): $json.JsonValue {
    switch (value.kind) {
        case "Low": return "Low";
        case "High": return "High";
        default: throw $json.unknownVariant("Level", value);
    }
}

export function fromJsonLevel(json: unknown): Level {
    const [variant] = $json.readExternal(json, "Level", ["Low", "High"]);
    switch (variant) {
        case "Low": return { kind: "Low" };
        case "High": return { kind: "High" };
        default: throw $json.unknownVariant("Level", variant);
    }
}

export function jsonSerializeLevel(value: Level): string {
    return $json.stringify(toJsonLevel(value));
}

export function jsonDeserializeLevel(text: string): Level {
    return fromJsonLevel($json.parse(text));
}

export type Outcome =
    | { kind: "Score"; value: uint32 }
    | { kind: "Missing" };

export const outcomeScore = (value: uint32): Outcome => ({ kind: "Score", value });

export const outcomeMissing = (): Outcome => ({ kind: "Missing" });

export function matchOutcome<R>(value: Outcome, cases: {
    Score: (v: Extract<Outcome, { kind: "Score" }>) => R;
    Missing: (v: Extract<Outcome, { kind: "Missing" }>) => R;
}): R {
    return cases[value.kind as Outcome["kind"]](value as never);
}

export function toJsonOutcome(value: Outcome): $json.JsonValue {
    switch (value.kind) {
        case "Score": return { "Score": value.value };
        case "Missing": return "Missing";
        default: throw $json.unknownVariant("Outcome", value);
    }
}

export function fromJsonOutcome(json: unknown): Outcome {
    const [variant, content] = $json.readExternal(json, "Outcome", ["Missing"]);
    switch (variant) {
        case "Score": return { kind: "Score", value: $json.readU32(content) };
        case "Missing": return { kind: "Missing" };
        default: throw $json.unknownVariant("Outcome", variant);
    }
}

export function jsonSerializeOutcome(value: Outcome): string {
    return $json.stringify(toJsonOutcome(value));
}

export function jsonDeserializeOutcome(text: string): Outcome {
    return fromJsonOutcome($json.parse(text));
}
