
import { Serializer, Deserializer } from '../serde/mod.ts';
import { BcsSerializer, BcsDeserializer } from '../bcs/mod.ts';
import { Optional, Seq, Tuple, ListTuple, unit, bool, int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, char, str, bytes } from '../serde/mod.ts';

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
