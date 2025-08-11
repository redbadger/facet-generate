use facet::Facet;

use super::*;
use crate::emit;

#[test]
fn unit_struct_1() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    insta::assert_snapshot!(emit!(UnitStruct), @r"
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

    insta::assert_snapshot!(emit!(UnitStruct), @r"
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

    insta::assert_snapshot!(emit!(NewType), @r"
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

    insta::assert_snapshot!(emit!(TupleStruct), @r"
    /// line 1
    /// line 2
    @Serializable
    data class TupleStruct (
        val field_0: String,
        val field_1: Int
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

    insta::assert_snapshot!(emit!(StructWithFields), @r"
    /// line 1
    /// line 2
    @Serializable
    data class StructWithFields (
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
        val string: String
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

    insta::assert_snapshot!(emit!(Outer), @r"
    @Serializable
    data class Inner1 (
        val field1: String
    )

    typealias Inner2 = String

    @Serializable
    data class Inner3 (
        val field_0: String,
        val field_1: Int
    )

    @Serializable
    data class Outer (
        val one: Inner1,
        val two: Inner2,
        val three: Inner3
    )
    ");
}
