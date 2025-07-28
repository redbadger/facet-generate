import * as Other from "../other";
type str = string;
export class OtherChild {

  constructor (public name: str) {
  }

}
export abstract class OtherParent {
}


export class OtherParentVariantChild extends OtherParent {

  constructor (public value: OtherChild) {
    super();
  }

}
