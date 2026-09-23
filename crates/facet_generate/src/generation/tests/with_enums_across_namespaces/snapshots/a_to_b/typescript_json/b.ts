import type { Serializer, Deserializer } from "./serde";
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

export function serializeSignal(value: Signal, serializer: Serializer): void {
    switch (value.kind) {
        case "Level": {
            serializer.serializeVariantIndex(0);
            serializer.serializeU8(value.value);
            break;
        }
        case "Silent": {
            serializer.serializeVariantIndex(1);
            break;
        }
        default: throw new Error("Unknown variant: " + (value as any).kind);
    }
}

export function deserializeSignal(deserializer: Deserializer): Signal {
    const index = deserializer.deserializeVariantIndex();
    switch (index) {
        case 0: {
            const value = deserializer.deserializeU8();
            return { kind: "Level", value };
        }
        case 1: {
            return { kind: "Silent" };
        }
        default: throw new Error("Unknown variant index for Signal: " + index);
    }
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

export function serializeStatus(value: Status, serializer: Serializer): void {
    switch (value.kind) {
        case "Up": {
            serializer.serializeVariantIndex(0);
            break;
        }
        case "Down": {
            serializer.serializeVariantIndex(1);
            break;
        }
        default: throw new Error("Unknown variant: " + (value as any).kind);
    }
}

export function deserializeStatus(deserializer: Deserializer): Status {
    const index = deserializer.deserializeVariantIndex();
    switch (index) {
        case 0: {
            return { kind: "Up" };
        }
        case 1: {
            return { kind: "Down" };
        }
        default: throw new Error("Unknown variant index for Status: " + index);
    }
}
