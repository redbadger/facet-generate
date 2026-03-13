export type AdvancedEnum = 
    | { type: "unitVariant" }
    | { type: "A",
    field1: string;
}
    | { type: "otherAnonymousStruct",
    field1: number;
    field2: number;
}
    | { type: "B",
    field3?: boolean | null;
};

