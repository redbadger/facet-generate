#![allow(clippy::too_many_lines)]
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use facet::Facet;

use super::*;
use crate::emit;

#[test]
fn unit_struct_1() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    @Serializable
    @SerialName("UnitStruct")
    data object UnitStruct
    "#);
}

#[test]
fn unit_struct_2() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct {}

    let actual = emit!(UnitStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    @Serializable
    @SerialName("UnitStruct")
    data object UnitStruct
    "#);
}

#[test]
fn newtype_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct NewType(String);

    let actual = emit!(NewType as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    @Serializable
    @SerialName("NewType")
    data class NewType(
        val value: String,
    )
    "#);
}

#[test]
fn tuple_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    @Serializable
    @SerialName("TupleStruct")
    data class TupleStruct(
        val field0: String,
        val field1: Int,
    )
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

    let actual = emit!(StructWithFields as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    @Serializable
    @SerialName("StructWithFields")
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
    )
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

    let actual = emit!(Outer as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("Inner1")
    data class Inner1(
        val field1: String,
    )

    @Serializable
    @SerialName("Inner2")
    data class Inner2(
        val value: String,
    )

    @Serializable
    @SerialName("Inner3")
    data class Inner3(
        val field0: String,
        val field1: Int,
    )

    @Serializable
    @SerialName("Outer")
    data class Outer(
        val one: Inner1,
        val two: Inner2,
        val three: Inner3,
    )
    "#);
}

#[test]
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val one: Pair<String, Int>,
    )
    "#);
}

#[test]
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val one: Triple<String, Int, UShort>,
    )
    "#);
}

#[test]
fn struct_with_field_that_is_a_4_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16, f32),
    }

    // TODO: The NTuple4 struct should be emitted in the preamble if required, e.g.
    // data class NTuple4<T1, T2, T3, T4>(val t1: T1, val t2: T2, val t3: T3, val t4: T4)

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val one: NTuple4<String, Int, UShort, Float>,
    )
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

    let actual = emit!(EnumWithUnitVariants as Encoding::Json).unwrap();
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

    let actual = emit!(MyEnum as Encoding::Json).unwrap();
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

    let actual = emit!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyEnum")
    sealed interface MyEnum {
        @Serializable
        @SerialName("Variant1")
        data class Variant1(
            val value: String,
        ) : MyEnum
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

    let actual = emit!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyEnum")
    sealed interface MyEnum {
        @Serializable
        @SerialName("Variant1")
        data class Variant1(
            val value: String,
        ) : MyEnum

        @Serializable
        @SerialName("Variant2")
        data class Variant2(
            val value: Int,
        ) : MyEnum
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

    let actual = emit!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyEnum")
    sealed interface MyEnum {
        @Serializable
        @SerialName("Variant1")
        data class Variant1(
            val field0: String,
            val field1: Int,
        ) : MyEnum

        @Serializable
        @SerialName("Variant2")
        data class Variant2(
            val field0: Boolean,
            val field1: Double,
            val field2: UByte,
        ) : MyEnum
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

    let actual = emit!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyEnum")
    sealed interface MyEnum {
        @Serializable
        @SerialName("Variant1")
        data class Variant1(
            val field1: String,
            val field2: Int,
        ) : MyEnum
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

    let actual = emit!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyEnum")
    sealed interface MyEnum {
        @Serializable
        @SerialName("Unit")
        data object Unit: MyEnum

        @Serializable
        @SerialName("NewType")
        data class NewType(
            val value: String,
        ) : MyEnum

        @Serializable
        @SerialName("Tuple")
        data class Tuple(
            val field0: String,
            val field1: Int,
        ) : MyEnum

        @Serializable
        @SerialName("Struct")
        data class Struct(
            val field: Boolean,
        ) : MyEnum
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val items: List<String>,
        val numbers: List<Int>,
        val nestedItems: List<List<String>>,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val optionalString: String? = null,
        val optionalNumber: Int? = null,
        val optionalBool: Boolean? = null,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val stringToInt: Map<String, Int>,
        val intToBool: Map<Int, Boolean>,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val optionalList: List<String>? = null,
        val listOfOptionals: List<Int?>,
        val mapToList: Map<String, List<Boolean>>,
        val optionalMap: Map<String, Int>? = null,
        val complex: List<Map<String, List<Boolean>>?>,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val fixedArray: List<Int>,
        val byteArray: List<UByte>,
        val stringArray: List<String>,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val stringToInt: Map<String, Int>,
        val intToBool: Map<Int, Boolean>,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val stringSet: Set<String>,
        val intSet: Set<Int>,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val stringSet: Set<String>,
        val intSet: Set<Int>,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val boxedString: String,
        val boxedInt: Int,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val rcString: String,
        val rcInt: Int,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val arcString: String,
        val arcInt: Int,
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val vecOfSets: List<Set<String>>,
        val optionalBtree: Map<String, Int>? = null,
        val boxedVec: List<String>,
        val arcOption: String? = null,
        val arrayOfBoxes: List<Int>,
    )
    "#);
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

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val data: ByteArray,
        val name: String,
        val header: ByteArray,
    )
    "#);
}

#[test]
fn struct_with_bytes_field_and_slice() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        #[facet(bytes)]
        data: &'a [u8],
        name: String,
        #[facet(bytes)]
        header: Vec<u8>,
        optional_bytes: Option<Vec<u8>>,
    }

    let actual = emit!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    @SerialName("MyStruct")
    data class MyStruct(
        val data: ByteArray,
        val name: String,
        val header: ByteArray,
        val optionalBytes: List<UByte>? = null,
    )
    "#);
}
