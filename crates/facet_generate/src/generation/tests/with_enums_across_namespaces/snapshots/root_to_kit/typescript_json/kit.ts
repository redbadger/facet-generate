import * as $json from "./serde/json";
type float64 = number;

export class Badge {
    constructor (public presence: Presence, public shape: Shape) {
    }

    static toJson(value: Badge): $json.JsonValue {
        return {
            "presence": toJsonPresence(value.presence),
            "shape": toJsonShape(value.shape),
        };
    }

    static fromJson(json: unknown): Badge {
        const obj = $json.readObject(json, "Badge");
        return new Badge(
            fromJsonPresence($json.field(obj, "presence")),
            fromJsonShape($json.field(obj, "shape")),
        );
    }

    static jsonSerialize(value: Badge): string {
        return $json.stringify(Badge.toJson(value));
    }

    static jsonDeserialize(text: string): Badge {
        return Badge.fromJson($json.parse(text));
    }
}

export type Presence =
    | { kind: "Online" }
    | { kind: "Offline" };

export const presenceOnline = (): Presence => ({ kind: "Online" });

export const presenceOffline = (): Presence => ({ kind: "Offline" });

export function matchPresence<R>(value: Presence, cases: {
    Online: (v: Extract<Presence, { kind: "Online" }>) => R;
    Offline: (v: Extract<Presence, { kind: "Offline" }>) => R;
}): R {
    return cases[value.kind as Presence["kind"]](value as never);
}

export function toJsonPresence(value: Presence): $json.JsonValue {
    switch (value.kind) {
        case "Online": return "Online";
        case "Offline": return "Offline";
        default: throw $json.unknownVariant("Presence", value);
    }
}

export function fromJsonPresence(json: unknown): Presence {
    const [variant] = $json.readExternal(json, "Presence", ["Online", "Offline"]);
    switch (variant) {
        case "Online": return { kind: "Online" };
        case "Offline": return { kind: "Offline" };
        default: throw $json.unknownVariant("Presence", variant);
    }
}

export function jsonSerializePresence(value: Presence): string {
    return $json.stringify(toJsonPresence(value));
}

export function jsonDeserializePresence(text: string): Presence {
    return fromJsonPresence($json.parse(text));
}

export type Shape =
    | { kind: "Circle"; value: float64 }
    | { kind: "Empty" };

export const shapeCircle = (value: float64): Shape => ({ kind: "Circle", value });

export const shapeEmpty = (): Shape => ({ kind: "Empty" });

export function matchShape<R>(value: Shape, cases: {
    Circle: (v: Extract<Shape, { kind: "Circle" }>) => R;
    Empty: (v: Extract<Shape, { kind: "Empty" }>) => R;
}): R {
    return cases[value.kind as Shape["kind"]](value as never);
}

export function toJsonShape(value: Shape): $json.JsonValue {
    switch (value.kind) {
        case "Circle": return { "Circle": $json.writeFloat(value.value) };
        case "Empty": return "Empty";
        default: throw $json.unknownVariant("Shape", value);
    }
}

export function fromJsonShape(json: unknown): Shape {
    const [variant, content] = $json.readExternal(json, "Shape", ["Empty"]);
    switch (variant) {
        case "Circle": return { kind: "Circle", value: $json.readF64(content) };
        case "Empty": return { kind: "Empty" };
        default: throw $json.unknownVariant("Shape", variant);
    }
}

export function jsonSerializeShape(value: Shape): string {
    return $json.stringify(toJsonShape(value));
}

export function jsonDeserializeShape(text: string): Shape {
    return fromJsonShape($json.parse(text));
}
