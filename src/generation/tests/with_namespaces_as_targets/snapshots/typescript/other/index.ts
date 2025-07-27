export class OtherChild {

constructor (public name: str) {
}

}
export abstract class OtherParent {
}


export class OtherParentVariantChild extends OtherParent {

constructor (public value: Other.OtherChild) {
  super();
}

}
