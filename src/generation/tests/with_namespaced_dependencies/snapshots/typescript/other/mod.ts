
import { Serializer, Deserializer } from '../serde/mod.ts';
import { BcsSerializer, BcsDeserializer } from '../bcs/mod.ts';
import { Optional, Seq, Tuple, ListTuple, unit, bool, int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, char, str, bytes } from '../serde/mod.ts';

export class OtherChild {

constructor (public name: str) {
}

public serialize(serializer: Serializer): void {
  serializer.serializeStr(this.name);
}

static deserialize(deserializer: Deserializer): OtherChild {
  const name = deserializer.deserializeStr();
  return new OtherChild(name);
}

}
export abstract class OtherParent {
abstract serialize(serializer: Serializer): void;

static deserialize(deserializer: Deserializer): OtherParent {
  const index = deserializer.deserializeVariantIndex();
  switch (index) {
    case 0: return OtherParentVariantChild.load(deserializer);
    default: throw new Error("Unknown variant index for OtherParent: " + index);
  }
}
}


export class OtherParentVariantChild extends OtherParent {

constructor (public value: Other.OtherChild) {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(0);
  this.value.serialize(serializer);
}

static load(deserializer: Deserializer): OtherParentVariantChild {
  const value = Other.OtherChild.deserialize(deserializer);
  return new OtherParentVariantChild(value);
}

}
export class Helpers {
}

