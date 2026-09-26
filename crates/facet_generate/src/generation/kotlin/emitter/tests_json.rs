//! Snapshot tests for the Kotlin emitter — **JSON encoding**.
//!
//! Mirrors the structure of [`tests`](super::tests) but uses `JsonPlugin`
//! so that every generated type carries `kotlinx.serialization` annotations.
//!
//! A struct whose fields kotlinx already writes the way `serde_json` does is
//! left to the `kotlinx.serialization` compiler plugin, with `@SerialName` on
//! each property; every other type gets a nested `JsonSerializer`. These tests
//! verify the annotations on each type and property, and those serializers.

#![allow(clippy::too_many_lines)]
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use crate::{self as fg, generation::json::JsonPlugin};
use facet::Facet;

use super::*;
use crate::emit;

#[test]
fn unit_struct_1() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    /// line 1
    /// line 2
    @Serializable(with = UnitStruct.JsonSerializer::class)
    data object UnitStruct {
        object JsonSerializer : JsonElementSerializer<UnitStruct>(
            "UnitStruct",
            toJson = { value ->
                unit()
            },
            fromJson = { element ->
                unit(element)
                UnitStruct
            },
        )
    }
    "#);
}

#[test]
fn unit_struct_2() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct {}

    let actual = emit!(UnitStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    /// line 1
    /// line 2
    @Serializable(with = UnitStruct.JsonSerializer::class)
    data object UnitStruct {
        object JsonSerializer : JsonElementSerializer<UnitStruct>(
            "UnitStruct",
            toJson = { value ->
                unit()
            },
            fromJson = { element ->
                unit(element)
                UnitStruct
            },
        )
    }
    "#);
}

#[test]
fn newtype_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct NewType(String);

    let actual = emit!(NewType as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    @Serializable(with = NewType.JsonSerializer::class)
    data class NewType(
        val value: String,
    ) {
        object JsonSerializer : JsonNewTypeSerializer<NewType, String>(
            serializer = { String.serializer() },
            wrap = { NewType(it) },
            unwrap = { it.value },
        )
    }
    ");
}

#[test]
fn tuple_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    /// line 1
    /// line 2
    @Serializable(with = TupleStruct.JsonSerializer::class)
    data class TupleStruct(
        val field0: String,
        val field1: Int,
    ) {
        object JsonSerializer : JsonElementSerializer<TupleStruct>(
            "TupleStruct",
            toJson = { value ->
                array(
                    encode(String.serializer(), value.field0),
                    encode(Int.serializer(), value.field1),
                )
            },
            fromJson = { element ->
                val items = tuple(element, 2)
                TupleStruct(
                    decode(String.serializer(), items[0]),
                    decode(Int.serializer(), items[1]),
                )
            },
        )
    }
    "#);
}

#[test]
fn struct_with_fields_of_primitive_types() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct StructWithFields {
        /// unit type
        unit: (),
        /// boolean
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

    let actual = emit!(StructWithFields as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    /// line 1
    /// line 2
    @Serializable(with = StructWithFields.JsonSerializer::class)
    data class StructWithFields(
        /// unit type
        val unit: Unit,
        /// boolean
        val bool: Boolean,
        val i8: Byte,
        val i16: Short,
        val i32: Int,
        val i64: Long,
        val i128: BigInteger,
        val u8: UByte,
        val u16: UShort,
        val u32: UInt,
        val u64: ULong,
        val u128: BigInteger,
        val f32: Float,
        val f64: Double,
        val char: String,
        val string: String,
    ) {
        object JsonSerializer : JsonElementSerializer<StructWithFields>(
            "StructWithFields",
            toJson = { value ->
                obj(
                    "unit" to encode(JsonUnitSerializer, value.unit),
                    "bool" to encode(Boolean.serializer(), value.bool),
                    "i8" to encode(Byte.serializer(), value.i8),
                    "i16" to encode(Short.serializer(), value.i16),
                    "i32" to encode(Int.serializer(), value.i32),
                    "i64" to encode(Long.serializer(), value.i64),
                    "i128" to encode(BigIntegerSerializer, value.i128),
                    "u8" to encode(UByte.serializer(), value.u8),
                    "u16" to encode(UShort.serializer(), value.u16),
                    "u32" to encode(UInt.serializer(), value.u32),
                    "u64" to encode(ULong.serializer(), value.u64),
                    "u128" to encode(BigIntegerSerializer, value.u128),
                    "f32" to encode(Float.serializer(), value.f32),
                    "f64" to encode(Double.serializer(), value.f64),
                    "char" to encode(String.serializer(), value.char),
                    "string" to encode(String.serializer(), value.string),
                )
            },
            fromJson = { element ->
                val fields = fields(element)
                StructWithFields(
                    unit = decode(JsonUnitSerializer, fields.required("unit")),
                    bool = decode(Boolean.serializer(), fields.required("bool")),
                    i8 = decode(Byte.serializer(), fields.required("i8")),
                    i16 = decode(Short.serializer(), fields.required("i16")),
                    i32 = decode(Int.serializer(), fields.required("i32")),
                    i64 = decode(Long.serializer(), fields.required("i64")),
                    i128 = decode(BigIntegerSerializer, fields.required("i128")),
                    u8 = decode(UByte.serializer(), fields.required("u8")),
                    u16 = decode(UShort.serializer(), fields.required("u16")),
                    u32 = decode(UInt.serializer(), fields.required("u32")),
                    u64 = decode(ULong.serializer(), fields.required("u64")),
                    u128 = decode(BigIntegerSerializer, fields.required("u128")),
                    f32 = decode(Float.serializer(), fields.required("f32")),
                    f64 = decode(Double.serializer(), fields.required("f64")),
                    char = decode(String.serializer(), fields.required("char")),
                    string = decode(String.serializer(), fields.required("string")),
                )
            },
        )
    }
    "#);
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

    let actual = emit!(Outer as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("Inner1")
    data class Inner1(
        @SerialName("field1") val field1: String,
    )

    @Serializable(with = Inner2.JsonSerializer::class)
    data class Inner2(
        val value: String,
    ) {
        object JsonSerializer : JsonNewTypeSerializer<Inner2, String>(
            serializer = { String.serializer() },
            wrap = { Inner2(it) },
            unwrap = { it.value },
        )
    }

    @Serializable(with = Inner3.JsonSerializer::class)
    data class Inner3(
        val field0: String,
        val field1: Int,
    ) {
        object JsonSerializer : JsonElementSerializer<Inner3>(
            "Inner3",
            toJson = { value ->
                array(
                    encode(String.serializer(), value.field0),
                    encode(Int.serializer(), value.field1),
                )
            },
            fromJson = { element ->
                val items = tuple(element, 2)
                Inner3(
                    decode(String.serializer(), items[0]),
                    decode(Int.serializer(), items[1]),
                )
            },
        )
    }

    @Serializable
    @SerialName("Outer")
    data class Outer(
        @SerialName("one") val one: Inner1,
        @SerialName("two") val two: Inner2,
        @SerialName("three") val three: Inner3,
    )
    "#);
}

#[test]
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyStruct.JsonSerializer::class)
    data class MyStruct(
        val one: Pair<String, Int>,
    ) {
        object JsonSerializer : JsonElementSerializer<MyStruct>(
            "MyStruct",
            toJson = { value ->
                obj(
                    "one" to encode(JsonPairSerializer(String.serializer(), Int.serializer()), value.one),
                )
            },
            fromJson = { element ->
                val fields = fields(element)
                MyStruct(
                    one = decode(JsonPairSerializer(String.serializer(), Int.serializer()), fields.required("one")),
                )
            },
        )
    }
    "#);
}

#[test]
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyStruct.JsonSerializer::class)
    data class MyStruct(
        val one: Triple<String, Int, UShort>,
    ) {
        object JsonSerializer : JsonElementSerializer<MyStruct>(
            "MyStruct",
            toJson = { value ->
                obj(
                    "one" to encode(JsonTripleSerializer(String.serializer(), Int.serializer(), UShort.serializer()), value.one),
                )
            },
            fromJson = { element ->
                val fields = fields(element)
                MyStruct(
                    one = decode(JsonTripleSerializer(String.serializer(), Int.serializer(), UShort.serializer()), fields.required("one")),
                )
            },
        )
    }
    "#);
}

#[test]
fn struct_with_field_that_is_a_4_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16, f32),
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyStruct.JsonSerializer::class)
    data class MyStruct(
        val one: Tuple4<String, Int, UShort, Float>,
    ) {
        object JsonSerializer : JsonElementSerializer<MyStruct>(
            "MyStruct",
            toJson = { value ->
                obj(
                    "one" to encode(JsonTuple4Serializer(String.serializer(), Int.serializer(), UShort.serializer(), Float.serializer()), value.one),
                )
            },
            fromJson = { element ->
                val fields = fields(element)
                MyStruct(
                    one = decode(JsonTuple4Serializer(String.serializer(), Int.serializer(), UShort.serializer(), Float.serializer()), fields.required("one")),
                )
            },
        )
    }
    "#);
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

    let actual = emit!(EnumWithUnitVariants as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    /// line one
    /// line two
    @Serializable
    @SerialName("EnumWithUnitVariants")
    enum class EnumWithUnitVariants {
        /// variant one
        @SerialName("Variant1") VARIANT1,
        /// variant two
        @SerialName("Variant2") VARIANT2,
        /// variant three
        @SerialName("Variant3") VARIANT3;

        val serialName: String
            get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
    }
    "#);
}

#[test]
fn enum_with_unit_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 {},
    }

    let actual = emit!(MyEnum as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyEnum")
    enum class MyEnum {
        @SerialName("Variant1") VARIANT1;

        val serialName: String
            get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
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

    let actual = emit!(MyEnum as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyEnum.JsonSerializer::class)
    sealed interface MyEnum {
        data class Variant1(
            val value: String,
        ) : MyEnum

        object JsonSerializer : JsonElementSerializer<MyEnum>(
            "MyEnum",
            toJson = { value ->
                when (value) {
                    is Variant1 -> variant("Variant1", encode(String.serializer(), value.value))
                }
            },
            fromJson = { element ->
                val (tag, content) = variant(element)
                when (tag) {
                    "Variant1" -> Variant1(decode(String.serializer(), payload(content)))
                    else -> unknownVariant(tag)
                }
            },
        )
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

    let actual = emit!(MyEnum as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyEnum.JsonSerializer::class)
    sealed interface MyEnum {
        data class Variant1(
            val value: String,
        ) : MyEnum

        data class Variant2(
            val value: Int,
        ) : MyEnum

        object JsonSerializer : JsonElementSerializer<MyEnum>(
            "MyEnum",
            toJson = { value ->
                when (value) {
                    is Variant1 -> variant("Variant1", encode(String.serializer(), value.value))
                    is Variant2 -> variant("Variant2", encode(Int.serializer(), value.value))
                }
            },
            fromJson = { element ->
                val (tag, content) = variant(element)
                when (tag) {
                    "Variant1" -> Variant1(decode(String.serializer(), payload(content)))
                    "Variant2" -> Variant2(decode(Int.serializer(), payload(content)))
                    else -> unknownVariant(tag)
                }
            },
        )
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

    let actual = emit!(MyEnum as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyEnum.JsonSerializer::class)
    sealed interface MyEnum {
        data class Variant1(
            val field0: String,
            val field1: Int,
        ) : MyEnum

        data class Variant2(
            val field0: Boolean,
            val field1: Double,
            val field2: UByte,
        ) : MyEnum

        object JsonSerializer : JsonElementSerializer<MyEnum>(
            "MyEnum",
            toJson = { value ->
                when (value) {
                    is Variant1 -> variant(
                        "Variant1",
                        array(
                            encode(String.serializer(), value.field0),
                            encode(Int.serializer(), value.field1),
                        ),
                    )
                    is Variant2 -> variant(
                        "Variant2",
                        array(
                            encode(Boolean.serializer(), value.field0),
                            encode(Double.serializer(), value.field1),
                            encode(UByte.serializer(), value.field2),
                        ),
                    )
                }
            },
            fromJson = { element ->
                val (tag, content) = variant(element)
                when (tag) {
                    "Variant1" -> {
                        val items = tuple(content, 2)
                        Variant1(
                            decode(String.serializer(), items[0]),
                            decode(Int.serializer(), items[1]),
                        )
                    }
                    "Variant2" -> {
                        val items = tuple(content, 3)
                        Variant2(
                            decode(Boolean.serializer(), items[0]),
                            decode(Double.serializer(), items[1]),
                            decode(UByte.serializer(), items[2]),
                        )
                    }
                    else -> unknownVariant(tag)
                }
            },
        )
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

    let actual = emit!(MyEnum as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyEnum.JsonSerializer::class)
    sealed interface MyEnum {
        data class Variant1(
            val field1: String,
            val field2: Int,
        ) : MyEnum

        object JsonSerializer : JsonElementSerializer<MyEnum>(
            "MyEnum",
            toJson = { value ->
                when (value) {
                    is Variant1 -> variant(
                        "Variant1",
                        obj(
                            "field1" to encode(String.serializer(), value.field1),
                            "field2" to encode(Int.serializer(), value.field2),
                        ),
                    )
                }
            },
            fromJson = { element ->
                val (tag, content) = variant(element)
                when (tag) {
                    "Variant1" -> {
                        val fields = fields(content)
                        Variant1(
                            field1 = decode(String.serializer(), fields.required("field1")),
                            field2 = decode(Int.serializer(), fields.required("field2")),
                        )
                    }
                    else -> unknownVariant(tag)
                }
            },
        )
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

    let actual = emit!(MyEnum as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyEnum.JsonSerializer::class)
    sealed interface MyEnum {
        data object Unit: MyEnum

        data class NewType(
            val value: String,
        ) : MyEnum

        data class Tuple(
            val field0: String,
            val field1: Int,
        ) : MyEnum

        data class Struct(
            val field: Boolean,
        ) : MyEnum

        object JsonSerializer : JsonElementSerializer<MyEnum>(
            "MyEnum",
            toJson = { value ->
                when (value) {
                    is Unit -> variant("Unit")
                    is NewType -> variant("NewType", encode(String.serializer(), value.value))
                    is Tuple -> variant(
                        "Tuple",
                        array(
                            encode(String.serializer(), value.field0),
                            encode(Int.serializer(), value.field1),
                        ),
                    )
                    is Struct -> variant(
                        "Struct",
                        obj(
                            "field" to encode(Boolean.serializer(), value.field),
                        ),
                    )
                }
            },
            fromJson = { element ->
                val (tag, content) = variant(element)
                when (tag) {
                    "Unit" -> Unit
                    "NewType" -> NewType(decode(String.serializer(), payload(content)))
                    "Tuple" -> {
                        val items = tuple(content, 2)
                        Tuple(
                            decode(String.serializer(), items[0]),
                            decode(Int.serializer(), items[1]),
                        )
                    }
                    "Struct" -> {
                        val fields = fields(content)
                        Struct(
                            field = decode(Boolean.serializer(), fields.required("field")),
                        )
                    }
                    else -> unknownVariant(tag)
                }
            },
        )
    }
    "#);
}

#[test]
fn struct_with_vec_field() {
    #[derive(Facet)]
    struct MyStruct {
        items: Vec<String>,
        numbers: Vec<i32>,
        nested_items: Vec<Vec<String>>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("items") val items: List<String>,
        @SerialName("numbers") val numbers: List<Int>,
        @SerialName("nested_items") val nestedItems: List<List<String>>,
    )
    "#);
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

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @OptIn(ExperimentalSerializationApi::class)
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("optional_string") @EncodeDefault val optionalString: String? = null,
        @SerialName("optional_number") @EncodeDefault val optionalNumber: Int? = null,
        @SerialName("optional_bool") @EncodeDefault val optionalBool: Boolean? = null,
    )
    "#);
}

#[test]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        int_to_bool: HashMap<i32, bool>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("string_to_int") val stringToInt: Map<String, Int>,
        @SerialName("int_to_bool") val intToBool: Map<Int, Boolean>,
    )
    "#);
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

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @OptIn(ExperimentalSerializationApi::class)
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("optional_list") @EncodeDefault val optionalList: List<String>? = null,
        @SerialName("list_of_optionals") val listOfOptionals: List<Int?>,
        @SerialName("map_to_list") val mapToList: Map<String, List<Boolean>>,
        @SerialName("optional_map") @EncodeDefault val optionalMap: Map<String, Int>? = null,
        @SerialName("complex") val complex: List<Map<String, List<Boolean>>?>,
    )
    "#);
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

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("fixed_array") val fixedArray: List<Int>,
        @SerialName("byte_array") val byteArray: List<UByte>,
        @SerialName("string_array") val stringArray: List<String>,
    )
    "#);
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("string_to_int") val stringToInt: Map<String, Int>,
        @SerialName("int_to_bool") val intToBool: Map<Int, Boolean>,
    )
    "#);
}

#[test]
fn struct_with_hashset_field() {
    // NOTE: HashSet<T> now maps to Set<T> in Kotlin with the new Format::Set variant.
    // This preserves the uniqueness constraint and provides better type safety.
    #[derive(Facet)]
    struct MyStruct {
        string_set: HashSet<String>,
        int_set: HashSet<i32>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("string_set") val stringSet: Set<String>,
        @SerialName("int_set") val intSet: Set<Int>,
    )
    "#);
}

#[test]
fn struct_with_btreeset_field() {
    // NOTE: BTreeSet<T> now maps to Set<T> in Kotlin with the new Format::Set variant.
    // This preserves the uniqueness constraint and provides better type safety.
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("string_set") val stringSet: Set<String>,
        @SerialName("int_set") val intSet: Set<Int>,
    )
    "#);
}

#[test]
fn struct_with_box_field() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        boxed_string: Box<String>,
        boxed_int: Box<i32>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("boxed_string") val boxedString: String,
        @SerialName("boxed_int") val boxedInt: Int,
    )
    "#);
}

#[test]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: Rc<String>,
        rc_int: Rc<i32>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("rc_string") val rcString: String,
        @SerialName("rc_int") val rcInt: Int,
    )
    "#);
}

#[test]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: Arc<String>,
        arc_int: Arc<i32>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("arc_string") val arcString: String,
        @SerialName("arc_int") val arcInt: Int,
    )
    "#);
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

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @OptIn(ExperimentalSerializationApi::class)
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("vec_of_sets") val vecOfSets: List<Set<String>>,
        @SerialName("optional_btree") @EncodeDefault val optionalBtree: Map<String, Int>? = null,
        @SerialName("boxed_vec") val boxedVec: List<String>,
        @SerialName("arc_option") @EncodeDefault val arcOption: String? = null,
        @SerialName("array_of_boxes") val arrayOfBoxes: List<Int>,
    )
    "#);
}

#[test]
fn struct_with_bytes_field() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(fg::bytes)]
        data: Vec<u8>,
        name: String,
        #[facet(fg::bytes)]
        header: Vec<u8>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("data") val data: Bytes,
        @SerialName("name") val name: String,
        @SerialName("header") val header: Bytes,
    )
    "#);
}

#[test]
fn struct_with_bytes_field_and_slice() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        #[facet(fg::bytes)]
        data: &'a [u8],
        name: String,
        #[facet(fg::bytes)]
        header: Vec<u8>,
        optional_bytes: Option<Vec<u8>>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @OptIn(ExperimentalSerializationApi::class)
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("data") val data: Bytes,
        @SerialName("name") val name: String,
        @SerialName("header") val header: Bytes,
        @SerialName("optional_bytes") @EncodeDefault val optionalBytes: List<UByte>? = null,
    )
    "#);
}

#[test]
fn keyword_fields_struct() {
    #[derive(Facet)]
    #[allow(clippy::struct_excessive_bools)]
    struct KeywordFields {
        r#default: String,
        r#in: i32,
        object: bool,
        import: bool,
    }

    let actual = emit!(KeywordFields as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable
    @SerialName("KeywordFields")
    data class KeywordFields(
        @SerialName("default") val default: String,
        @SerialName("in") val `in`: Int,
        @SerialName("object") val `object`: Boolean,
        @SerialName("import") val import: Boolean,
    )
    "#);
}

#[test]
fn keyword_enum() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum KeywordEnum {
        Default,
        Switch(String),
        Where { r#in: i32, r#default: String },
    }

    let actual = emit!(KeywordEnum as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = KeywordEnum.JsonSerializer::class)
    sealed interface KeywordEnum {
        data object Default: KeywordEnum

        data class Switch(
            val value: String,
        ) : KeywordEnum

        data class Where(
            val `in`: Int,
            val default: String,
        ) : KeywordEnum

        object JsonSerializer : JsonElementSerializer<KeywordEnum>(
            "KeywordEnum",
            toJson = { value ->
                when (value) {
                    is Default -> variant("Default")
                    is Switch -> variant("Switch", encode(String.serializer(), value.value))
                    is Where -> variant(
                        "Where",
                        obj(
                            "in" to encode(Int.serializer(), value.`in`),
                            "default" to encode(String.serializer(), value.default),
                        ),
                    )
                }
            },
            fromJson = { element ->
                val (tag, content) = variant(element)
                when (tag) {
                    "Default" -> Default
                    "Switch" -> Switch(decode(String.serializer(), payload(content)))
                    "Where" -> {
                        val fields = fields(content)
                        Where(
                            `in` = decode(Int.serializer(), fields.required("in")),
                            default = decode(String.serializer(), fields.required("default")),
                        )
                    }
                    else -> unknownVariant(tag)
                }
            },
        )
    }
    "#);
}

#[test]
fn struct_with_renamed_fields() {
    #[derive(Facet)]
    #[facet(rename_all = "camelCase")]
    struct MyStruct {
        snake_case: u8,
        #[facet(rename = "$ref")]
        reference: String,
        #[facet(rename = "with-dash")]
        maybe: Option<u64>,
    }

    let actual = emit!(MyStruct as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @OptIn(ExperimentalSerializationApi::class)
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        @SerialName("snakeCase") val snakeCase: UByte,
        @SerialName("\$ref") val ref: String,
        @SerialName("with-dash") @EncodeDefault val withDash: ULong? = null,
    )
    "#);
}

#[test]
fn enum_internally_tagged() {
    #[derive(Facet)]
    struct Point {
        x: i32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[facet(tag = "type")]
    #[allow(unused)]
    enum MyEnum {
        Unit,
        Wrapped(Point),
        Struct { x: i32 },
    }

    let actual = emit!(MyEnum as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyEnum.JsonSerializer::class)
    sealed interface MyEnum {
        data object Unit: MyEnum

        data class Wrapped(
            val value: Point,
        ) : MyEnum

        data class Struct(
            val x: Int,
        ) : MyEnum

        object JsonSerializer : JsonElementSerializer<MyEnum>(
            "MyEnum",
            tag = "type",
            toJson = { value ->
                when (value) {
                    is Unit -> variant("Unit")
                    is Wrapped -> variant("Wrapped", encode(Point.serializer(), value.value))
                    is Struct -> variant(
                        "Struct",
                        obj(
                            "x" to encode(Int.serializer(), value.x),
                        ),
                    )
                }
            },
            fromJson = { element ->
                val (tag, content) = variant(element)
                when (tag) {
                    "Unit" -> Unit
                    "Wrapped" -> Wrapped(decode(Point.serializer(), payload(content)))
                    "Struct" -> {
                        val fields = fields(content)
                        Struct(
                            x = decode(Int.serializer(), fields.required("x")),
                        )
                    }
                    else -> unknownVariant(tag)
                }
            },
        )
    }

    @Serializable
    @SerialName("Point")
    data class Point(
        @SerialName("x") val x: Int,
    )
    "#);
}

#[test]
fn enum_adjacently_tagged() {
    #[derive(Facet)]
    #[repr(C)]
    #[facet(tag = "t", content = "c")]
    #[allow(unused)]
    enum MyEnum {
        Unit,
        NewType(Option<u8>),
        Tuple(u8, String),
    }

    let actual = emit!(MyEnum as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = MyEnum.JsonSerializer::class)
    sealed interface MyEnum {
        data object Unit: MyEnum

        data class NewType(
            val value: UByte? = null,
        ) : MyEnum

        data class Tuple(
            val field0: UByte,
            val field1: String,
        ) : MyEnum

        object JsonSerializer : JsonElementSerializer<MyEnum>(
            "MyEnum",
            tag = "t",
            content = "c",
            toJson = { value ->
                when (value) {
                    is Unit -> variant("Unit")
                    is NewType -> variant("NewType", encode(UByte.serializer().nullable, value.value))
                    is Tuple -> variant(
                        "Tuple",
                        array(
                            encode(UByte.serializer(), value.field0),
                            encode(String.serializer(), value.field1),
                        ),
                    )
                }
            },
            fromJson = { element ->
                val (tag, content) = variant(element)
                when (tag) {
                    "Unit" -> Unit
                    "NewType" -> NewType(decode(UByte.serializer().nullable, payload(content)))
                    "Tuple" -> {
                        val items = tuple(content, 2)
                        Tuple(
                            decode(UByte.serializer(), items[0]),
                            decode(String.serializer(), items[1]),
                        )
                    }
                    else -> unknownVariant(tag)
                }
            },
        )
    }
    "#);
}

#[test]
fn enum_with_unit_variants_internally_tagged() {
    #[derive(Facet)]
    #[repr(C)]
    #[facet(tag = "kind")]
    #[allow(unused)]
    enum Mode {
        Fast,
        #[facet(rename = "SLOW")]
        Slow,
    }

    let actual = emit!(Mode as Kotlin with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    @Serializable(with = Mode.JsonSerializer::class)
    enum class Mode {
        @SerialName("Fast") FAST,
        @SerialName("SLOW") SLOW;

        val serialName: String
            get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value

        object JsonSerializer : JsonElementSerializer<Mode>(
            "Mode",
            tag = "kind",
            toJson = { value ->
                when (value) {
                    FAST -> variant("Fast")
                    SLOW -> variant("SLOW")
                }
            },
            fromJson = { element ->
                val (tag, _) = variant(element)
                when (tag) {
                    "Fast" -> FAST
                    "SLOW" -> SLOW
                    else -> unknownVariant(tag)
                }
            },
        )
    }
    "#);
}
