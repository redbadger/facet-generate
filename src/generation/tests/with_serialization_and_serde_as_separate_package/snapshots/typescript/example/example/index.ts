import { Serializer, Deserializer } from '../serde';
export class Child {

constructor (public name: str) {
}

public serialize(serializer: Serializer): void {
  serializer.serializeStr(this.name);
}

static deserialize(deserializer: Deserializer): Child {
  const name = deserializer.deserializeStr();
  return new Child(name);
}

}
export abstract class Parent {
abstract serialize(serializer: Serializer): void;

static deserialize(deserializer: Deserializer): Parent {
  const index = deserializer.deserializeVariantIndex();
  switch (index) {
    case 0: return ParentVariantChild.load(deserializer);
    default: throw new Error("Unknown variant index for Parent: " + index);
  }
}
}


export class ParentVariantChild extends Parent {

constructor (public value: Child) {
  super();
}

public serialize(serializer: Serializer): void {
  serializer.serializeVariantIndex(0);
  this.value.serialize(serializer);
}

static load(deserializer: Deserializer): ParentVariantChild {
  const value = Child.deserialize(deserializer);
  return new ParentVariantChild(value);
}

}
type str = string;
export class Helpers {
}

