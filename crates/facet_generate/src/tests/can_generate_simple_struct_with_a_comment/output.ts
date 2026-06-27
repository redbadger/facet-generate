type Optional<T> = T | null;
type Seq<T> = T[];
type str = string;
type uint8 = number;

export class Location {
    constructor () {
    }
}

/// This is a comment.
export class Person {
    constructor (public name: str, public age: uint8, public info: Optional<str>, public emails: Seq<str>, public location: Location) {
    }
}
