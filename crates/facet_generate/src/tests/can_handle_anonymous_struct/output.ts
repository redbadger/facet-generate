type int32 = number;
type str = string;

/// Enum keeping track of who autofilled a field
export type AutofilledBy =
    | { type: "Us"; content: { uuid: str; } }
    | { type: "SomethingElse"; content: { uuid: str; thing: int32; } };

export const autofilledByUs = (uuid: str): AutofilledBy => ({ type: "Us", content: { uuid } });

export const autofilledBySomethingElse = (uuid: str, thing: int32): AutofilledBy => ({ type: "SomethingElse", content: { uuid, thing } });

export function matchAutofilledBy<R>(value: AutofilledBy, cases: {
    Us: (v: Extract<AutofilledBy, { type: "Us" }>) => R;
    SomethingElse: (v: Extract<AutofilledBy, { type: "SomethingElse" }>) => R;
}): R {
    return cases[value.type as AutofilledBy["type"]](value as never);
}

/// This is a comment (yareek sameek wuz here)
export type EnumWithManyVariants =
    | { type: "UnitVariant" }
    | { type: "TupleVariantString"; content: str }
    | { type: "AnonVariant"; content: { uuid: str; } }
    | { type: "TupleVariantInt"; content: int32 }
    | { type: "AnotherUnitVariant" }
    | { type: "AnotherAnonVariant"; content: { uuid: str; thing: int32; } };

export const enumWithManyVariantsUnitVariant = (): EnumWithManyVariants => ({ type: "UnitVariant" });

export const enumWithManyVariantsTupleVariantString = (value: str): EnumWithManyVariants => ({ type: "TupleVariantString", content: value });

export const enumWithManyVariantsAnonVariant = (uuid: str): EnumWithManyVariants => ({ type: "AnonVariant", content: { uuid } });

export const enumWithManyVariantsTupleVariantInt = (value: int32): EnumWithManyVariants => ({ type: "TupleVariantInt", content: value });

export const enumWithManyVariantsAnotherUnitVariant = (): EnumWithManyVariants => ({ type: "AnotherUnitVariant" });

export const enumWithManyVariantsAnotherAnonVariant = (uuid: str, thing: int32): EnumWithManyVariants => ({ type: "AnotherAnonVariant", content: { uuid, thing } });

export function matchEnumWithManyVariants<R>(value: EnumWithManyVariants, cases: {
    UnitVariant: (v: Extract<EnumWithManyVariants, { type: "UnitVariant" }>) => R;
    TupleVariantString: (v: Extract<EnumWithManyVariants, { type: "TupleVariantString" }>) => R;
    AnonVariant: (v: Extract<EnumWithManyVariants, { type: "AnonVariant" }>) => R;
    TupleVariantInt: (v: Extract<EnumWithManyVariants, { type: "TupleVariantInt" }>) => R;
    AnotherUnitVariant: (v: Extract<EnumWithManyVariants, { type: "AnotherUnitVariant" }>) => R;
    AnotherAnonVariant: (v: Extract<EnumWithManyVariants, { type: "AnotherAnonVariant" }>) => R;
}): R {
    return cases[value.type as EnumWithManyVariants["type"]](value as never);
}
