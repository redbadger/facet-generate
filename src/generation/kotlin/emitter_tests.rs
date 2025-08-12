use facet::Facet;

use super::*;
use crate::emit;

#[test]
fn unit_struct_1() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    /// line 1
    /// line 2
    @Serializable
    data object UnitStruct
    ");
}

#[test]
fn unit_struct_2() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct {}

    let actual = emit!(UnitStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    /// line 1
    /// line 2
    @Serializable
    data object UnitStruct
    ");
}

#[test]
fn newtype_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct NewType(String);

    let actual = emit!(NewType).unwrap();
    insta::assert_snapshot!(actual, @r"
    /// line 1
    /// line 2
    typealias NewType = String
    ");
}

#[test]
fn tuple_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    /// line 1
    /// line 2
    @Serializable
    data class TupleStruct(
        val field_0: String,
        val field_1: Int,
    )
    ");
}

#[test]
fn struct_with_fields_of_primitive_types() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct StructWithFields {
        /// unit
        unit: (),
        /// bool
        bool: bool,
        /// i8
        i8: i8,
        /// i16
        i16: i16,
        /// i32
        i32: i32,
        /// i64
        i64: i64,
        /// i128
        i128: i128,
        /// u8
        u8: u8,
        /// u16
        u16: u16,
        /// u32
        u32: u32,
        /// u64
        u64: u64,
        /// u128
        u128: u128,
        /// f32
        f32: f32,
        /// f64
        f64: f64,
        /// char
        char: char,
        /// string
        string: String,
    }

    let actual = emit!(StructWithFields).unwrap();
    insta::assert_snapshot!(actual, @r"
    /// line 1
    /// line 2
    @Serializable
    data class StructWithFields(
        /// unit
        val unit: Unit,
        /// bool
        val bool: Boolean,
        /// i8
        val i8: Byte,
        /// i16
        val i16: Short,
        /// i32
        val i32: Int,
        /// i64
        val i64: Long,
        /// i128
        val i128: java.math.@com.novi.serde.Int128 BigInteger,
        /// u8
        val u8: UByte,
        /// u16
        val u16: UShort,
        /// u32
        val u32: UInt,
        /// u64
        val u64: ULong,
        /// u128
        val u128: java.math.@com.novi.serde.Unsigned @com.novi.serde.Int128 BigInteger,
        /// f32
        val f32: Float,
        /// f64
        val f64: Double,
        /// char
        val char: String,
        /// string
        val string: String,
    )
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

    let actual = emit!(Outer).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class Inner1(
        val field1: String,
    )

    typealias Inner2 = String

    @Serializable
    data class Inner3(
        val field_0: String,
        val field_1: Int,
    )

    @Serializable
    data class Outer(
        val one: Inner1,
        val two: Inner2,
        val three: Inner3,
    )
    ");
}

#[test]
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val one: Pair<String, Int>,
    )
    ");
}

#[test]
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val one: Triple<String, Int, UShort>,
    )
    ");
}

#[test]
fn struct_with_field_that_is_a_4_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16, f32),
    }

    // TODO: The NTuple4 struct should be emitted in the preamble if required, e.g.
    // data class NTuple4<T1, T2, T3, T4>(val t1: T1, val t2: T2, val t3: T3, val t4: T4)

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val one: NTuple4<String, Int, UShort, Float>,
    )
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

    let actual = emit!(EnumWithUnitVariants).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line one
    /// line two
    @Serializable
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

    let actual = emit!(MyEnum).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
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

    let actual = emit!(MyEnum).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    sealed interface MyEnum {
        val serialName: String

        @Serializable
        @SerialName("Variant1")
        data class Variant1(
            val value: String,
        ) : MyEnum {
            override val serialName: String = "Variant1"
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

    let actual = emit!(MyEnum).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    sealed interface MyEnum {
        val serialName: String

        @Serializable
        @SerialName("Variant1")
        data class Variant1(
            val value: String,
        ) : MyEnum {
            override val serialName: String = "Variant1"
        }

        @Serializable
        @SerialName("Variant2")
        data class Variant2(
            val value: Int,
        ) : MyEnum {
            override val serialName: String = "Variant2"
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

    let actual = emit!(MyEnum).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    sealed interface MyEnum {
        val serialName: String

        @Serializable
        @SerialName("Variant1")
        data class Variant1(
            val field_0: String,
            val field_1: Int,
        ) : MyEnum {
            override val serialName: String = "Variant1"
        }

        @Serializable
        @SerialName("Variant2")
        data class Variant2(
            val field_0: Boolean,
            val field_1: Double,
            val field_2: UByte,
        ) : MyEnum {
            override val serialName: String = "Variant2"
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

    let actual = emit!(MyEnum).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    sealed interface MyEnum {
        val serialName: String

        @Serializable
        @SerialName("Variant1")
        data class Variant1(
            val field1: String,

            val field2: Int,

        ) : MyEnum {
            override val serialName: String = "Variant1"
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

    let actual = emit!(MyEnum).unwrap();
    insta::assert_snapshot!(actual, @r#"
    @Serializable
    sealed interface MyEnum {
        val serialName: String

        @Serializable
        @SerialName("Unit")
        data object Unit : MyEnum {
            override val serialName: String = "Unit"
        }

        @Serializable
        @SerialName("NewType")
        data class NewType(
            val value: String,
        ) : MyEnum {
            override val serialName: String = "NewType"
        }

        @Serializable
        @SerialName("Tuple")
        data class Tuple(
            val field_0: String,
            val field_1: Int,
        ) : MyEnum {
            override val serialName: String = "Tuple"
        }

        @Serializable
        @SerialName("Struct")
        data class Struct(
            val field: Boolean,

        ) : MyEnum {
            override val serialName: String = "Struct"
        }
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

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val items: List<String>,
        val numbers: List<Int>,
        val nested_items: List<List<String>>,
    )
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

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val optional_string: String? = null,
        val optional_number: Int? = null,
        val optional_bool: Boolean? = null,
    )
    ");
}

#[test]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: std::collections::HashMap<String, i32>,
        int_to_bool: std::collections::HashMap<i32, bool>,
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val string_to_int: Map<String, Int>,
        val int_to_bool: Map<Int, Boolean>,
    )
    ");
}

#[test]
fn struct_with_nested_generics() {
    #[derive(Facet)]
    struct MyStruct {
        optional_list: Option<Vec<String>>,
        list_of_optionals: Vec<Option<i32>>,
        map_to_list: std::collections::HashMap<String, Vec<bool>>,
        optional_map: Option<std::collections::HashMap<String, i32>>,
        complex: Vec<Option<std::collections::HashMap<String, Vec<bool>>>>,
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val optional_list: List<String>? = null,
        val list_of_optionals: List<Int?>,
        val map_to_list: Map<String, List<Boolean>>,
        val optional_map: Map<String, Int>? = null,
        val complex: List<Map<String, List<Boolean>>?>,
    )
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

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val fixed_array: List<Int>,
        val byte_array: List<UByte>,
        val string_array: List<String>,
    )
    ");
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: std::collections::BTreeMap<String, i32>,
        int_to_bool: std::collections::BTreeMap<i32, bool>,
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val string_to_int: Map<String, Int>,
        val int_to_bool: Map<Int, Boolean>,
    )
    ");
}

#[test]
fn struct_with_hashset_field() {
    // NOTE: HashSet<T> now maps to Set<T> in Kotlin with the new Format::Set variant.
    // This preserves the uniqueness constraint and provides better type safety.
    #[derive(Facet)]
    struct MyStruct {
        string_set: std::collections::HashSet<String>,
        int_set: std::collections::HashSet<i32>,
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val string_set: Set<String>,
        val int_set: Set<Int>,
    )
    ");
}

#[test]
fn struct_with_btreeset_field() {
    // NOTE: BTreeSet<T> now maps to Set<T> in Kotlin with the new Format::Set variant.
    // This preserves the uniqueness constraint and provides better type safety.
    #[derive(Facet)]
    struct MyStruct {
        string_set: std::collections::BTreeSet<String>,
        int_set: std::collections::BTreeSet<i32>,
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val string_set: Set<String>,
        val int_set: Set<Int>,
    )
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

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val boxed_string: String,
        val boxed_int: Int,
    )
    ");
}

#[test]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: std::rc::Rc<String>,
        rc_int: std::rc::Rc<i32>,
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val rc_string: String,
        val rc_int: Int,
    )
    ");
}

#[test]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: std::sync::Arc<String>,
        arc_int: std::sync::Arc<i32>,
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val arc_string: String,
        val arc_int: Int,
    )
    ");
}

#[test]
fn struct_with_mixed_collections_and_pointers() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        vec_of_sets: Vec<std::collections::HashSet<String>>,
        optional_btree: Option<std::collections::BTreeMap<String, i32>>,
        boxed_vec: Box<Vec<String>>,
        arc_option: std::sync::Arc<Option<String>>,
        array_of_boxes: [Box<i32>; 3],
    }

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val vec_of_sets: List<Set<String>>,
        val optional_btree: Map<String, Int>? = null,
        val boxed_vec: List<String>,
        val arc_option: String? = null,
        val array_of_boxes: List<Int>,
    )
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

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val data: ByteArray,
        val name: String,
        val header: ByteArray,
    )
    ");
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

    let actual = emit!(MyStruct).unwrap();
    insta::assert_snapshot!(actual, @r"
    @Serializable
    data class MyStruct(
        val data: ByteArray,
        val name: String,
        val header: ByteArray,
        val optional_bytes: List<UByte>? = null,
    )
    ");
}
