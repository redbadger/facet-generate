import { Serializer, Deserializer } from "./serde";
import * as Other from "other";

export class Child {
    constructor (public external: Other.OtherParent) {
    }

    public serialize(serializer: Serializer): void {
        this.external.serialize(serializer);
    }

    static deserialize(deserializer: Deserializer): Child {
        const external = Other.OtherParent.deserialize(deserializer);
        return new Child(external);
    }
}

export type Parent =
    | { kind: "Child"; value: Child };

export const parentChild = (value: Child): Parent => ({ kind: "Child", value });

export function matchParent<R>(value: Parent, cases: {
    Child: (v: Extract<Parent, { kind: "Child" }>) => R;
}): R {
    return cases[value.kind as Parent["kind"]](value as never);
}

export function serializeParent(value: Parent, serializer: Serializer): void {
    switch (value.kind) {
        case "Child": {
            serializer.serializeVariantIndex(0);
            value.value.serialize(serializer);
            break;
        }
        default: throw new Error("Unknown variant: " + (value as any).kind);
    }
}

export function deserializeParent(deserializer: Deserializer): Parent {
    const index = deserializer.deserializeVariantIndex();
    switch (index) {
        case 0: {
            const value = Child.deserialize(deserializer);
            return { kind: "Child", value };
        }
        default: throw new Error("Unknown variant index for Parent: " + index);
    }
}
