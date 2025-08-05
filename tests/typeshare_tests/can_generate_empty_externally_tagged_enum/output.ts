export interface NamedEmptyStruct {
}

export type Test = 
    | { NamedEmptyStruct: NamedEmptyStruct }
    | { AnonymousEmptyStruct: {
}};

