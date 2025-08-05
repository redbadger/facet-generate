export interface OverrideStruct {
    fieldToOverride: any | undefined;
}

export type OverrideEnum = 
    | { type: "UnitVariant" }
    | { type: "TupleVariant", content: string }
    | { type: "AnonymousStructVariant", content: {
    fieldToOverride: any | undefined;
}};

