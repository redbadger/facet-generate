#![allow(clippy::too_many_lines)]
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use facet::Facet;

use super::*;
use crate::{emit_two_modules, generation::typescript::CodeGenerator};

#[test]
fn test_format_type_aliases() {
    let input = BTreeSet::from([
        "bool".to_string(),
        "bytes".to_string(),
        "char".to_string(),
        "float32".to_string(),
        "float64".to_string(),
        "int128".to_string(),
        "int16".to_string(),
        "int32".to_string(),
        "int64".to_string(),
        "int8".to_string(),
        "list_tuple".to_string(),
        "option".to_string(),
        "seq".to_string(),
        "str".to_string(),
        "tuple".to_string(),
        "uint128".to_string(),
        "uint16".to_string(),
        "uint32".to_string(),
        "uint64".to_string(),
        "uint8".to_string(),
        "unit".to_string(),
    ]);
    let actual = format_type_aliases(&input);
    insta::assert_snapshot!(&actual, @"
    type bool = boolean;
    type bytes = Uint8Array;
    type char = string;
    type float32 = number;
    type float64 = number;
    type int128 = bigint;
    type int16 = number;
    type int32 = number;
    type int64 = bigint;
    type int8 = number;
    type ListTuple<T extends any[]> = Tuple<T>[];
    type Optional<T> = T | null;
    type Seq<T> = T[];
    type str = string;
    type Tuple<T extends any[]> = T;
    type uint128 = bigint;
    type uint16 = number;
    type uint32 = number;
    type uint64 = bigint;
    type uint8 = number;
    type unit = null;
    ");
}

pub(super) fn emit<T: for<'a> facet::Facet<'a>>(encoding: Encoding) -> String {
    use crate::generation::{
        Container, Emitter,
        indent::{IndentConfig, IndentedWriter},
    };

    let registry = crate::reflect!(T).unwrap();

    let type_alias_keys = collect_type_alias_keys(&registry);
    let lang = TypeScript::new(encoding, InstallTarget::Node);

    let mut out = Vec::new();
    let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(2));

    write_type_aliases(&mut w, &type_alias_keys).unwrap();
    for container in registry.iter().map(Container::from) {
        container.write(&mut w, lang).unwrap();
    }

    String::from_utf8(out).unwrap()
}

#[test]
fn unit_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit::<UnitStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    export class UnitStruct {
      constructor () {
      }

    }
    ");
}

#[test]
fn unit_struct_empty_body() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct {}

    let actual = emit::<UnitStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    export class UnitStruct {
      constructor () {
      }

    }
    ");
}

#[test]
fn newtype_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct NewType(String);

    let actual = emit::<NewType>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type str = string;
    /// line 1
    /// line 2
    export class NewType {

      constructor (public value: str) {
      }

    }
    ");
}

#[test]
fn tuple_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct TupleStruct(String, i32);

    let actual = emit::<TupleStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    /// line 1
    /// line 2
    export class TupleStruct {

      constructor (public field0: str, public field1: int32) {
      }

    }
    ");
}

#[test]
fn struct_with_fields_of_primitive_types() {
    /// line 1
    #[derive(Facet)]
    /// line 2
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

    let actual = emit::<StructWithFields>(Encoding::None);
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
    /// line 1
    /// line 2
    export class StructWithFields {

      constructor (public unit: unit, public bool: bool, public i8: int8, public i16: int16, public i32: int32, public i64: int64, public i128: int128, public u8: uint8, public u16: uint16, public u32: uint32, public u64: uint64, public u128: uint128, public f32: float32, public f64: float64, public char: char, public string: str) {
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

    let actual = emit::<Outer>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export class Inner1 {

      constructor (public field1: str) {
      }

    }
    export class Inner2 {

      constructor (public value: str) {
      }

    }
    export class Inner3 {

      constructor (public field0: str, public field1: int32) {
      }

    }
    export class Outer {

      constructor (public one: Inner1, public two: Inner2, public three: Inner3) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    type Tuple<T extends any[]> = T;
    export class MyStruct {

      constructor (public one: Tuple<[str, int32]>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    type Tuple<T extends any[]> = T;
    type uint16 = number;
    export class MyStruct {

      constructor (public one: Tuple<[str, int32, uint16]>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type float32 = number;
    type int32 = number;
    type str = string;
    type Tuple<T extends any[]> = T;
    type uint16 = number;
    export class MyStruct {

      constructor (public one: Tuple<[str, int32, uint16, float32]>) {
      }

    }
    ");
}

#[test]
fn enum_with_unit_variants() {
    /// line one
    #[derive(Facet)]
    #[repr(C)]
    /// line two
    #[allow(unused)]
    enum EnumWithUnitVariants {
        /// variant one
        Variant1,
        /// variant two
        Variant2,
        /// variant three
        Variant3,
    }

    let actual = emit::<EnumWithUnitVariants>(Encoding::None);
    insta::assert_snapshot!(actual, @"

    /// line one
    /// line two
    export abstract class EnumWithUnitVariants {
    }


    /// variant one
    export class EnumWithUnitVariantsVariantVariant1 extends EnumWithUnitVariants {
      constructor () {
        super();
      }

    }

    /// variant two
    export class EnumWithUnitVariantsVariantVariant2 extends EnumWithUnitVariants {
      constructor () {
        super();
      }

    }

    /// variant three
    export class EnumWithUnitVariantsVariantVariant3 extends EnumWithUnitVariants {
      constructor () {
        super();
      }

    }
    ");
}

#[test]
fn enum_with_unit_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        // TS has no separate "unit struct variant" representation, so this maps
        // to the same empty variant class shape as a unit variant.
        Variant1 {},
    }

    let actual = emit::<MyEnum>(Encoding::None);
    insta::assert_snapshot!(actual, @"

    export abstract class MyEnum {
    }


    export class MyEnumVariantVariant1 extends MyEnum {
      constructor () {
        super();
      }

    }
    ");
}

#[test]
fn enum_with_1_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
    }

    let actual = emit::<MyEnum>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type str = string;
    export abstract class MyEnum {
    }


    export class MyEnumVariantVariant1 extends MyEnum {

      constructor (public value: str) {
        super();
      }

    }
    ");
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

    let actual = emit::<MyEnum>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export abstract class MyEnum {
    }


    export class MyEnumVariantVariant1 extends MyEnum {

      constructor (public value: str) {
        super();
      }

    }

    export class MyEnumVariantVariant2 extends MyEnum {

      constructor (public value: int32) {
        super();
      }

    }
    ");
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

    let actual = emit::<MyEnum>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type float64 = number;
    type int32 = number;
    type str = string;
    type uint8 = number;
    export abstract class MyEnum {
    }


    export class MyEnumVariantVariant1 extends MyEnum {

      constructor (public field0: str, public field1: int32) {
        super();
      }

    }

    export class MyEnumVariantVariant2 extends MyEnum {

      constructor (public field0: bool, public field1: float64, public field2: uint8) {
        super();
      }

    }
    ");
}

#[test]
fn enum_with_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 { field1: String, field2: i32 },
    }

    let actual = emit::<MyEnum>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export abstract class MyEnum {
    }


    export class MyEnumVariantVariant1 extends MyEnum {

      constructor (public field1: str, public field2: int32) {
        super();
      }

    }
    ");
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

    let actual = emit::<MyEnum>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type int32 = number;
    type str = string;
    export abstract class MyEnum {
    }


    export class MyEnumVariantUnit extends MyEnum {
      constructor () {
        super();
      }

    }

    export class MyEnumVariantNewType extends MyEnum {

      constructor (public value: str) {
        super();
      }

    }

    export class MyEnumVariantTuple extends MyEnum {

      constructor (public field0: str, public field1: int32) {
        super();
      }

    }

    export class MyEnumVariantStruct extends MyEnum {

      constructor (public field: bool) {
        super();
      }

    }
    ");
}

#[test]
fn struct_with_vec_field() {
    #[derive(Facet)]
    struct MyStruct {
        items: Vec<String>,
        numbers: Vec<i32>,
        nested_items: Vec<Vec<String>>,
    }

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

      constructor (public items: Seq<str>, public numbers: Seq<int32>, public nested_items: Seq<Seq<str>>) {
      }

    }
    ");
}

#[test]
fn struct_with_option_field() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct MyStruct {
        optional_string: Option<String>,
        optional_number: Option<i32>,
        optional_bool: Option<bool>,
    }

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type int32 = number;
    type Optional<T> = T | null;
    type str = string;
    export class MyStruct {

      constructor (public optional_string: Optional<str>, public optional_number: Optional<int32>, public optional_bool: Optional<bool>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type int32 = number;
    type str = string;
    export class MyStruct {

      constructor (public string_to_int: Map<str,int32>, public int_to_bool: Map<int32,bool>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type int32 = number;
    type Optional<T> = T | null;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

      constructor (public optional_list: Optional<Seq<str>>, public list_of_optionals: Seq<Optional<int32>>, public map_to_list: Map<str,Seq<bool>>, public optional_map: Optional<Map<str,int32>>, public complex: Seq<Optional<Map<str,Seq<bool>>>>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type ListTuple<T extends any[]> = Tuple<T>[];
    type str = string;
    type uint8 = number;
    export class MyStruct {

      constructor (public fixed_array: ListTuple<[int32]>, public byte_array: ListTuple<[uint8]>, public string_array: ListTuple<[str]>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type bool = boolean;
    type int32 = number;
    type str = string;
    export class MyStruct {

      constructor (public string_to_int: Map<str,int32>, public int_to_bool: Map<int32,bool>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

      constructor (public string_set: Seq<str>, public int_set: Seq<int32>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

      constructor (public string_set: Seq<str>, public int_set: Seq<int32>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export class MyStruct {

      constructor (public boxed_string: str, public boxed_int: int32) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export class MyStruct {

      constructor (public rc_string: str, public rc_int: int32) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type str = string;
    export class MyStruct {

      constructor (public arc_string: str, public arc_int: int32) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type int32 = number;
    type ListTuple<T extends any[]> = Tuple<T>[];
    type Optional<T> = T | null;
    type Seq<T> = T[];
    type str = string;
    export class MyStruct {

      constructor (public vec_of_sets: Seq<Seq<str>>, public optional_btree: Optional<Map<str,int32>>, public boxed_vec: Seq<str>, public arc_option: Optional<str>, public array_of_boxes: ListTuple<[int32]>) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type bytes = Uint8Array;
    type str = string;
    export class MyStruct {

      constructor (public data: bytes, public name: str, public header: bytes) {
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

    let actual = emit::<MyStruct>(Encoding::None);
    insta::assert_snapshot!(actual, @"
    type bytes = Uint8Array;
    type Optional<T> = T | null;
    type Seq<T> = T[];
    type str = string;
    type uint8 = number;
    export class MyStruct {

      constructor (public data: bytes, public name: str, public header: bytes, public optional_bytes: Optional<Seq<uint8>>) {
      }

    }
    ");
}

#[test]
fn type_in_root_and_named_namespace() {
    #[derive(Facet)]
    struct Child {
        value: String,
    }

    mod other {
        use facet::Facet;

        #[derive(Facet)]
        #[facet(namespace = "other")]
        pub struct Child {
            value: i32,
        }
    }

    #[derive(Facet)]
    struct Parent {
        child: Child,
        other_child: other::Child,
    }

    let (other, root) = emit_two_modules!(CodeGenerator, Parent, "root");
    insta::assert_snapshot!(other, @"
    type int32 = number;
    export class Child {

      constructor (public value: int32) {
      }

    }
    ");
    insta::assert_snapshot!(root, @r#"
    import * as Other from "../other";
    type str = string;
    export class Child {

      constructor (public value: str) {
      }

    }
    export class Parent {

      constructor (public child: Child, public other_child: Other.Child) {
      }

    }
    "#);
}
