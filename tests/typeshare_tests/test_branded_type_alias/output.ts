export type SimpleAlias1 = string;

export type SimpleAlias2 = string;

export type BrandedStringAlias = (string & { __brand: "BrandedStringAlias" });

export type BrandedOptionalStringAlias = (string & { __brand: "BrandedOptionalStringAlias" }) | undefined | null;

export type BrandedU32Alias = (number & { __brand: "BrandedU32Alias" });

export interface MyStruct {
    field: number;
    other_field: string;
}

export type BrandedStructAlias = (MyStruct & { __brand: "BrandedStructAlias" });

