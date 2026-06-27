type Seq<T> = T[];
type str = string;
type uint32 = number;

export class SomeStruct {
    constructor (public field_a: uint32, public field_b: Seq<str>) {
    }
}
