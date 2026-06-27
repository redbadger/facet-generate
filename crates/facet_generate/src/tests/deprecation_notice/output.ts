type bool = boolean;
type str = string;
type uint32 = number;

export type MyExternallyTaggedEnum =
    | { kind: "VariantA"; value: str }
    | { kind: "VariantB"; value: uint32 }
    | { kind: "LegacyVariant"; value: bool };

export const myExternallyTaggedEnumVariantA = (value: str): MyExternallyTaggedEnum => ({ kind: "VariantA", value });

export const myExternallyTaggedEnumVariantB = (value: uint32): MyExternallyTaggedEnum => ({ kind: "VariantB", value });

export const myExternallyTaggedEnumLegacyVariant = (value: bool): MyExternallyTaggedEnum => ({ kind: "LegacyVariant", value });

export function matchMyExternallyTaggedEnum<R>(value: MyExternallyTaggedEnum, cases: {
    VariantA: (v: Extract<MyExternallyTaggedEnum, { kind: "VariantA" }>) => R;
    VariantB: (v: Extract<MyExternallyTaggedEnum, { kind: "VariantB" }>) => R;
    LegacyVariant: (v: Extract<MyExternallyTaggedEnum, { kind: "LegacyVariant" }>) => R;
}): R {
    return cases[value.kind as MyExternallyTaggedEnum["kind"]](value as never);
}

export type MyInternallyTaggedEnum =
    | { type: "VariantA"; field: str }
    | { type: "VariantB"; field: uint32 }
    | { type: "LegacyVariant"; field: bool };

export const myInternallyTaggedEnumVariantA = (field: str): MyInternallyTaggedEnum => ({ type: "VariantA", field });

export const myInternallyTaggedEnumVariantB = (field: uint32): MyInternallyTaggedEnum => ({ type: "VariantB", field });

export const myInternallyTaggedEnumLegacyVariant = (field: bool): MyInternallyTaggedEnum => ({ type: "LegacyVariant", field });

export function matchMyInternallyTaggedEnum<R>(value: MyInternallyTaggedEnum, cases: {
    VariantA: (v: Extract<MyInternallyTaggedEnum, { type: "VariantA" }>) => R;
    VariantB: (v: Extract<MyInternallyTaggedEnum, { type: "VariantB" }>) => R;
    LegacyVariant: (v: Extract<MyInternallyTaggedEnum, { type: "LegacyVariant" }>) => R;
}): R {
    return cases[value.type as MyInternallyTaggedEnum["type"]](value as never);
}

export class MyLegacyAlias {
    constructor (public value: uint32) {
    }
}

export type MyLegacyEnum =
    | { kind: "VariantA" }
    | { kind: "VariantB" }
    | { kind: "VariantC" };

export const myLegacyEnumVariantA = (): MyLegacyEnum => ({ kind: "VariantA" });

export const myLegacyEnumVariantB = (): MyLegacyEnum => ({ kind: "VariantB" });

export const myLegacyEnumVariantC = (): MyLegacyEnum => ({ kind: "VariantC" });

export function matchMyLegacyEnum<R>(value: MyLegacyEnum, cases: {
    VariantA: (v: Extract<MyLegacyEnum, { kind: "VariantA" }>) => R;
    VariantB: (v: Extract<MyLegacyEnum, { kind: "VariantB" }>) => R;
    VariantC: (v: Extract<MyLegacyEnum, { kind: "VariantC" }>) => R;
}): R {
    return cases[value.kind as MyLegacyEnum["kind"]](value as never);
}

export class MyLegacyStruct {
    constructor (public field: str) {
    }
}

export type MyUnitEnum =
    | { kind: "VariantA" }
    | { kind: "VariantB" }
    | { kind: "LegacyVariant" };

export const myUnitEnumVariantA = (): MyUnitEnum => ({ kind: "VariantA" });

export const myUnitEnumVariantB = (): MyUnitEnum => ({ kind: "VariantB" });

export const myUnitEnumLegacyVariant = (): MyUnitEnum => ({ kind: "LegacyVariant" });

export function matchMyUnitEnum<R>(value: MyUnitEnum, cases: {
    VariantA: (v: Extract<MyUnitEnum, { kind: "VariantA" }>) => R;
    VariantB: (v: Extract<MyUnitEnum, { kind: "VariantB" }>) => R;
    LegacyVariant: (v: Extract<MyUnitEnum, { kind: "LegacyVariant" }>) => R;
}): R {
    return cases[value.kind as MyUnitEnum["kind"]](value as never);
}
