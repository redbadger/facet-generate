export interface ExplicitlyNamedStruct {
    a_field: string;
    another_field: number;
}

export type SomeEnum = 
    | { type: "A" }
    | { type: "B",
    field1: string;
}
    | { type: "C",
    field1: number;
    field2: number;
}
    | { type: "D",
    field3?: boolean | null;
}
    | { type: "E" } & (ExplicitlyNamedStruct);

