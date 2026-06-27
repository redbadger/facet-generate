import { Serializer, Deserializer } from "./serde";
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
