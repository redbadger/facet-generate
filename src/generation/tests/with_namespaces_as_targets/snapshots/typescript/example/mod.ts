
import { Serializer, Deserializer } from '../serde/mod.ts';
import { BcsSerializer, BcsDeserializer } from '../bcs/mod.ts';
import { Optional, Seq, Tuple, ListTuple, unit, bool, int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, char, str, bytes } from '../serde/mod.ts';

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
