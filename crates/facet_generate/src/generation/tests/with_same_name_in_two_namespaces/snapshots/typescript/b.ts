import * as A from "./a";
type uint8 = number;

export class Child {
    constructor (public y: uint8) {
    }
}

export class Parent {
    constructor (public first: A.Child, public second: Child) {
    }
}
