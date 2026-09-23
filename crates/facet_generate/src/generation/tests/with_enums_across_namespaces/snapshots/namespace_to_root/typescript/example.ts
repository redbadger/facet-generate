import { Serializer, Deserializer } from "./serde";
import * as Kv from "./kv";
type uint32 = number;

export class App {
    constructor (public entry: Kv.Entry) {
    }

    public serialize(serializer: Serializer): void {
        this.entry.serialize(serializer);
    }

    static deserialize(deserializer: Deserializer): App {
        const entry = Kv.Entry.deserialize(deserializer);
        return new App(entry);
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

export function serializeLevel(value: Level, serializer: Serializer): void {
    switch (value.kind) {
        case "Low": {
            serializer.serializeVariantIndex(0);
            break;
        }
        case "High": {
            serializer.serializeVariantIndex(1);
            break;
        }
        default: throw new Error("Unknown variant: " + (value as any).kind);
    }
}

export function deserializeLevel(deserializer: Deserializer): Level {
    const index = deserializer.deserializeVariantIndex();
    switch (index) {
        case 0: {
            return { kind: "Low" };
        }
        case 1: {
            return { kind: "High" };
        }
        default: throw new Error("Unknown variant index for Level: " + index);
    }
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

export function serializeOutcome(value: Outcome, serializer: Serializer): void {
    switch (value.kind) {
        case "Score": {
            serializer.serializeVariantIndex(0);
            serializer.serializeU32(value.value);
            break;
        }
        case "Missing": {
            serializer.serializeVariantIndex(1);
            break;
        }
        default: throw new Error("Unknown variant: " + (value as any).kind);
    }
}

export function deserializeOutcome(deserializer: Deserializer): Outcome {
    const index = deserializer.deserializeVariantIndex();
    switch (index) {
        case 0: {
            const value = deserializer.deserializeU32();
            return { kind: "Score", value };
        }
        case 1: {
            return { kind: "Missing" };
        }
        default: throw new Error("Unknown variant index for Outcome: " + index);
    }
}
