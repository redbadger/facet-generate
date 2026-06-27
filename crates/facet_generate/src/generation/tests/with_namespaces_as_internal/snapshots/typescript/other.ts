type str = string;

export class OtherChild {
    constructor (public name: str) {
    }
}

export type OtherParent =
    | { kind: "Child"; value: OtherChild };

export const otherParentChild = (value: OtherChild): OtherParent => ({ kind: "Child", value });

export function matchOtherParent<R>(value: OtherParent, cases: {
    Child: (v: Extract<OtherParent, { kind: "Child" }>) => R;
}): R {
    return cases[value.kind as OtherParent["kind"]](value as never);
}
