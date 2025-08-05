/** Struct comment */
export interface ExplicitlyNamedStruct {
    /** Field comment */
    a: number;
    b: number;
}

/** Enum comment */
export type AdvancedColors = 
    | { type: "Unit" }
    /** This is a case comment */
    | { type: "Str", content: string }
    | { type: "Number", content: number }
    | { type: "UnsignedNumber", content: number }
    | { type: "NumberArray", content: number[] }
    | { type: "TestWithAnonymousStruct", content: {
    a: number;
    b: number;
}}
    /** Comment on the last element */
    | { type: "TestWithExplicitlyNamedStruct", content: ExplicitlyNamedStruct };

export type AdvancedColors2 = 
    /** This is a case comment */
    | { type: "str", content: string }
    | { type: "number", content: number }
    | { type: "number-array", content: number[] }
    /** Comment on the last element */
    | { type: "really-cool-type", content: ExplicitlyNamedStruct };

