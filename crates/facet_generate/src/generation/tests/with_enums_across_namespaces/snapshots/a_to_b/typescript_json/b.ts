import * as $json from "./serde/json";
type uint8 = number;

export type Signal =
    | { kind: "Level"; value: uint8 }
    | { kind: "Silent" };

export const signalLevel = (value: uint8): Signal => ({ kind: "Level", value });

export const signalSilent = (): Signal => ({ kind: "Silent" });

export function matchSignal<R>(value: Signal, cases: {
    Level: (v: Extract<Signal, { kind: "Level" }>) => R;
    Silent: (v: Extract<Signal, { kind: "Silent" }>) => R;
}): R {
    return cases[value.kind as Signal["kind"]](value as never);
}

export function toJsonSignal(value: Signal): $json.JsonValue {
    switch (value.kind) {
        case "Level": return { "Level": value.value };
        case "Silent": return "Silent";
        default: throw $json.unknownVariant("Signal", value);
    }
}

export function fromJsonSignal(json: unknown): Signal {
    const [variant, content] = $json.readExternal(json, "Signal", ["Silent"]);
    switch (variant) {
        case "Level": return { kind: "Level", value: $json.readU8(content) };
        case "Silent": return { kind: "Silent" };
        default: throw $json.unknownVariant("Signal", variant);
    }
}

export function jsonSerializeSignal(value: Signal): string {
    return $json.stringify(toJsonSignal(value));
}

export function jsonDeserializeSignal(text: string): Signal {
    return fromJsonSignal($json.parse(text));
}

export type Status =
    | { kind: "Up" }
    | { kind: "Down" };

export const statusUp = (): Status => ({ kind: "Up" });

export const statusDown = (): Status => ({ kind: "Down" });

export function matchStatus<R>(value: Status, cases: {
    Up: (v: Extract<Status, { kind: "Up" }>) => R;
    Down: (v: Extract<Status, { kind: "Down" }>) => R;
}): R {
    return cases[value.kind as Status["kind"]](value as never);
}

export function toJsonStatus(value: Status): $json.JsonValue {
    switch (value.kind) {
        case "Up": return "Up";
        case "Down": return "Down";
        default: throw $json.unknownVariant("Status", value);
    }
}

export function fromJsonStatus(json: unknown): Status {
    const [variant] = $json.readExternal(json, "Status", ["Up", "Down"]);
    switch (variant) {
        case "Up": return { kind: "Up" };
        case "Down": return { kind: "Down" };
        default: throw $json.unknownVariant("Status", variant);
    }
}

export function jsonSerializeStatus(value: Status): string {
    return $json.stringify(toJsonStatus(value));
}

export function jsonDeserializeStatus(text: string): Status {
    return fromJsonStatus($json.parse(text));
}
