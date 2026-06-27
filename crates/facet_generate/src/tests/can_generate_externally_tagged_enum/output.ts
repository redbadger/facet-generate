type bool = boolean;
type float32 = number;
type Optional<T> = T | null;
type str = string;
type uint32 = number;

export type SomeEnum =
    | { kind: "A"; field1: str }
    | { kind: "B"; field1: uint32; field2: float32 }
    | { kind: "C"; field3: Optional<bool> }
    | { kind: "D"; value: uint32 }
    | { kind: "E"; value: SomeNamedStruct }
    | { kind: "F"; value: Optional<SomeNamedStruct> };

export const someEnumA = (field1: str): SomeEnum => ({ kind: "A", field1 });

export const someEnumB = (field1: uint32, field2: float32): SomeEnum => ({ kind: "B", field1, field2 });

export const someEnumC = (field3: Optional<bool>): SomeEnum => ({ kind: "C", field3 });

export const someEnumD = (value: uint32): SomeEnum => ({ kind: "D", value });

export const someEnumE = (value: SomeNamedStruct): SomeEnum => ({ kind: "E", value });

export const someEnumF = (value: Optional<SomeNamedStruct>): SomeEnum => ({ kind: "F", value });

export function matchSomeEnum<R>(value: SomeEnum, cases: {
    A: (v: Extract<SomeEnum, { kind: "A" }>) => R;
    B: (v: Extract<SomeEnum, { kind: "B" }>) => R;
    C: (v: Extract<SomeEnum, { kind: "C" }>) => R;
    D: (v: Extract<SomeEnum, { kind: "D" }>) => R;
    E: (v: Extract<SomeEnum, { kind: "E" }>) => R;
    F: (v: Extract<SomeEnum, { kind: "F" }>) => R;
}): R {
    return cases[value.kind as SomeEnum["kind"]](value as never);
}

export class SomeNamedStruct {
    constructor (public a_field: str, public another_field: uint32) {
    }
}

export type SomeResult =
    | { kind: "Ok"; value: uint32 }
    | { kind: "Error"; value: str };

export const someResultOk = (value: uint32): SomeResult => ({ kind: "Ok", value });

export const someResultError = (value: str): SomeResult => ({ kind: "Error", value });

export function matchSomeResult<R>(value: SomeResult, cases: {
    Ok: (v: Extract<SomeResult, { kind: "Ok" }>) => R;
    Error: (v: Extract<SomeResult, { kind: "Error" }>) => R;
}): R {
    return cases[value.kind as SomeResult["kind"]](value as never);
}
