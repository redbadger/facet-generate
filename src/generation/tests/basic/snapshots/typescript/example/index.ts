export class Child {

constructor (public name: str) {
}

}
export abstract class Parent {
}


export class ParentVariantChild extends Parent {

constructor (public value: Child) {
  super();
}

}
type str = string;
