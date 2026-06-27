import { Serializer, Deserializer } from "serde";
type str = string;

export class Child {
    constructor (public name: str) {
    }

    public serialize(serializer: Serializer): void {
        serializer.serializeStr(this.name);
    }

    static deserialize(deserializer: Deserializer): Child {
        const name = deserializer.deserializeStr();
        return new Child(name);
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
