#![allow(clippy::too_many_lines)]
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use facet::Facet;

use super::{Encoding, tests::emit};

#[test]
fn unit_struct_1() {
    #[derive(Facet)]
    struct UnitStruct;

    let actual = emit::<UnitStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"

    export class UnitStruct {
        constructor () {
        }

        public serialize(serializer: Serializer): void {
        }

        static deserialize(deserializer: Deserializer): UnitStruct {
            return new UnitStruct();
        }

    }
    ");
}

#[test]
fn unit_struct_2() {
    #[derive(Facet)]
    struct UnitStruct {}

    let actual = emit::<UnitStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"

    export class UnitStruct {
        constructor () {
        }

        public serialize(serializer: Serializer): void {
        }

        static deserialize(deserializer: Deserializer): UnitStruct {
            return new UnitStruct();
        }

    }
    ");
}

#[test]
fn newtype_struct() {
    #[derive(Facet)]
    struct NewType(String);

    let actual = emit::<NewType>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type str = string;
    export class NewType {

        constructor (public value: str) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.value);
        }

        static deserialize(deserializer: Deserializer): NewType {
            const value = deserializer.deserializeStr();
            return new NewType(value);
        }

    }
    ");
}

#[test]
fn tuple_struct() {
    #[derive(Facet)]
    struct TupleStruct(String, i32);

    let actual = emit::<TupleStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export class TupleStruct {

        constructor (public field0: str, public field1: int32) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.field0);
            serializer.serializeI32(this.field1);
        }

        static deserialize(deserializer: Deserializer): TupleStruct {
            const field0 = deserializer.deserializeStr();
            const field1 = deserializer.deserializeI32();
            return new TupleStruct(field0,field1);
        }

    }
    ");
}

#[test]
fn struct_with_fields_of_primitive_types() {
    #[derive(Facet)]
    struct StructWithFields {
        unit: (),
        bool: bool,
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        i128: i128,
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
        u128: u128,
        f32: f32,
        f64: f64,
        char: char,
        string: String,
    }

    let actual = emit::<StructWithFields>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type char = string;
    type float32 = number;
    type float64 = number;
    type int128 = bigint;
    type int16 = number;
    type int32 = number;
    type int64 = bigint;
    type int8 = number;
    type str = string;
    type uint128 = bigint;
    type uint16 = number;
    type uint32 = number;
    type uint64 = bigint;
    type uint8 = number;
    type unit = null;
    export class StructWithFields {

        constructor (public unit: unit, public bool: bool, public i8: int8, public i16: int16, public i32: int32, public i64: int64, public i128: int128, public u8: uint8, public u16: uint16, public u32: uint32, public u64: uint64, public u128: uint128, public f32: float32, public f64: float64, public char: char, public string: str) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeUnit(this.unit);
            serializer.serializeBool(this.bool);
            serializer.serializeI8(this.i8);
            serializer.serializeI16(this.i16);
            serializer.serializeI32(this.i32);
            serializer.serializeI64(this.i64);
            serializer.serializeI128(this.i128);
            serializer.serializeU8(this.u8);
            serializer.serializeU16(this.u16);
            serializer.serializeU32(this.u32);
            serializer.serializeU64(this.u64);
            serializer.serializeU128(this.u128);
            serializer.serializeF32(this.f32);
            serializer.serializeF64(this.f64);
            serializer.serializeChar(this.char);
            serializer.serializeStr(this.string);
        }

        static deserialize(deserializer: Deserializer): StructWithFields {
            const unit = deserializer.deserializeUnit();
            const bool = deserializer.deserializeBool();
            const i8 = deserializer.deserializeI8();
            const i16 = deserializer.deserializeI16();
            const i32 = deserializer.deserializeI32();
            const i64 = deserializer.deserializeI64();
            const i128 = deserializer.deserializeI128();
            const u8 = deserializer.deserializeU8();
            const u16 = deserializer.deserializeU16();
            const u32 = deserializer.deserializeU32();
            const u64 = deserializer.deserializeU64();
            const u128 = deserializer.deserializeU128();
            const f32 = deserializer.deserializeF32();
            const f64 = deserializer.deserializeF64();
            const char = deserializer.deserializeChar();
            const string = deserializer.deserializeStr();
            return new StructWithFields(unit,bool,i8,i16,i32,i64,i128,u8,u16,u32,u64,u128,f32,f64,char,string);
        }

    }
    ");
}

#[test]
fn struct_with_fields_of_user_types() {
    #[derive(Facet)]
    struct Inner1 {
        field1: String,
    }

    #[derive(Facet)]
    struct Inner2(String);

    #[derive(Facet)]
    struct Inner3(String, i32);

    #[derive(Facet)]
    struct Outer {
        one: Inner1,
        two: Inner2,
        three: Inner3,
    }

    let actual = emit::<Outer>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export class Inner1 {

        constructor (public field1: str) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.field1);
        }

        static deserialize(deserializer: Deserializer): Inner1 {
            const field1 = deserializer.deserializeStr();
            return new Inner1(field1);
        }

    }
    export class Inner2 {

        constructor (public value: str) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.value);
        }

        static deserialize(deserializer: Deserializer): Inner2 {
            const value = deserializer.deserializeStr();
            return new Inner2(value);
        }

    }
    export class Inner3 {

        constructor (public field0: str, public field1: int32) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.field0);
            serializer.serializeI32(this.field1);
        }

        static deserialize(deserializer: Deserializer): Inner3 {
            const field0 = deserializer.deserializeStr();
            const field1 = deserializer.deserializeI32();
            return new Inner3(field0,field1);
        }

    }
    export class Outer {

        constructor (public one: Inner1, public two: Inner2, public three: Inner3) {
        }

        public serialize(serializer: Serializer): void {
            this.one.serialize(serializer);
            this.two.serialize(serializer);
            this.three.serialize(serializer);
        }

        static deserialize(deserializer: Deserializer): Outer {
            const one = Inner1.deserialize(deserializer);
            const two = Inner2.deserialize(deserializer);
            const three = Inner3.deserialize(deserializer);
            return new Outer(one,two,three);
        }

    }
    ");
}

#[test]
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    type Tuple<T extends any[]> = T;
    export class MyStruct {

        constructor (public one: Tuple<[str, int32]>) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.one[0]);
            serializer.serializeI32(this.one[1]);
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const field0 = deserializer.deserializeStr();
            const field1 = deserializer.deserializeI32();
            const one = [field0, field1] as [str, int32];
            return new MyStruct(one);
        }

    }
    ");
}

#[test]
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    type Tuple<T extends any[]> = T;
    type uint16 = number;
    export class MyStruct {

        constructor (public one: Tuple<[str, int32, uint16]>) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.one[0]);
            serializer.serializeI32(this.one[1]);
            serializer.serializeU16(this.one[2]);
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const field0 = deserializer.deserializeStr();
            const field1 = deserializer.deserializeI32();
            const field2 = deserializer.deserializeU16();
            const one = [field0, field1, field2] as [str, int32, uint16];
            return new MyStruct(one);
        }

    }
    ");
}

#[test]
fn struct_with_field_that_is_a_4_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16, f32),
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type float32 = number;
    type int32 = number;
    type str = string;
    type Tuple<T extends any[]> = T;
    type uint16 = number;
    export class MyStruct {

        constructor (public one: Tuple<[str, int32, uint16, float32]>) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.one[0]);
            serializer.serializeI32(this.one[1]);
            serializer.serializeU16(this.one[2]);
            serializer.serializeF32(this.one[3]);
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const field0 = deserializer.deserializeStr();
            const field1 = deserializer.deserializeI32();
            const field2 = deserializer.deserializeU16();
            const field3 = deserializer.deserializeF32();
            const one = [field0, field1, field2, field3] as [str, int32, uint16, float32];
            return new MyStruct(one);
        }

    }
    ");
}

#[test]
fn enum_with_unit_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum EnumWithUnitVariants {
        Variant1,
        Variant2,
        Variant3,
    }

    let actual = emit::<EnumWithUnitVariants>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @r#"

    export abstract class EnumWithUnitVariants {
        abstract serialize(serializer: Serializer): void;

        static deserialize(deserializer: Deserializer): EnumWithUnitVariants {
            const index = deserializer.deserializeVariantIndex();
            switch (index) {
                case 0: return EnumWithUnitVariantsVariantVariant1.load(deserializer);
                case 1: return EnumWithUnitVariantsVariantVariant2.load(deserializer);
                case 2: return EnumWithUnitVariantsVariantVariant3.load(deserializer);
                default: throw new Error("Unknown variant index for EnumWithUnitVariants: " + index);
            }
        }
    }


    export class EnumWithUnitVariantsVariantVariant1 extends EnumWithUnitVariants {
        constructor () {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(0);
        }

        static load(deserializer: Deserializer): EnumWithUnitVariantsVariantVariant1 {
            return new EnumWithUnitVariantsVariantVariant1();
        }

    }

    export class EnumWithUnitVariantsVariantVariant2 extends EnumWithUnitVariants {
        constructor () {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(1);
        }

        static load(deserializer: Deserializer): EnumWithUnitVariantsVariantVariant2 {
            return new EnumWithUnitVariantsVariantVariant2();
        }

    }

    export class EnumWithUnitVariantsVariantVariant3 extends EnumWithUnitVariants {
        constructor () {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(2);
        }

        static load(deserializer: Deserializer): EnumWithUnitVariantsVariantVariant3 {
            return new EnumWithUnitVariantsVariantVariant3();
        }

    }
    "#);
}

#[test]
fn enum_with_unit_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        // TypeScript has the same emitted shape for unit and unit-struct variants.
        Variant1 {},
    }

    let actual = emit::<MyEnum>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @r#"

    export abstract class MyEnum {
        abstract serialize(serializer: Serializer): void;

        static deserialize(deserializer: Deserializer): MyEnum {
            const index = deserializer.deserializeVariantIndex();
            switch (index) {
                case 0: return MyEnumVariantVariant1.load(deserializer);
                default: throw new Error("Unknown variant index for MyEnum: " + index);
            }
        }
    }


    export class MyEnumVariantVariant1 extends MyEnum {
        constructor () {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(0);
        }

        static load(deserializer: Deserializer): MyEnumVariantVariant1 {
            return new MyEnumVariantVariant1();
        }

    }
    "#);
}

#[test]
fn enum_with_1_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
    }

    let actual = emit::<MyEnum>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @r#"
    type str = string;
    export abstract class MyEnum {
        abstract serialize(serializer: Serializer): void;

        static deserialize(deserializer: Deserializer): MyEnum {
            const index = deserializer.deserializeVariantIndex();
            switch (index) {
                case 0: return MyEnumVariantVariant1.load(deserializer);
                default: throw new Error("Unknown variant index for MyEnum: " + index);
            }
        }
    }


    export class MyEnumVariantVariant1 extends MyEnum {

        constructor (public value: str) {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(0);
            serializer.serializeStr(this.value);
        }

        static load(deserializer: Deserializer): MyEnumVariantVariant1 {
            const value = deserializer.deserializeStr();
            return new MyEnumVariantVariant1(value);
        }

    }
    "#);
}

#[test]
fn enum_with_newtype_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
        Variant2(i32),
    }

    let actual = emit::<MyEnum>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @r#"
    type int32 = number;
    type str = string;
    export abstract class MyEnum {
        abstract serialize(serializer: Serializer): void;

        static deserialize(deserializer: Deserializer): MyEnum {
            const index = deserializer.deserializeVariantIndex();
            switch (index) {
                case 0: return MyEnumVariantVariant1.load(deserializer);
                case 1: return MyEnumVariantVariant2.load(deserializer);
                default: throw new Error("Unknown variant index for MyEnum: " + index);
            }
        }
    }


    export class MyEnumVariantVariant1 extends MyEnum {

        constructor (public value: str) {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(0);
            serializer.serializeStr(this.value);
        }

        static load(deserializer: Deserializer): MyEnumVariantVariant1 {
            const value = deserializer.deserializeStr();
            return new MyEnumVariantVariant1(value);
        }

    }

    export class MyEnumVariantVariant2 extends MyEnum {

        constructor (public value: int32) {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(1);
            serializer.serializeI32(this.value);
        }

        static load(deserializer: Deserializer): MyEnumVariantVariant2 {
            const value = deserializer.deserializeI32();
            return new MyEnumVariantVariant2(value);
        }

    }
    "#);
}

#[test]
fn enum_with_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String, i32),
        Variant2(bool, f64, u8),
    }

    let actual = emit::<MyEnum>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @r#"
    type bool = boolean;
    type float64 = number;
    type int32 = number;
    type str = string;
    type uint8 = number;
    export abstract class MyEnum {
        abstract serialize(serializer: Serializer): void;

        static deserialize(deserializer: Deserializer): MyEnum {
            const index = deserializer.deserializeVariantIndex();
            switch (index) {
                case 0: return MyEnumVariantVariant1.load(deserializer);
                case 1: return MyEnumVariantVariant2.load(deserializer);
                default: throw new Error("Unknown variant index for MyEnum: " + index);
            }
        }
    }


    export class MyEnumVariantVariant1 extends MyEnum {

        constructor (public field0: str, public field1: int32) {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(0);
            serializer.serializeStr(this.field0);
            serializer.serializeI32(this.field1);
        }

        static load(deserializer: Deserializer): MyEnumVariantVariant1 {
            const field0 = deserializer.deserializeStr();
            const field1 = deserializer.deserializeI32();
            return new MyEnumVariantVariant1(field0,field1);
        }

    }

    export class MyEnumVariantVariant2 extends MyEnum {

        constructor (public field0: bool, public field1: float64, public field2: uint8) {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(1);
            serializer.serializeBool(this.field0);
            serializer.serializeF64(this.field1);
            serializer.serializeU8(this.field2);
        }

        static load(deserializer: Deserializer): MyEnumVariantVariant2 {
            const field0 = deserializer.deserializeBool();
            const field1 = deserializer.deserializeF64();
            const field2 = deserializer.deserializeU8();
            return new MyEnumVariantVariant2(field0,field1,field2);
        }

    }
    "#);
}

#[test]
fn enum_with_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 { field1: String, field2: i32 },
    }

    let actual = emit::<MyEnum>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @r#"
    type int32 = number;
    type str = string;
    export abstract class MyEnum {
        abstract serialize(serializer: Serializer): void;

        static deserialize(deserializer: Deserializer): MyEnum {
            const index = deserializer.deserializeVariantIndex();
            switch (index) {
                case 0: return MyEnumVariantVariant1.load(deserializer);
                default: throw new Error("Unknown variant index for MyEnum: " + index);
            }
        }
    }


    export class MyEnumVariantVariant1 extends MyEnum {

        constructor (public field1: str, public field2: int32) {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(0);
            serializer.serializeStr(this.field1);
            serializer.serializeI32(this.field2);
        }

        static load(deserializer: Deserializer): MyEnumVariantVariant1 {
            const field1 = deserializer.deserializeStr();
            const field2 = deserializer.deserializeI32();
            return new MyEnumVariantVariant1(field1,field2);
        }

    }
    "#);
}

#[test]
fn enum_with_mixed_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Unit,
        NewType(String),
        Tuple(String, i32),
        Struct { field: bool },
    }

    let actual = emit::<MyEnum>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @r#"
    type bool = boolean;
    type int32 = number;
    type str = string;
    export abstract class MyEnum {
        abstract serialize(serializer: Serializer): void;

        static deserialize(deserializer: Deserializer): MyEnum {
            const index = deserializer.deserializeVariantIndex();
            switch (index) {
                case 0: return MyEnumVariantUnit.load(deserializer);
                case 1: return MyEnumVariantNewType.load(deserializer);
                case 2: return MyEnumVariantTuple.load(deserializer);
                case 3: return MyEnumVariantStruct.load(deserializer);
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

    export class MyEnumVariantTuple extends MyEnum {

        constructor (public field0: str, public field1: int32) {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(2);
            serializer.serializeStr(this.field0);
            serializer.serializeI32(this.field1);
        }

        static load(deserializer: Deserializer): MyEnumVariantTuple {
            const field0 = deserializer.deserializeStr();
            const field1 = deserializer.deserializeI32();
            return new MyEnumVariantTuple(field0,field1);
        }

    }

    export class MyEnumVariantStruct extends MyEnum {

        constructor (public field: bool) {
            super();
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeVariantIndex(3);
            serializer.serializeBool(this.field);
        }

        static load(deserializer: Deserializer): MyEnumVariantStruct {
            const field = deserializer.deserializeBool();
            return new MyEnumVariantStruct(field);
        }

    }
    "#);
}

#[test]
fn struct_with_vec_field_1() {
    #[derive(Facet)]
    struct MyStruct {
        items: Vec<String>,
        numbers: Vec<i32>,
        nested_items: Vec<Vec<String>>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

        constructor (public items: Seq<str>, public numbers: Seq<int32>, public nested_items: Seq<Seq<str>>) {
        }

        public serialize(serializer: Serializer): void {
            serializeArray(this.items, serializer, (item, serializer) => {
                serializer.serializeStr(item);
            });
            serializeArray(this.numbers, serializer, (item, serializer) => {
                serializer.serializeI32(item);
            });
            serializeArray(this.nested_items, serializer, (item, serializer) => {
                serializeArray(item, serializer, (item, serializer) => {
                    serializer.serializeStr(item);
                });
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const items = deserializeArray(deserializer, (deserializer) => {
                return deserializer.deserializeStr();
            });
            const numbers = deserializeArray(deserializer, (deserializer) => {
                return deserializer.deserializeI32();
            });
            const nested_items = deserializeArray(deserializer, (deserializer) => {
                return deserializeArray(deserializer, (deserializer) => {
                    return deserializer.deserializeStr();
                });
            });
            return new MyStruct(items,numbers,nested_items);
        }

    }
    ");
}

#[test]
fn struct_with_vec_field_2() {
    #[derive(Facet)]
    pub struct Child {
        name: String,
    }

    #[derive(Facet)]
    pub struct Parent {
        children: Vec<Vec<Child>>,
    }

    let actual = emit::<Parent>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
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
    export class Parent {

        constructor (public children: Seq<Seq<Child>>) {
        }

        public serialize(serializer: Serializer): void {
            serializeArray(this.children, serializer, (item, serializer) => {
                serializeArray(item, serializer, (item, serializer) => {
                    item.serialize(serializer);
                });
            });
        }

        static deserialize(deserializer: Deserializer): Parent {
            const children = deserializeArray(deserializer, (deserializer) => {
                return deserializeArray(deserializer, (deserializer) => {
                    return Child.deserialize(deserializer);
                });
            });
            return new Parent(children);
        }

    }
    ");
}

#[test]
fn struct_with_option_field() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names, clippy::option_option)]
    struct MyStruct {
        simple: Option<String>,
        nested: Option<Option<i32>>,
        list: Option<Vec<bool>>,
        list_of_options: Vec<Option<bool>>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type int32 = number;
    type Optional<T> = T | null;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

        constructor (public simple: Optional<str>, public nested: Optional<Optional<int32>>, public list: Optional<Seq<bool>>, public list_of_options: Seq<Optional<bool>>) {
        }

        public serialize(serializer: Serializer): void {
            serializeOption(this.simple, serializer, (value, serializer) => {
                serializer.serializeStr(value);
            });
            serializeOption(this.nested, serializer, (value, serializer) => {
                serializeOption(value, serializer, (value, serializer) => {
                    serializer.serializeI32(value);
                });
            });
            serializeOption(this.list, serializer, (value, serializer) => {
                serializeArray(value, serializer, (item, serializer) => {
                    serializer.serializeBool(item);
                });
            });
            serializeArray(this.list_of_options, serializer, (item, serializer) => {
                serializeOption(item, serializer, (value, serializer) => {
                    serializer.serializeBool(value);
                });
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const simple = deserializeOption(deserializer, (deserializer) => {
                return deserializer.deserializeStr();
            });
            const nested = deserializeOption(deserializer, (deserializer) => {
                return deserializeOption(deserializer, (deserializer) => {
                    return deserializer.deserializeI32();
                });
            });
            const list = deserializeOption(deserializer, (deserializer) => {
                return deserializeArray(deserializer, (deserializer) => {
                    return deserializer.deserializeBool();
                });
            });
            const list_of_options = deserializeArray(deserializer, (deserializer) => {
                return deserializeOption(deserializer, (deserializer) => {
                    return deserializer.deserializeBool();
                });
            });
            return new MyStruct(simple,nested,list,list_of_options);
        }

    }
    ");
}

#[test]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        int_to_bool: HashMap<i32, bool>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type int32 = number;
    type str = string;
    export class MyStruct {

        constructor (public string_to_int: Map<str,int32>, public int_to_bool: Map<int32,bool>) {
        }

        public serialize(serializer: Serializer): void {
            serializeMap(this.string_to_int, serializer, (key, value, serializer) => {
                serializer.serializeStr(key);
                serializer.serializeI32(value);
            });
            serializeMap(this.int_to_bool, serializer, (key, value, serializer) => {
                serializer.serializeI32(key);
                serializer.serializeBool(value);
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const string_to_int = deserializeMap(deserializer, (deserializer) => {
                const key = deserializer.deserializeStr();
                const value = deserializer.deserializeI32();
                return [key, value];
            });
            const int_to_bool = deserializeMap(deserializer, (deserializer) => {
                const key = deserializer.deserializeI32();
                const value = deserializer.deserializeBool();
                return [key, value];
            });
            return new MyStruct(string_to_int,int_to_bool);
        }

    }
    ");
}

#[test]
fn struct_with_nested_generics() {
    #[derive(Facet)]
    struct MyStruct {
        optional_list: Option<Vec<String>>,
        list_of_optionals: Vec<Option<i32>>,
        map_to_list: HashMap<String, Vec<bool>>,
        optional_map: Option<HashMap<String, i32>>,
        complex: Vec<Option<HashMap<String, Vec<bool>>>>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type int32 = number;
    type Optional<T> = T | null;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

        constructor (public optional_list: Optional<Seq<str>>, public list_of_optionals: Seq<Optional<int32>>, public map_to_list: Map<str,Seq<bool>>, public optional_map: Optional<Map<str,int32>>, public complex: Seq<Optional<Map<str,Seq<bool>>>>) {
        }

        public serialize(serializer: Serializer): void {
            serializeOption(this.optional_list, serializer, (value, serializer) => {
                serializeArray(value, serializer, (item, serializer) => {
                    serializer.serializeStr(item);
                });
            });
            serializeArray(this.list_of_optionals, serializer, (item, serializer) => {
                serializeOption(item, serializer, (value, serializer) => {
                    serializer.serializeI32(value);
                });
            });
            serializeMap(this.map_to_list, serializer, (key, value, serializer) => {
                serializer.serializeStr(key);
                serializeArray(value, serializer, (item, serializer) => {
                    serializer.serializeBool(item);
                });
            });
            serializeOption(this.optional_map, serializer, (value, serializer) => {
                serializeMap(value, serializer, (key, value, serializer) => {
                    serializer.serializeStr(key);
                    serializer.serializeI32(value);
                });
            });
            serializeArray(this.complex, serializer, (item, serializer) => {
                serializeOption(item, serializer, (value, serializer) => {
                    serializeMap(value, serializer, (key, value, serializer) => {
                        serializer.serializeStr(key);
                        serializeArray(value, serializer, (item, serializer) => {
                            serializer.serializeBool(item);
                        });
                    });
                });
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const optional_list = deserializeOption(deserializer, (deserializer) => {
                return deserializeArray(deserializer, (deserializer) => {
                    return deserializer.deserializeStr();
                });
            });
            const list_of_optionals = deserializeArray(deserializer, (deserializer) => {
                return deserializeOption(deserializer, (deserializer) => {
                    return deserializer.deserializeI32();
                });
            });
            const map_to_list = deserializeMap(deserializer, (deserializer) => {
                const key = deserializer.deserializeStr();
                const value = deserializeArray(deserializer, (deserializer) => {
                    return deserializer.deserializeBool();
                });
                return [key, value];
            });
            const optional_map = deserializeOption(deserializer, (deserializer) => {
                return deserializeMap(deserializer, (deserializer) => {
                    const key = deserializer.deserializeStr();
                    const value = deserializer.deserializeI32();
                    return [key, value];
                });
            });
            const complex = deserializeArray(deserializer, (deserializer) => {
                return deserializeOption(deserializer, (deserializer) => {
                    return deserializeMap(deserializer, (deserializer) => {
                        const key = deserializer.deserializeStr();
                        const value = deserializeArray(deserializer, (deserializer) => {
                            return deserializer.deserializeBool();
                        });
                        return [key, value];
                    });
                });
            });
            return new MyStruct(optional_list,list_of_optionals,map_to_list,optional_map,complex);
        }

    }
    ");
}

#[test]
fn struct_with_array_field() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct MyStruct {
        fixed_array: [i32; 5],
        byte_array: [u8; 32],
        string_array: [String; 3],
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type ListTuple<T extends any[]> = Tuple<T>[];
    type str = string;
    type uint8 = number;
    export class MyStruct {

        constructor (public fixed_array: ListTuple<[int32]>, public byte_array: ListTuple<[uint8]>, public string_array: ListTuple<[str]>) {
        }

        public serialize(serializer: Serializer): void {
            serializeTupleArray(this.fixed_array, serializer, (item, serializer) => {
                serializer.serializeI32(item[0]);
            });
            serializeTupleArray(this.byte_array, serializer, (item, serializer) => {
                serializer.serializeU8(item[0]);
            });
            serializeTupleArray(this.string_array, serializer, (item, serializer) => {
                serializer.serializeStr(item[0]);
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const fixed_array = deserializeTupleArray(deserializer, 5, (deserializer) => {
                const item = deserializer.deserializeI32();
                return [item];
            });
            const byte_array = deserializeTupleArray(deserializer, 32, (deserializer) => {
                const item = deserializer.deserializeU8();
                return [item];
            });
            const string_array = deserializeTupleArray(deserializer, 3, (deserializer) => {
                const item = deserializer.deserializeStr();
                return [item];
            });
            return new MyStruct(fixed_array,byte_array,string_array);
        }

    }
    ");
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type int32 = number;
    type str = string;
    export class MyStruct {

        constructor (public string_to_int: Map<str,int32>, public int_to_bool: Map<int32,bool>) {
        }

        public serialize(serializer: Serializer): void {
            serializeMap(this.string_to_int, serializer, (key, value, serializer) => {
                serializer.serializeStr(key);
                serializer.serializeI32(value);
            });
            serializeMap(this.int_to_bool, serializer, (key, value, serializer) => {
                serializer.serializeI32(key);
                serializer.serializeBool(value);
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const string_to_int = deserializeMap(deserializer, (deserializer) => {
                const key = deserializer.deserializeStr();
                const value = deserializer.deserializeI32();
                return [key, value];
            });
            const int_to_bool = deserializeMap(deserializer, (deserializer) => {
                const key = deserializer.deserializeI32();
                const value = deserializer.deserializeBool();
                return [key, value];
            });
            return new MyStruct(string_to_int,int_to_bool);
        }

    }
    ");
}

#[test]
fn struct_with_nested_map_field() {
    #[derive(Facet)]
    struct MyStruct {
        map_to_list: HashMap<String, Vec<i32>>,
        list_to_map: Vec<HashMap<i32, String>>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

        constructor (public map_to_list: Map<str,Seq<int32>>, public list_to_map: Seq<Map<int32,str>>) {
        }

        public serialize(serializer: Serializer): void {
            serializeMap(this.map_to_list, serializer, (key, value, serializer) => {
                serializer.serializeStr(key);
                serializeArray(value, serializer, (item, serializer) => {
                    serializer.serializeI32(item);
                });
            });
            serializeArray(this.list_to_map, serializer, (item, serializer) => {
                serializeMap(item, serializer, (key, value, serializer) => {
                    serializer.serializeI32(key);
                    serializer.serializeStr(value);
                });
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const map_to_list = deserializeMap(deserializer, (deserializer) => {
                const key = deserializer.deserializeStr();
                const value = deserializeArray(deserializer, (deserializer) => {
                    return deserializer.deserializeI32();
                });
                return [key, value];
            });
            const list_to_map = deserializeArray(deserializer, (deserializer) => {
                return deserializeMap(deserializer, (deserializer) => {
                    const key = deserializer.deserializeI32();
                    const value = deserializer.deserializeStr();
                    return [key, value];
                });
            });
            return new MyStruct(map_to_list,list_to_map);
        }

    }
    ");
}

#[test]
fn struct_with_hashset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: HashSet<String>,
        int_set: HashSet<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

        constructor (public string_set: Seq<str>, public int_set: Seq<int32>) {
        }

        public serialize(serializer: Serializer): void {
            serializeSet(this.string_set, serializer, (item, serializer) => {
                serializer.serializeStr(item);
            });
            serializeSet(this.int_set, serializer, (item, serializer) => {
                serializer.serializeI32(item);
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const string_set = deserializeSet(deserializer, (deserializer) => {
                return deserializer.deserializeStr();
            });
            const int_set = deserializeSet(deserializer, (deserializer) => {
                return deserializer.deserializeI32();
            });
            return new MyStruct(string_set,int_set);
        }

    }
    ");
}

#[test]
fn struct_with_btreeset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

        constructor (public string_set: Seq<str>, public int_set: Seq<int32>) {
        }

        public serialize(serializer: Serializer): void {
            serializeSet(this.string_set, serializer, (item, serializer) => {
                serializer.serializeStr(item);
            });
            serializeSet(this.int_set, serializer, (item, serializer) => {
                serializer.serializeI32(item);
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const string_set = deserializeSet(deserializer, (deserializer) => {
                return deserializer.deserializeStr();
            });
            const int_set = deserializeSet(deserializer, (deserializer) => {
                return deserializer.deserializeI32();
            });
            return new MyStruct(string_set,int_set);
        }

    }
    ");
}

#[test]
fn struct_with_nested_set_field() {
    #[derive(Facet)]
    struct MyStruct {
        vec_of_sets: Vec<HashSet<String>>,
        set_of_ints: HashSet<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

        constructor (public vec_of_sets: Seq<Seq<str>>, public set_of_ints: Seq<int32>) {
        }

        public serialize(serializer: Serializer): void {
            serializeArray(this.vec_of_sets, serializer, (item, serializer) => {
                serializeSet(item, serializer, (item, serializer) => {
                    serializer.serializeStr(item);
                });
            });
            serializeSet(this.set_of_ints, serializer, (item, serializer) => {
                serializer.serializeI32(item);
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const vec_of_sets = deserializeArray(deserializer, (deserializer) => {
                return deserializeSet(deserializer, (deserializer) => {
                    return deserializer.deserializeStr();
                });
            });
            const set_of_ints = deserializeSet(deserializer, (deserializer) => {
                return deserializer.deserializeI32();
            });
            return new MyStruct(vec_of_sets,set_of_ints);
        }

    }
    ");
}

#[test]
fn struct_with_box_field() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        boxed_string: Box<String>,
        boxed_int: Box<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export class MyStruct {

        constructor (public boxed_string: str, public boxed_int: int32) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.boxed_string);
            serializer.serializeI32(this.boxed_int);
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const boxed_string = deserializer.deserializeStr();
            const boxed_int = deserializer.deserializeI32();
            return new MyStruct(boxed_string,boxed_int);
        }

    }
    ");
}

#[test]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: Rc<String>,
        rc_int: Rc<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export class MyStruct {

        constructor (public rc_string: str, public rc_int: int32) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.rc_string);
            serializer.serializeI32(this.rc_int);
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const rc_string = deserializer.deserializeStr();
            const rc_int = deserializer.deserializeI32();
            return new MyStruct(rc_string,rc_int);
        }

    }
    ");
}

#[test]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: Arc<String>,
        arc_int: Arc<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export class MyStruct {

        constructor (public arc_string: str, public arc_int: int32) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeStr(this.arc_string);
            serializer.serializeI32(this.arc_int);
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const arc_string = deserializer.deserializeStr();
            const arc_int = deserializer.deserializeI32();
            return new MyStruct(arc_string,arc_int);
        }

    }
    ");
}

#[test]
fn struct_with_mixed_collections_and_pointers() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        vec_of_sets: Vec<HashSet<String>>,
        optional_btree: Option<BTreeMap<String, i32>>,
        boxed_vec: Box<Vec<String>>,
        arc_option: Arc<Option<String>>,
        array_of_boxes: [Box<i32>; 3],
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type ListTuple<T extends any[]> = Tuple<T>[];
    type Optional<T> = T | null;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

        constructor (public vec_of_sets: Seq<Seq<str>>, public optional_btree: Optional<Map<str,int32>>, public boxed_vec: Seq<str>, public arc_option: Optional<str>, public array_of_boxes: ListTuple<[int32]>) {
        }

        public serialize(serializer: Serializer): void {
            serializeArray(this.vec_of_sets, serializer, (item, serializer) => {
                serializeSet(item, serializer, (item, serializer) => {
                    serializer.serializeStr(item);
                });
            });
            serializeOption(this.optional_btree, serializer, (value, serializer) => {
                serializeMap(value, serializer, (key, value, serializer) => {
                    serializer.serializeStr(key);
                    serializer.serializeI32(value);
                });
            });
            serializeArray(this.boxed_vec, serializer, (item, serializer) => {
                serializer.serializeStr(item);
            });
            serializeOption(this.arc_option, serializer, (value, serializer) => {
                serializer.serializeStr(value);
            });
            serializeTupleArray(this.array_of_boxes, serializer, (item, serializer) => {
                serializer.serializeI32(item[0]);
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const vec_of_sets = deserializeArray(deserializer, (deserializer) => {
                return deserializeSet(deserializer, (deserializer) => {
                    return deserializer.deserializeStr();
                });
            });
            const optional_btree = deserializeOption(deserializer, (deserializer) => {
                return deserializeMap(deserializer, (deserializer) => {
                    const key = deserializer.deserializeStr();
                    const value = deserializer.deserializeI32();
                    return [key, value];
                });
            });
            const boxed_vec = deserializeArray(deserializer, (deserializer) => {
                return deserializer.deserializeStr();
            });
            const arc_option = deserializeOption(deserializer, (deserializer) => {
                return deserializer.deserializeStr();
            });
            const array_of_boxes = deserializeTupleArray(deserializer, 3, (deserializer) => {
                const item = deserializer.deserializeI32();
                return [item];
            });
            return new MyStruct(vec_of_sets,optional_btree,boxed_vec,arc_option,array_of_boxes);
        }

    }
    ");
}

#[test]
fn struct_with_bytes_field() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(bytes)]
        data: Vec<u8>,
        name: String,
        #[facet(bytes)]
        header: Vec<u8>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type bytes = Uint8Array;
    type str = string;
    export class MyStruct {

        constructor (public data: bytes, public name: str, public header: bytes) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeBytes(this.data);
            serializer.serializeStr(this.name);
            serializer.serializeBytes(this.header);
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const data = deserializer.deserializeBytes();
            const name = deserializer.deserializeStr();
            const header = deserializer.deserializeBytes();
            return new MyStruct(data,name,header);
        }

    }
    ");
}

#[test]
fn struct_with_bytes_field_and_slice() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(bytes)]
        data: &'static [u8],
        name: String,
        #[facet(bytes)]
        header: Vec<u8>,
        optional_bytes: Option<Vec<u8>>,
    }

    let actual = emit::<MyStruct>(Encoding::Bincode);
    insta::assert_snapshot!(actual, @"
    type bytes = Uint8Array;
    type Optional<T> = T | null;
    type Seq<T> = T[];
    type str = string;
    type uint8 = number;
    export class MyStruct {

        constructor (public data: bytes, public name: str, public header: bytes, public optional_bytes: Optional<Seq<uint8>>) {
        }

        public serialize(serializer: Serializer): void {
            serializer.serializeBytes(this.data);
            serializer.serializeStr(this.name);
            serializer.serializeBytes(this.header);
            serializeOption(this.optional_bytes, serializer, (value, serializer) => {
                serializeArray(value, serializer, (item, serializer) => {
                    serializer.serializeU8(item);
                });
            });
        }

        static deserialize(deserializer: Deserializer): MyStruct {
            const data = deserializer.deserializeBytes();
            const name = deserializer.deserializeStr();
            const header = deserializer.deserializeBytes();
            const optional_bytes = deserializeOption(deserializer, (deserializer) => {
                return deserializeArray(deserializer, (deserializer) => {
                    return deserializer.deserializeU8();
                });
            });
            return new MyStruct(data,name,header,optional_bytes);
        }

    }
    ");
}
