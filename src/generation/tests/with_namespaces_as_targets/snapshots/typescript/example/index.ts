import * as Other from './other';

export class Child {

constructor (public external: Other.OtherParent) {
}

}
export abstract class Parent {
}


export class ParentVariantChild extends Parent {

constructor (public value: Child) {
  super();
}

}

