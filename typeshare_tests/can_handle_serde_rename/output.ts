export interface OtherType {
}

/** This is a comment. */
export interface Person {
    name: string;
    age: number;
    extraSpecialFieldOne: number;
    extraSpecialFieldTwo?: string[] | null;
    nonStandardDataType: OtherType;
    nonStandardDataTypeInArray?: OtherType[] | null;
}

