import { Serializer, Deserializer } from "./serde";
type float64 = number;

export class Badge {
    constructor (public presence: Presence, public shape: Shape) {
    }

    public serialize(serializer: Serializer): void {
        serializePresence(this.presence, serializer);
        serializeShape(this.shape, serializer);
    }

    static deserialize(deserializer: Deserializer): Badge {
        const presence = deserializePresence(deserializer);
        const shape = deserializeShape(deserializer);
        return new Badge(presence,shape);
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

export function serializePresence(value: Presence, serializer: Serializer): void {
    switch (value.kind) {
        case "Online": {
            serializer.serializeVariantIndex(0);
            break;
        }
        case "Offline": {
            serializer.serializeVariantIndex(1);
            break;
        }
        default: throw new Error("Unknown variant: " + (value as any).kind);
    }
}

export function deserializePresence(deserializer: Deserializer): Presence {
    const index = deserializer.deserializeVariantIndex();
    switch (index) {
        case 0: {
            return { kind: "Online" };
        }
        case 1: {
            return { kind: "Offline" };
        }
        default: throw new Error("Unknown variant index for Presence: " + index);
    }
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

export function serializeShape(value: Shape, serializer: Serializer): void {
    switch (value.kind) {
        case "Circle": {
            serializer.serializeVariantIndex(0);
            serializer.serializeF64(value.value);
            break;
        }
        case "Empty": {
            serializer.serializeVariantIndex(1);
            break;
        }
        default: throw new Error("Unknown variant: " + (value as any).kind);
    }
}

export function deserializeShape(deserializer: Deserializer): Shape {
    const index = deserializer.deserializeVariantIndex();
    switch (index) {
        case 0: {
            const value = deserializer.deserializeF64();
            return { kind: "Circle", value };
        }
        case 1: {
            return { kind: "Empty" };
        }
        default: throw new Error("Unknown variant index for Shape: " + index);
    }
}
