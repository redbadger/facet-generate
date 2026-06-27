type bool = boolean;
type float32 = number;
type Optional<T> = T | null;
type str = string;
type uint32 = number;

export class ExplicitlyNamedStruct {
    constructor (public a_field: str, public another_field: uint32) {
    }
}

export type SomeEnum =
    | { type: "A" }
    | { type: "B"; field1: str }
    | { type: "C"; field1: uint32; field2: float32 }
    | { type: "D"; field3: Optional<bool> }
    | { type: "E" } & ExplicitlyNamedStruct;

export const someEnumA = (): SomeEnum => ({ type: "A" });

export const someEnumB = (field1: str): SomeEnum => ({ type: "B", field1 });

export const someEnumC = (field1: uint32, field2: float32): SomeEnum => ({ type: "C", field1, field2 });

export const someEnumD = (field3: Optional<bool>): SomeEnum => ({ type: "D", field3 });

export const someEnumE = (value: ExplicitlyNamedStruct): SomeEnum => ({ type: "E", ...value });

export function matchSomeEnum<R>(value: SomeEnum, cases: {
    A: (v: Extract<SomeEnum, { type: "A" }>) => R;
    B: (v: Extract<SomeEnum, { type: "B" }>) => R;
    C: (v: Extract<SomeEnum, { type: "C" }>) => R;
    D: (v: Extract<SomeEnum, { type: "D" }>) => R;
    E: (v: Extract<SomeEnum, { type: "E" }>) => R;
}): R {
    return cases[value.type as SomeEnum["type"]](value as never);
}
