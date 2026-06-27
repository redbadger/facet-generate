type int32 = number;
type Seq<T> = T[];
type str = string;
type uint32 = number;

/// Enum comment
export type AdvancedColors =
    | { type: "Unit" }
    | { type: "Str"; content: str }
    | { type: "Number"; content: int32 }
    | { type: "UnsignedNumber"; content: uint32 }
    | { type: "NumberArray"; content: Seq<int32> }
    | { type: "TestWithAnonymousStruct"; content: { a: uint32; b: uint32; } }
    | { type: "TestWithExplicitlyNamedStruct"; content: ExplicitlyNamedStruct };

export const advancedColorsUnit = (): AdvancedColors => ({ type: "Unit" });

export const advancedColorsStr = (value: str): AdvancedColors => ({ type: "Str", content: value });

export const advancedColorsNumber = (value: int32): AdvancedColors => ({ type: "Number", content: value });

export const advancedColorsUnsignedNumber = (value: uint32): AdvancedColors => ({ type: "UnsignedNumber", content: value });

export const advancedColorsNumberArray = (value: Seq<int32>): AdvancedColors => ({ type: "NumberArray", content: value });

export const advancedColorsTestWithAnonymousStruct = (a: uint32, b: uint32): AdvancedColors => ({ type: "TestWithAnonymousStruct", content: { a, b } });

export const advancedColorsTestWithExplicitlyNamedStruct = (value: ExplicitlyNamedStruct): AdvancedColors => ({ type: "TestWithExplicitlyNamedStruct", content: value });

export function matchAdvancedColors<R>(value: AdvancedColors, cases: {
    Unit: (v: Extract<AdvancedColors, { type: "Unit" }>) => R;
    Str: (v: Extract<AdvancedColors, { type: "Str" }>) => R;
    Number: (v: Extract<AdvancedColors, { type: "Number" }>) => R;
    UnsignedNumber: (v: Extract<AdvancedColors, { type: "UnsignedNumber" }>) => R;
    NumberArray: (v: Extract<AdvancedColors, { type: "NumberArray" }>) => R;
    TestWithAnonymousStruct: (v: Extract<AdvancedColors, { type: "TestWithAnonymousStruct" }>) => R;
    TestWithExplicitlyNamedStruct: (v: Extract<AdvancedColors, { type: "TestWithExplicitlyNamedStruct" }>) => R;
}): R {
    return cases[value.type as AdvancedColors["type"]](value as never);
}

export type AdvancedColors2 =
    | { type: "str"; content: str }
    | { type: "number"; content: int32 }
    | { type: "number-array"; content: Seq<int32> }
    | { type: "really-cool-type"; content: ExplicitlyNamedStruct };

export const advancedColors2Str = (value: str): AdvancedColors2 => ({ type: "str", content: value });

export const advancedColors2Number = (value: int32): AdvancedColors2 => ({ type: "number", content: value });

export const advancedColors2NumberArray = (value: Seq<int32>): AdvancedColors2 => ({ type: "number-array", content: value });

export const advancedColors2ReallyCoolType = (value: ExplicitlyNamedStruct): AdvancedColors2 => ({ type: "really-cool-type", content: value });

export function matchAdvancedColors2<R>(value: AdvancedColors2, cases: {
    str: (v: Extract<AdvancedColors2, { type: "str" }>) => R;
    number: (v: Extract<AdvancedColors2, { type: "number" }>) => R;
    number-array: (v: Extract<AdvancedColors2, { type: "number-array" }>) => R;
    really-cool-type: (v: Extract<AdvancedColors2, { type: "really-cool-type" }>) => R;
}): R {
    return cases[value.type as AdvancedColors2["type"]](value as never);
}

/// Struct comment
export class ExplicitlyNamedStruct {
    constructor (public a: uint32, public b: uint32) {
    }
}
