import { Serializer, Deserializer } from "../serde";
type int32 = number;
type Optional<T> = T | null;
type Seq<T> = T[];
type str = string;
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
export class MyStruct {

  constructor (public string_to_int: Map<str,int32>, public map_to_list: Map<str,Seq<int32>>, public option_of_vec_of_set: Optional<Seq<Seq<str>>>, public parent: Parent) {
  }

  public serialize(serializer: Serializer): void {
    Helpers.serializeMapStrToI32(this.string_to_int, serializer);
    Helpers.serializeMapStrToVectorI32(this.map_to_list, serializer);
    Helpers.serializeOptionVectorSetStr(this.option_of_vec_of_set, serializer);
    this.parent.serialize(serializer);
  }

  static deserialize(deserializer: Deserializer): MyStruct {
    const string_to_int = Helpers.deserializeMapStrToI32(deserializer);
    const map_to_list = Helpers.deserializeMapStrToVectorI32(deserializer);
    const option_of_vec_of_set = Helpers.deserializeOptionVectorSetStr(deserializer);
    const parent = Parent.deserialize(deserializer);
    return new MyStruct(string_to_int,map_to_list,option_of_vec_of_set,parent);
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
export class Helpers {
  static serializeMapStrToI32(value: Map<str,int32>, serializer: Serializer): void {
    serializer.serializeLen(value.size);
    const offsets: number[] = [];
    for (const [k, v] of value.entries()) {
      offsets.push(serializer.getBufferOffset());
      serializer.serializeStr(k);
      serializer.serializeI32(v);
    }
    serializer.sortMapEntries(offsets);
  }

  static deserializeMapStrToI32(deserializer: Deserializer): Map<str,int32> {
    const length = deserializer.deserializeLen();
    const obj = new Map<str, int32>();
    let previousKeyStart = 0;
    let previousKeyEnd = 0;
    for (let i = 0; i < length; i++) {
        const keyStart = deserializer.getBufferOffset();
        const key = deserializer.deserializeStr();
        const keyEnd = deserializer.getBufferOffset();
        if (i > 0) {
            deserializer.checkThatKeySlicesAreIncreasing(
                [previousKeyStart, previousKeyEnd],
                [keyStart, keyEnd]);
        }
        previousKeyStart = keyStart;
        previousKeyEnd = keyEnd;
        const value = deserializer.deserializeI32();
        obj.set(key, value);
    }
    return obj;
  }

  static serializeMapStrToVectorI32(value: Map<str,Seq<int32>>, serializer: Serializer): void {
    serializer.serializeLen(value.size);
    const offsets: number[] = [];
    for (const [k, v] of value.entries()) {
      offsets.push(serializer.getBufferOffset());
      serializer.serializeStr(k);
      Helpers.serializeVectorI32(v, serializer);
    }
    serializer.sortMapEntries(offsets);
  }

  static deserializeMapStrToVectorI32(deserializer: Deserializer): Map<str,Seq<int32>> {
    const length = deserializer.deserializeLen();
    const obj = new Map<str, Seq<int32>>();
    let previousKeyStart = 0;
    let previousKeyEnd = 0;
    for (let i = 0; i < length; i++) {
        const keyStart = deserializer.getBufferOffset();
        const key = deserializer.deserializeStr();
        const keyEnd = deserializer.getBufferOffset();
        if (i > 0) {
            deserializer.checkThatKeySlicesAreIncreasing(
                [previousKeyStart, previousKeyEnd],
                [keyStart, keyEnd]);
        }
        previousKeyStart = keyStart;
        previousKeyEnd = keyEnd;
        const value = Helpers.deserializeVectorI32(deserializer);
        obj.set(key, value);
    }
    return obj;
  }

  static serializeOptionVectorSetStr(value: Optional<Seq<Seq<str>>>, serializer: Serializer): void {
    if (value) {
        serializer.serializeOptionTag(true);
        Helpers.serializeVectorSetStr(value, serializer);
    } else {
        serializer.serializeOptionTag(false);
    }
  }

  static deserializeOptionVectorSetStr(deserializer: Deserializer): Optional<Seq<Seq<str>>> {
    const tag = deserializer.deserializeOptionTag();
    if (!tag) {
        return null;
    } else {
        return Helpers.deserializeVectorSetStr(deserializer);
    }
  }

  static serializeSetStr(value: Seq<str>, serializer: Serializer): void {
    serializer.serializeLen(value.length);
    value.forEach((item: str) => {
        serializer.serializeStr(item);
    });
  }

  static deserializeSetStr(deserializer: Deserializer): Seq<str> {
    const length = deserializer.deserializeLen();
    const list: Seq<str> = [];
    for (let i = 0; i < length; i++) {
        list.push(deserializer.deserializeStr());
    }
    return list;
  }

  static serializeVectorI32(value: Seq<int32>, serializer: Serializer): void {
    serializer.serializeLen(value.length);
    value.forEach((item: int32) => {
        serializer.serializeI32(item);
    });
  }

  static deserializeVectorI32(deserializer: Deserializer): Seq<int32> {
    const length = deserializer.deserializeLen();
    const list: Seq<int32> = [];
    for (let i = 0; i < length; i++) {
        list.push(deserializer.deserializeI32());
    }
    return list;
  }

  static serializeVectorSetStr(value: Seq<Seq<str>>, serializer: Serializer): void {
    serializer.serializeLen(value.length);
    value.forEach((item: Seq<str>) => {
        Helpers.serializeSetStr(item, serializer);
    });
  }

  static deserializeVectorSetStr(deserializer: Deserializer): Seq<Seq<str>> {
    const length = deserializer.deserializeLen();
    const list: Seq<Seq<str>> = [];
    for (let i = 0; i < length; i++) {
        list.push(Helpers.deserializeSetStr(deserializer));
    }
    return list;
  }

}

