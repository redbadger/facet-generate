use facet::Facet;

use super::{Encoding, tests::emit};

#[test]
fn struct_with_json_encoding() {
    #[derive(Facet)]
    struct MyStruct {
        name: String,
        age: u32,
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    insta::assert_snapshot!(actual, @"
    type str = string;
    type uint32 = number;
    export class MyStruct {

      constructor (public name: str, public age: uint32) {
      }

      public serialize(serializer: Serializer): void {
        serializer.serializeStr(this.name);
        serializer.serializeU32(this.age);
      }

      static deserialize(deserializer: Deserializer): MyStruct {
        const name = deserializer.deserializeStr();
        const age = deserializer.deserializeU32();
        return new MyStruct(name,age);
      }

    }
    ");
}

#[test]
fn enum_with_json_encoding() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Unit,
        NewType(String),
        Struct { x: u32, y: i64 },
    }

    let actual = emit::<MyEnum>(Encoding::Json);
    insta::assert_snapshot!(actual, @r#"
    type int64 = bigint;
    type str = string;
    type uint32 = number;
    export abstract class MyEnum {
      abstract serialize(serializer: Serializer): void;

      static deserialize(deserializer: Deserializer): MyEnum {
        const index = deserializer.deserializeVariantIndex();
        switch (index) {
          case 0: return MyEnumVariantUnit.load(deserializer);
          case 1: return MyEnumVariantNewType.load(deserializer);
          case 2: return MyEnumVariantStruct.load(deserializer);
          default: throw new Error("Unknown variant index for MyEnum: " + index);
        }
      }
    }


    export class MyEnumVariantUnit extends MyEnum {
      constructor () {
        super();
      }

      public serialize(serializer: Serializer): void {
        serializer.serializeVariantIndex(0);
      }

      static load(deserializer: Deserializer): MyEnumVariantUnit {
        return new MyEnumVariantUnit();
      }

    }

    export class MyEnumVariantNewType extends MyEnum {

      constructor (public value: str) {
        super();
      }

      public serialize(serializer: Serializer): void {
        serializer.serializeVariantIndex(1);
        serializer.serializeStr(this.value);
      }

      static load(deserializer: Deserializer): MyEnumVariantNewType {
        const value = deserializer.deserializeStr();
        return new MyEnumVariantNewType(value);
      }

    }

    export class MyEnumVariantStruct extends MyEnum {

      constructor (public x: uint32, public y: int64) {
        super();
      }

      public serialize(serializer: Serializer): void {
        serializer.serializeVariantIndex(2);
        serializer.serializeU32(this.x);
        serializer.serializeI64(this.y);
      }

      static load(deserializer: Deserializer): MyEnumVariantStruct {
        const x = deserializer.deserializeU32();
        const y = deserializer.deserializeI64();
        return new MyEnumVariantStruct(x,y);
      }

    }
    "#);
}
