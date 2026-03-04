#![allow(clippy::too_many_lines)]
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use facet::Facet;

use super::{Encoding, tests::emit};

fn assert_contains_all(actual: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            actual.contains(snippet),
            "Missing snippet `{snippet}` in output:\n{actual}"
        );
    }
}

fn assert_struct_serde(actual: &str, type_name: &str) {
    assert_contains_all(
        actual,
        &[
            "public serialize(serializer: Serializer): void",
            &format!("static deserialize(deserializer: Deserializer): {type_name}"),
        ],
    );
}

#[test]
fn unit_struct_1() {
    #[derive(Facet)]
    struct UnitStruct;

    let actual = emit::<UnitStruct>(Encoding::Json);
    assert_struct_serde(&actual, "UnitStruct");
    assert_contains_all(
        &actual,
        &["export class UnitStruct", "return new UnitStruct();"],
    );
}

#[test]
fn unit_struct_2() {
    #[derive(Facet)]
    struct UnitStruct {}

    let actual = emit::<UnitStruct>(Encoding::Json);
    assert_struct_serde(&actual, "UnitStruct");
    assert_contains_all(
        &actual,
        &["export class UnitStruct", "return new UnitStruct();"],
    );
}

#[test]
fn newtype_struct() {
    #[derive(Facet)]
    struct NewType(String);

    let actual = emit::<NewType>(Encoding::Json);
    assert_struct_serde(&actual, "NewType");
    assert_contains_all(
        &actual,
        &[
            "constructor (public value: str)",
            "serializer.serializeStr(this.value);",
            "const value = deserializer.deserializeStr();",
        ],
    );
}

#[test]
fn tuple_struct() {
    #[derive(Facet)]
    struct TupleStruct(String, i32);

    let actual = emit::<TupleStruct>(Encoding::Json);
    assert_struct_serde(&actual, "TupleStruct");
    assert_contains_all(
        &actual,
        &[
            "constructor (public field0: str, public field1: int32)",
            "serializer.serializeStr(this.field0);",
            "serializer.serializeI32(this.field1);",
        ],
    );
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

    let actual = emit::<StructWithFields>(Encoding::Json);
    assert_struct_serde(&actual, "StructWithFields");
    assert_contains_all(
        &actual,
        &[
            "serializer.serializeUnit(this.unit);",
            "serializer.serializeBool(this.bool);",
            "serializer.serializeU128(this.u128);",
            "serializer.serializeF64(this.f64);",
            "serializer.serializeChar(this.char);",
            "const i128 = deserializer.deserializeI128();",
        ],
    );
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

    let actual = emit::<Outer>(Encoding::Json);
    assert_struct_serde(&actual, "Outer");
    assert_contains_all(
        &actual,
        &[
            "this.one.serialize(serializer);",
            "this.two.serialize(serializer);",
            "this.three.serialize(serializer);",
            "const one = Inner1.deserialize(deserializer);",
        ],
    );
}

#[test]
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public one: Tuple<[str, int32]>",
            "Helpers.serializeTuple",
            "const one = Helpers.deserializeTuple",
        ],
    );
}

#[test]
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public one: Tuple<[str, int32, uint16]>",
            "Helpers.serializeTuple",
            "const one = Helpers.deserializeTuple",
        ],
    );
}

#[test]
fn struct_with_field_that_is_a_4_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16, f32),
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public one: Tuple<[str, int32, uint16, float32]>",
            "Helpers.serializeTuple",
            "const one = Helpers.deserializeTuple",
        ],
    );
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

    let actual = emit::<EnumWithUnitVariants>(Encoding::Json);
    assert_contains_all(
        &actual,
        &[
            "export abstract class EnumWithUnitVariants",
            "case 0: return EnumWithUnitVariantsVariantVariant1.load(deserializer);",
            "case 2: return EnumWithUnitVariantsVariantVariant3.load(deserializer);",
            "serializer.serializeVariantIndex(0);",
        ],
    );
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

    let actual = emit::<MyEnum>(Encoding::Json);
    assert_contains_all(
        &actual,
        &[
            "export class MyEnumVariantVariant1 extends MyEnum",
            "serializer.serializeVariantIndex(0);",
            "return new MyEnumVariantVariant1();",
        ],
    );
}

#[test]
fn enum_with_1_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
    }

    let actual = emit::<MyEnum>(Encoding::Json);
    assert_contains_all(
        &actual,
        &[
            "export class MyEnumVariantVariant1 extends MyEnum",
            "constructor (public value: str)",
            "serializer.serializeStr(this.value);",
        ],
    );
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

    let actual = emit::<MyEnum>(Encoding::Json);
    assert_contains_all(
        &actual,
        &[
            "case 0: return MyEnumVariantVariant1.load(deserializer);",
            "case 1: return MyEnumVariantVariant2.load(deserializer);",
            "serializer.serializeI32(this.value);",
        ],
    );
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

    let actual = emit::<MyEnum>(Encoding::Json);
    assert_contains_all(
        &actual,
        &[
            "export class MyEnumVariantVariant1 extends MyEnum",
            "export class MyEnumVariantVariant2 extends MyEnum",
            "serializer.serializeBool(this.field0);",
            "serializer.serializeF64(this.field1);",
            "serializer.serializeU8(this.field2);",
        ],
    );
}

#[test]
fn enum_with_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 { field1: String, field2: i32 },
    }

    let actual = emit::<MyEnum>(Encoding::Json);
    assert_contains_all(
        &actual,
        &[
            "constructor (public field1: str, public field2: int32)",
            "serializer.serializeStr(this.field1);",
            "serializer.serializeI32(this.field2);",
        ],
    );
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

    let actual = emit::<MyEnum>(Encoding::Json);
    assert_contains_all(
        &actual,
        &[
            "case 0: return MyEnumVariantUnit.load(deserializer);",
            "case 3: return MyEnumVariantStruct.load(deserializer);",
            "serializer.serializeVariantIndex(3);",
        ],
    );
}

#[test]
fn struct_with_vec_field() {
    #[derive(Facet)]
    struct MyStruct {
        items: Vec<String>,
        numbers: Vec<i32>,
        nested_items: Vec<Vec<String>>,
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public items: Seq<str>",
            "public nested_items: Seq<Seq<str>>",
            "Helpers.serializeVectorVectorStr(this.nested_items, serializer);",
        ],
    );
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

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public optional_string: Optional<str>",
            "public optional_number: Optional<int32>",
            "public optional_bool: Optional<bool>",
            "Helpers.serializeOptionStr(this.optional_string, serializer);",
        ],
    );
}

#[test]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        int_to_bool: HashMap<i32, bool>,
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public string_to_int: Map<str,int32>",
            "public int_to_bool: Map<int32,bool>",
            "Helpers.serializeMapStrToI32(this.string_to_int, serializer);",
        ],
    );
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

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public optional_list: Optional<Seq<str>>",
            "public complex: Seq<Optional<Map<str,Seq<bool>>>>",
            "Helpers.serializeVectorOptionMapStrToVectorBool(this.complex, serializer);",
        ],
    );
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

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public fixed_array: ListTuple<[int32]>",
            "public byte_array: ListTuple<[uint8]>",
            "Helpers.serializeArray5I32Array(this.fixed_array, serializer);",
        ],
    );
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public string_to_int: Map<str,int32>",
            "public int_to_bool: Map<int32,bool>",
            "Helpers.serializeMapI32ToBool(this.int_to_bool, serializer);",
        ],
    );
}

#[test]
fn struct_with_hashset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: HashSet<String>,
        int_set: HashSet<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public string_set: Seq<str>",
            "public int_set: Seq<int32>",
            "Helpers.serializeSetI32(this.int_set, serializer);",
        ],
    );
}

#[test]
fn struct_with_btreeset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public string_set: Seq<str>",
            "public int_set: Seq<int32>",
            "Helpers.serializeSetStr(this.string_set, serializer);",
        ],
    );
}

#[test]
fn struct_with_box_field() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        boxed_string: Box<String>,
        boxed_int: Box<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public boxed_string: str",
            "public boxed_int: int32",
            "serializer.serializeI32(this.boxed_int);",
        ],
    );
}

#[test]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: Rc<String>,
        rc_int: Rc<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public rc_string: str",
            "public rc_int: int32",
            "const rc_string = deserializer.deserializeStr();",
        ],
    );
}

#[test]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: Arc<String>,
        arc_int: Arc<i32>,
    }

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public arc_string: str",
            "public arc_int: int32",
            "const arc_int = deserializer.deserializeI32();",
        ],
    );
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

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public vec_of_sets: Seq<Seq<str>>",
            "public optional_btree: Optional<Map<str,int32>>",
            "public boxed_vec: Seq<str>",
            "public arc_option: Optional<str>",
            "public array_of_boxes: ListTuple<[int32]>",
        ],
    );
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

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "type bytes = Uint8Array;",
            "public data: bytes",
            "public header: bytes",
            "serializer.serializeBytes(this.data);",
            "serializer.serializeBytes(this.header);",
        ],
    );
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

    let actual = emit::<MyStruct>(Encoding::Json);
    assert_struct_serde(&actual, "MyStruct");
    assert_contains_all(
        &actual,
        &[
            "public data: bytes",
            "public header: bytes",
            "public optional_bytes: Optional<Seq<uint8>>",
            "serializer.serializeBytes(this.data);",
        ],
    );
}
