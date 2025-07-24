export interface NamedEmptyStruct {
}

export type Test = 
    | { type: "NamedEmptyStruct", content: NamedEmptyStruct }
    | { type: "AnonymousEmptyStruct", content: {
}}
    | { type: "NoStruct" };

