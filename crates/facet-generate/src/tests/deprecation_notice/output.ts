/** @deprecated Use `MySuperAwesomeAlias` instead */
export type MyLegacyAlias = number;

/** @deprecated Use `MySuperAwesomeStruct` instead */
export interface MyLegacyStruct {
    field: string;
}

/** @deprecated Use `MySuperAwesomeEnum` instead */
export enum MyLegacyEnum {
    VariantA = "VariantA",
    VariantB = "VariantB",
    VariantC = "VariantC",
}

export enum MyUnitEnum {
    VariantA = "VariantA",
    VariantB = "VariantB",
    /** @deprecated Use `VariantB` instead */
    LegacyVariant = "LegacyVariant",
}

export type MyInternallyTaggedEnum = 
    | { type: "VariantA",
    field: string;
}
    | { type: "VariantB",
    field: number;
}
    | /** @deprecated Use `VariantA` instead */ { type: "LegacyVariant",
    field: boolean;
};

export type MyExternallyTaggedEnum = 
    | { VariantA: string }
    | { VariantB: number }
    | /** @deprecated Use `VariantB` instead */ { LegacyVariant: boolean };

