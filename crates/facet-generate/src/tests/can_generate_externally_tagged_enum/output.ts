export interface SomeNamedStruct {
    a_field: string;
    another_field: number;
}

export type SomeResult = 
    | { Ok: number }
    | { Error: string };

export type SomeEnum = 
    | { A: {
    field1: string;
}}
    | { B: {
    field1: number;
    field2: number;
}}
    | { C: {
    field3?: boolean | null;
}}
    | { D: number }
    | { E: SomeNamedStruct }
    | { F: SomeNamedStruct | null };

