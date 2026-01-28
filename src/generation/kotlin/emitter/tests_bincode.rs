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

    let actual = emit!(UnitStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    data object UnitStruct {
        fun serialize(serializer: Serializer) {}

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        fun deserialize(deserializer: Deserializer): UnitStruct {
            return UnitStruct
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): UnitStruct {
            if (input == null) {
                throw DeserializationError("Cannot deserialize null array")
            }
            val deserializer = BincodeDeserializer(input)
            val value = deserialize(deserializer)
            if (deserializer._buffer_offset < input.size) {
                throw DeserializationError("Some input bytes were not read")
            }
            return value
        }
    }
    "#);
}

#[test]
fn unit_struct_2() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct {}

    let actual = emit!(UnitStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    data object UnitStruct {
        fun serialize(serializer: Serializer) {}

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        fun deserialize(deserializer: Deserializer): UnitStruct {
            return UnitStruct
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): UnitStruct {
            if (input == null) {
                throw DeserializationError("Cannot deserialize null array")
            }
            val deserializer = BincodeDeserializer(input)
            val value = deserialize(deserializer)
            if (deserializer._buffer_offset < input.size) {
                throw DeserializationError("Some input bytes were not read")
            }
            return value
        }
    }
    "#);
}

#[test]
fn newtype_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct NewType(String);

    let actual = emit!(NewType as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    data class NewType(
        val value: String,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(value)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): NewType {
                deserializer.increase_container_depth()
                val value = deserializer.deserialize_str()
                deserializer.decrease_container_depth()
                return NewType(value)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): NewType {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}

#[test]
fn tuple_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    data class TupleStruct(
        val field0: String,
        val field1: Int,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(field0)
            serializer.serialize_i32(field1)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): TupleStruct {
                deserializer.increase_container_depth()
                val field0 = deserializer.deserialize_str()
                val field1 = deserializer.deserialize_i32()
                deserializer.decrease_container_depth()
                return TupleStruct(field0, field1)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): TupleStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
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

    let actual = emit!(StructWithFields as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
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
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_unit(unit)
            serializer.serialize_bool(bool)
            serializer.serialize_i8(i8)
            serializer.serialize_i16(i16)
            serializer.serialize_i32(i32)
            serializer.serialize_i64(i64)
            serializer.serialize_i128(i128)
            serializer.serialize_u8(u8)
            serializer.serialize_u16(u16)
            serializer.serialize_u32(u32)
            serializer.serialize_u64(u64)
            serializer.serialize_u128(u128)
            serializer.serialize_f32(f32)
            serializer.serialize_f64(f64)
            serializer.serialize_char(char)
            serializer.serialize_str(string)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): StructWithFields {
                deserializer.increase_container_depth()
                val unit = deserializer.deserialize_unit()
                val bool = deserializer.deserialize_bool()
                val i8 = deserializer.deserialize_i8()
                val i16 = deserializer.deserialize_i16()
                val i32 = deserializer.deserialize_i32()
                val i64 = deserializer.deserialize_i64()
                val i128 = deserializer.deserialize_i128()
                val u8 = deserializer.deserialize_u8()
                val u16 = deserializer.deserialize_u16()
                val u32 = deserializer.deserialize_u32()
                val u64 = deserializer.deserialize_u64()
                val u128 = deserializer.deserialize_u128()
                val f32 = deserializer.deserialize_f32()
                val f64 = deserializer.deserialize_f64()
                val char = deserializer.deserialize_char()
                val string = deserializer.deserialize_str()
                deserializer.decrease_container_depth()
                return StructWithFields(unit, bool, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, char, string)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): StructWithFields {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
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

    let actual = emit!(Outer as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class Inner1(
        val field1: String,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(field1)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Inner1 {
                deserializer.increase_container_depth()
                val field1 = deserializer.deserialize_str()
                deserializer.decrease_container_depth()
                return Inner1(field1)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): Inner1 {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }

    data class Inner2(
        val value: String,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(value)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Inner2 {
                deserializer.increase_container_depth()
                val value = deserializer.deserialize_str()
                deserializer.decrease_container_depth()
                return Inner2(value)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): Inner2 {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }

    data class Inner3(
        val field0: String,
        val field1: Int,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(field0)
            serializer.serialize_i32(field1)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Inner3 {
                deserializer.increase_container_depth()
                val field0 = deserializer.deserialize_str()
                val field1 = deserializer.deserialize_i32()
                deserializer.decrease_container_depth()
                return Inner3(field0, field1)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): Inner3 {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }

    data class Outer(
        val one: Inner1,
        val two: Inner2,
        val three: Inner3,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            one.serialize(serializer)
            two.serialize(serializer)
            three.serialize(serializer)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Outer {
                deserializer.increase_container_depth()
                val one = Inner1.deserialize(deserializer)
                val two = Inner2.deserialize(deserializer)
                val three = Inner3.deserialize(deserializer)
                deserializer.decrease_container_depth()
                return Outer(one, two, three)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): Outer {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}

#[test]
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val one: Pair<String, Int>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(one.first)
            serializer.serialize_i32(one.second)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val one = run {
                    val first = deserializer.deserialize_str()
                    val second = deserializer.deserialize_i32()
                    Pair(first, second)
                }
                deserializer.decrease_container_depth()
                return MyStruct(one)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}

#[test]
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val one: Triple<String, Int, UShort>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(one.first)
            serializer.serialize_i32(one.second)
            serializer.serialize_u16(one.third)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val one = run {
                    val first = deserializer.deserialize_str()
                    val second = deserializer.deserialize_i32()
                    val third = deserializer.deserialize_u16()
                    Triple(first, second, third)
                }
                deserializer.decrease_container_depth()
                return MyStruct(one)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val one: NTuple4<String, Int, UShort, Float>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(one.component1())
            serializer.serialize_i32(one.component2())
            serializer.serialize_u16(one.component3())
            serializer.serialize_f32(one.component4())
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val one = run {
                    val v0 = deserializer.deserialize_str()
                    val v1 = deserializer.deserialize_i32()
                    val v2 = deserializer.deserialize_u16()
                    val v3 = deserializer.deserialize_f32()
                    NTuple4(v0, v1, v2, v3)
                }
                deserializer.decrease_container_depth()
                return MyStruct(one)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
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

    let actual = emit!(EnumWithUnitVariants as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line one
    /// line two
    enum class EnumWithUnitVariants {
        /// variant one
        VARIANT1,
        /// variant two
        VARIANT2,
        /// variant three
        VARIANT3;

        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(ordinal)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            @Throws(DeserializationError::class)
            fun deserialize(deserializer: Deserializer): EnumWithUnitVariants {
                deserializer.increase_container_depth()
                val index = deserializer.deserialize_variant_index()
                deserializer.decrease_container_depth()
                return when (index) {
                    0 -> VARIANT1
                    1 -> VARIANT2
                    2 -> VARIANT3
                    else -> throw DeserializationError("Unknown variant index for EnumWithUnitVariants: $index")
                }
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): EnumWithUnitVariants {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
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
        Variant1 {},
    }

    let actual = emit!(MyEnum as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    enum class MyEnum {
        VARIANT1;

        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(ordinal)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            @Throws(DeserializationError::class)
            fun deserialize(deserializer: Deserializer): MyEnum {
                deserializer.increase_container_depth()
                val index = deserializer.deserialize_variant_index()
                deserializer.decrease_container_depth()
                return when (index) {
                    0 -> VARIANT1
                    else -> throw DeserializationError("Unknown variant index for MyEnum: $index")
                }
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyEnum {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
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

    let actual = emit!(MyEnum as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    sealed interface MyEnum {
        fun serialize(serializer: Serializer)

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        data class Variant1(
            val value: String,
        ) : MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(0)
                serializer.serialize_str(value)
                serializer.decrease_container_depth()
            }

            companion object {
                fun deserialize(deserializer: Deserializer): Variant1 {
                    deserializer.increase_container_depth()
                    val value = deserializer.deserialize_str()
                    deserializer.decrease_container_depth()
                    return Variant1(value)
                }
            }
        }

        companion object {
            @Throws(DeserializationError::class)
            fun deserialize(deserializer: Deserializer): MyEnum {
                val index = deserializer.deserialize_variant_index()
                return when (index) {
                    0 -> Variant1.deserialize(deserializer)
                    else -> throw DeserializationError("Unknown variant index for MyEnum: $index")
                }
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyEnum {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
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

    let actual = emit!(MyEnum as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    sealed interface MyEnum {
        fun serialize(serializer: Serializer)

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        data class Variant1(
            val value: String,
        ) : MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(0)
                serializer.serialize_str(value)
                serializer.decrease_container_depth()
            }

            companion object {
                fun deserialize(deserializer: Deserializer): Variant1 {
                    deserializer.increase_container_depth()
                    val value = deserializer.deserialize_str()
                    deserializer.decrease_container_depth()
                    return Variant1(value)
                }
            }
        }

        data class Variant2(
            val value: Int,
        ) : MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(1)
                serializer.serialize_i32(value)
                serializer.decrease_container_depth()
            }

            companion object {
                fun deserialize(deserializer: Deserializer): Variant2 {
                    deserializer.increase_container_depth()
                    val value = deserializer.deserialize_i32()
                    deserializer.decrease_container_depth()
                    return Variant2(value)
                }
            }
        }

        companion object {
            @Throws(DeserializationError::class)
            fun deserialize(deserializer: Deserializer): MyEnum {
                val index = deserializer.deserialize_variant_index()
                return when (index) {
                    0 -> Variant1.deserialize(deserializer)
                    1 -> Variant2.deserialize(deserializer)
                    else -> throw DeserializationError("Unknown variant index for MyEnum: $index")
                }
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyEnum {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
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

    let actual = emit!(MyEnum as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    sealed interface MyEnum {
        fun serialize(serializer: Serializer)

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        data class Variant1(
            val field0: String,
            val field1: Int,
        ) : MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(0)
                serializer.serialize_str(field0)
                serializer.serialize_i32(field1)
                serializer.decrease_container_depth()
            }

            companion object {
                fun deserialize(deserializer: Deserializer): Variant1 {
                    deserializer.increase_container_depth()
                    val field0 = deserializer.deserialize_str()
                    val field1 = deserializer.deserialize_i32()
                    deserializer.decrease_container_depth()
                    return Variant1(field0, field1)
                }
            }
        }

        data class Variant2(
            val field0: Boolean,
            val field1: Double,
            val field2: UByte,
        ) : MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(1)
                serializer.serialize_bool(field0)
                serializer.serialize_f64(field1)
                serializer.serialize_u8(field2)
                serializer.decrease_container_depth()
            }

            companion object {
                fun deserialize(deserializer: Deserializer): Variant2 {
                    deserializer.increase_container_depth()
                    val field0 = deserializer.deserialize_bool()
                    val field1 = deserializer.deserialize_f64()
                    val field2 = deserializer.deserialize_u8()
                    deserializer.decrease_container_depth()
                    return Variant2(field0, field1, field2)
                }
            }
        }

        companion object {
            @Throws(DeserializationError::class)
            fun deserialize(deserializer: Deserializer): MyEnum {
                val index = deserializer.deserialize_variant_index()
                return when (index) {
                    0 -> Variant1.deserialize(deserializer)
                    1 -> Variant2.deserialize(deserializer)
                    else -> throw DeserializationError("Unknown variant index for MyEnum: $index")
                }
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyEnum {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
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

    let actual = emit!(MyEnum as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    sealed interface MyEnum {
        fun serialize(serializer: Serializer)

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        data class Variant1(
            val field1: String,
            val field2: Int,
        ) : MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(0)
                serializer.serialize_str(field1)
                serializer.serialize_i32(field2)
                serializer.decrease_container_depth()
            }

            companion object {
                fun deserialize(deserializer: Deserializer): Variant1 {
                    deserializer.increase_container_depth()
                    val field1 = deserializer.deserialize_str()
                    val field2 = deserializer.deserialize_i32()
                    deserializer.decrease_container_depth()
                    return Variant1(field1, field2)
                }
            }
        }

        companion object {
            @Throws(DeserializationError::class)
            fun deserialize(deserializer: Deserializer): MyEnum {
                val index = deserializer.deserialize_variant_index()
                return when (index) {
                    0 -> Variant1.deserialize(deserializer)
                    else -> throw DeserializationError("Unknown variant index for MyEnum: $index")
                }
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyEnum {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
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

    let actual = emit!(MyEnum as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    sealed interface MyEnum {
        fun serialize(serializer: Serializer)

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        data object Unit: MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(0)
                serializer.decrease_container_depth()
            }

            fun deserialize(deserializer: Deserializer): Unit {
                return Unit
            }
        }

        data class NewType(
            val value: String,
        ) : MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(1)
                serializer.serialize_str(value)
                serializer.decrease_container_depth()
            }

            companion object {
                fun deserialize(deserializer: Deserializer): NewType {
                    deserializer.increase_container_depth()
                    val value = deserializer.deserialize_str()
                    deserializer.decrease_container_depth()
                    return NewType(value)
                }
            }
        }

        data class Tuple(
            val field0: String,
            val field1: Int,
        ) : MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(2)
                serializer.serialize_str(field0)
                serializer.serialize_i32(field1)
                serializer.decrease_container_depth()
            }

            companion object {
                fun deserialize(deserializer: Deserializer): Tuple {
                    deserializer.increase_container_depth()
                    val field0 = deserializer.deserialize_str()
                    val field1 = deserializer.deserialize_i32()
                    deserializer.decrease_container_depth()
                    return Tuple(field0, field1)
                }
            }
        }

        data class Struct(
            val field: Boolean,
        ) : MyEnum {
            override fun serialize(serializer: Serializer) {
                serializer.increase_container_depth()
                serializer.serialize_variant_index(3)
                serializer.serialize_bool(field)
                serializer.decrease_container_depth()
            }

            companion object {
                fun deserialize(deserializer: Deserializer): Struct {
                    deserializer.increase_container_depth()
                    val field = deserializer.deserialize_bool()
                    deserializer.decrease_container_depth()
                    return Struct(field)
                }
            }
        }

        companion object {
            @Throws(DeserializationError::class)
            fun deserialize(deserializer: Deserializer): MyEnum {
                val index = deserializer.deserialize_variant_index()
                return when (index) {
                    0 -> Unit.deserialize(deserializer)
                    1 -> NewType.deserialize(deserializer)
                    2 -> Tuple.deserialize(deserializer)
                    3 -> Struct.deserialize(deserializer)
                    else -> throw DeserializationError("Unknown variant index for MyEnum: $index")
                }
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyEnum {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val items: List<String>,
        val numbers: List<Int>,
        val nestedItems: List<List<String>>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            items.serialize(serializer) {
                serializer.serialize_str(it)
            }
            numbers.serialize(serializer) {
                serializer.serialize_i32(it)
            }
            nestedItems.serialize(serializer) { level1 ->
                level1.serialize(serializer) {
                    serializer.serialize_str(it)
                }
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val items =
                    deserializer.deserializeListOf {
                        deserializer.deserialize_str()
                    }
                val numbers =
                    deserializer.deserializeListOf {
                        deserializer.deserialize_i32()
                    }
                val nestedItems =
                    deserializer.deserializeListOf {
                        deserializer.deserializeListOf {
                            deserializer.deserialize_str()
                        }
                    }
                deserializer.decrease_container_depth()
                return MyStruct(items, numbers, nestedItems)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
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

    let actual = emit!(Parent as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class Child(
        val name: String,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(name)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Child {
                deserializer.increase_container_depth()
                val name = deserializer.deserialize_str()
                deserializer.decrease_container_depth()
                return Child(name)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): Child {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }

    data class Parent(
        val children: List<List<Child>>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            children.serialize(serializer) { level1 ->
                level1.serialize(serializer) {
                    it.serialize(serializer)
                }
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Parent {
                deserializer.increase_container_depth()
                val children =
                    deserializer.deserializeListOf {
                        deserializer.deserializeListOf {
                            Child.deserialize(deserializer)
                        }
                    }
                deserializer.decrease_container_depth()
                return Parent(children)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): Parent {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val simple: String? = null,
        val nested: Int?? = null,
        val list: List<Boolean>? = null,
        val listOfOptions: List<Boolean?>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            simple.serializeOptionOf(serializer) {
                serializer.serialize_str(it)
            }
            nested.serializeOptionOf(serializer) { level1 ->
                level1.serializeOptionOf(serializer) {
                    serializer.serialize_i32(it)
                }
            }
            list.serializeOptionOf(serializer) { level1 ->
                level1.serialize(serializer) {
                    serializer.serialize_bool(it)
                }
            }
            listOfOptions.serialize(serializer) { level1 ->
                level1.serializeOptionOf(serializer) {
                    serializer.serialize_bool(it)
                }
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val simple =
                    deserializer.deserializeOptionOf {
                        deserializer.deserialize_str()
                    }
                val nested =
                    deserializer.deserializeOptionOf {
                        deserializer.deserializeOptionOf {
                            deserializer.deserialize_i32()
                        }
                    }
                val list =
                    deserializer.deserializeOptionOf {
                        deserializer.deserializeListOf {
                            deserializer.deserialize_bool()
                        }
                    }
                val listOfOptions =
                    deserializer.deserializeListOf {
                        deserializer.deserializeOptionOf {
                            deserializer.deserialize_bool()
                        }
                    }
                deserializer.decrease_container_depth()
                return MyStruct(simple, nested, list, listOfOptions)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}

#[test]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        int_to_bool: HashMap<i32, bool>,
    }

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val stringToInt: Map<String, Int>,
        val intToBool: Map<Int, Boolean>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            stringToInt.serialize(serializer) { key, value ->
                serializer.serialize_str(key)
                serializer.serialize_i32(value)
            }
            intToBool.serialize(serializer) { key, value ->
                serializer.serialize_i32(key)
                serializer.serialize_bool(value)
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val stringToInt =
                    deserializer.deserializeMapOf {
                        val key = deserializer.deserialize_str()
                        val value = deserializer.deserialize_i32()
                        Pair(key, value)
                    }
                val intToBool =
                    deserializer.deserializeMapOf {
                        val key = deserializer.deserialize_i32()
                        val value = deserializer.deserialize_bool()
                        Pair(key, value)
                    }
                deserializer.decrease_container_depth()
                return MyStruct(stringToInt, intToBool)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val optionalList: List<String>? = null,
        val listOfOptionals: List<Int?>,
        val mapToList: Map<String, List<Boolean>>,
        val optionalMap: Map<String, Int>? = null,
        val complex: List<Map<String, List<Boolean>>?>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            optionalList.serializeOptionOf(serializer) { level1 ->
                level1.serialize(serializer) {
                    serializer.serialize_str(it)
                }
            }
            listOfOptionals.serialize(serializer) { level1 ->
                level1.serializeOptionOf(serializer) {
                    serializer.serialize_i32(it)
                }
            }
            mapToList.serialize(serializer) { key, value ->
                serializer.serialize_str(key)
                value.serialize(serializer) {
                    serializer.serialize_bool(it)
                }
            }
            optionalMap.serializeOptionOf(serializer) { level1 ->
                level1.serialize(serializer) { key, value ->
                    serializer.serialize_str(key)
                    serializer.serialize_i32(value)
                }
            }
            complex.serialize(serializer) { level1 ->
                level1.serializeOptionOf(serializer) { level2 ->
                    level2.serialize(serializer) { key, value ->
                        serializer.serialize_str(key)
                        value.serialize(serializer) {
                            serializer.serialize_bool(it)
                        }
                    }
                }
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val optionalList =
                    deserializer.deserializeOptionOf {
                        deserializer.deserializeListOf {
                            deserializer.deserialize_str()
                        }
                    }
                val listOfOptionals =
                    deserializer.deserializeListOf {
                        deserializer.deserializeOptionOf {
                            deserializer.deserialize_i32()
                        }
                    }
                val mapToList =
                    deserializer.deserializeMapOf {
                        val key = deserializer.deserialize_str()
                        val value =
                            deserializer.deserializeListOf {
                                deserializer.deserialize_bool()
                            }
                        Pair(key, value)
                    }
                val optionalMap =
                    deserializer.deserializeOptionOf {
                        deserializer.deserializeMapOf {
                            val key = deserializer.deserialize_str()
                            val value = deserializer.deserialize_i32()
                            Pair(key, value)
                        }
                    }
                val complex =
                    deserializer.deserializeListOf {
                        deserializer.deserializeOptionOf {
                            deserializer.deserializeMapOf {
                                val key = deserializer.deserialize_str()
                                val value =
                                    deserializer.deserializeListOf {
                                        deserializer.deserialize_bool()
                                    }
                                Pair(key, value)
                            }
                        }
                    }
                deserializer.decrease_container_depth()
                return MyStruct(optionalList, listOfOptionals, mapToList, optionalMap, complex)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val fixedArray: List<Int>,
        val byteArray: List<UByte>,
        val stringArray: List<String>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            fixedArray.serialize(serializer)
            byteArray.serialize(serializer)
            stringArray.serialize(serializer)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val fixedArray = buildList(5) { repeat(5) { add(deserializer.deserialize_i32()) } }
                val byteArray = buildList(32) { repeat(32) { add(deserializer.deserialize_u8()) } }
                val stringArray = buildList(3) { repeat(3) { add(deserializer.deserialize_str()) } }
                deserializer.decrease_container_depth()
                return MyStruct(fixedArray, byteArray, stringArray)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val stringToInt: Map<String, Int>,
        val intToBool: Map<Int, Boolean>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            stringToInt.serialize(serializer) { key, value ->
                serializer.serialize_str(key)
                serializer.serialize_i32(value)
            }
            intToBool.serialize(serializer) { key, value ->
                serializer.serialize_i32(key)
                serializer.serialize_bool(value)
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val stringToInt =
                    deserializer.deserializeMapOf {
                        val key = deserializer.deserialize_str()
                        val value = deserializer.deserialize_i32()
                        Pair(key, value)
                    }
                val intToBool =
                    deserializer.deserializeMapOf {
                        val key = deserializer.deserialize_i32()
                        val value = deserializer.deserialize_bool()
                        Pair(key, value)
                    }
                deserializer.decrease_container_depth()
                return MyStruct(stringToInt, intToBool)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}

#[test]
fn struct_with_nested_map_field() {
    #[derive(Facet)]
    struct MyStruct {
        map_to_list: HashMap<String, Vec<i32>>,
        list_to_map: Vec<HashMap<i32, String>>,
    }

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val mapToList: Map<String, List<Int>>,
        val listToMap: List<Map<Int, String>>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            mapToList.serialize(serializer) { key, value ->
                serializer.serialize_str(key)
                value.serialize(serializer) {
                    serializer.serialize_i32(it)
                }
            }
            listToMap.serialize(serializer) { level1 ->
                level1.serialize(serializer) { key, value ->
                    serializer.serialize_i32(key)
                    serializer.serialize_str(value)
                }
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val mapToList =
                    deserializer.deserializeMapOf {
                        val key = deserializer.deserialize_str()
                        val value =
                            deserializer.deserializeListOf {
                                deserializer.deserialize_i32()
                            }
                        Pair(key, value)
                    }
                val listToMap =
                    deserializer.deserializeListOf {
                        deserializer.deserializeMapOf {
                            val key = deserializer.deserialize_i32()
                            val value = deserializer.deserialize_str()
                            Pair(key, value)
                        }
                    }
                deserializer.decrease_container_depth()
                return MyStruct(mapToList, listToMap)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val stringSet: Set<String>,
        val intSet: Set<Int>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            stringSet.serialize(serializer) {
                serializer.serialize_str(it)
            }
            intSet.serialize(serializer) {
                serializer.serialize_i32(it)
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val stringSet =
                    deserializer.deserializeSetOf {
                        deserializer.deserialize_str()
                    }
                val intSet =
                    deserializer.deserializeSetOf {
                        deserializer.deserialize_i32()
                    }
                deserializer.decrease_container_depth()
                return MyStruct(stringSet, intSet)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val stringSet: Set<String>,
        val intSet: Set<Int>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            stringSet.serialize(serializer) {
                serializer.serialize_str(it)
            }
            intSet.serialize(serializer) {
                serializer.serialize_i32(it)
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val stringSet =
                    deserializer.deserializeSetOf {
                        deserializer.deserialize_str()
                    }
                val intSet =
                    deserializer.deserializeSetOf {
                        deserializer.deserialize_i32()
                    }
                deserializer.decrease_container_depth()
                return MyStruct(stringSet, intSet)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}

#[test]
fn struct_with_nested_set_field() {
    #[derive(Facet)]
    struct MyStruct {
        vec_of_sets: Vec<HashSet<String>>,
        set_of_ints: HashSet<i32>,
    }

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val vecOfSets: List<Set<String>>,
        val setOfInts: Set<Int>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            vecOfSets.serialize(serializer) { level1 ->
                level1.serialize(serializer) {
                    serializer.serialize_str(it)
                }
            }
            setOfInts.serialize(serializer) {
                serializer.serialize_i32(it)
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val vecOfSets =
                    deserializer.deserializeListOf {
                        deserializer.deserializeSetOf {
                            deserializer.deserialize_str()
                        }
                    }
                val setOfInts =
                    deserializer.deserializeSetOf {
                        deserializer.deserialize_i32()
                    }
                deserializer.decrease_container_depth()
                return MyStruct(vecOfSets, setOfInts)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val boxedString: String,
        val boxedInt: Int,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(boxedString)
            serializer.serialize_i32(boxedInt)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val boxedString = deserializer.deserialize_str()
                val boxedInt = deserializer.deserialize_i32()
                deserializer.decrease_container_depth()
                return MyStruct(boxedString, boxedInt)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}

#[test]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: Rc<String>,
        rc_int: Rc<i32>,
    }

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val rcString: String,
        val rcInt: Int,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(rcString)
            serializer.serialize_i32(rcInt)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val rcString = deserializer.deserialize_str()
                val rcInt = deserializer.deserialize_i32()
                deserializer.decrease_container_depth()
                return MyStruct(rcString, rcInt)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}

#[test]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: Arc<String>,
        arc_int: Arc<i32>,
    }

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val arcString: String,
        val arcInt: Int,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_str(arcString)
            serializer.serialize_i32(arcInt)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val arcString = deserializer.deserialize_str()
                val arcInt = deserializer.deserialize_i32()
                deserializer.decrease_container_depth()
                return MyStruct(arcString, arcInt)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val vecOfSets: List<Set<String>>,
        val optionalBtree: Map<String, Int>? = null,
        val boxedVec: List<String>,
        val arcOption: String? = null,
        val arrayOfBoxes: List<Int>,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            vecOfSets.serialize(serializer) { level1 ->
                level1.serialize(serializer) {
                    serializer.serialize_str(it)
                }
            }
            optionalBtree.serializeOptionOf(serializer) { level1 ->
                level1.serialize(serializer) { key, value ->
                    serializer.serialize_str(key)
                    serializer.serialize_i32(value)
                }
            }
            boxedVec.serialize(serializer) {
                serializer.serialize_str(it)
            }
            arcOption.serializeOptionOf(serializer) {
                serializer.serialize_str(it)
            }
            arrayOfBoxes.serialize(serializer)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val vecOfSets =
                    deserializer.deserializeListOf {
                        deserializer.deserializeSetOf {
                            deserializer.deserialize_str()
                        }
                    }
                val optionalBtree =
                    deserializer.deserializeOptionOf {
                        deserializer.deserializeMapOf {
                            val key = deserializer.deserialize_str()
                            val value = deserializer.deserialize_i32()
                            Pair(key, value)
                        }
                    }
                val boxedVec =
                    deserializer.deserializeListOf {
                        deserializer.deserialize_str()
                    }
                val arcOption =
                    deserializer.deserializeOptionOf {
                        deserializer.deserialize_str()
                    }
                val arrayOfBoxes = buildList(3) { repeat(3) { add(deserializer.deserialize_i32()) } }
                deserializer.decrease_container_depth()
                return MyStruct(vecOfSets, optionalBtree, boxedVec, arcOption, arrayOfBoxes)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val data: ByteArray,
        val name: String,
        val header: ByteArray,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_bytes(data)
            serializer.serialize_str(name)
            serializer.serialize_bytes(header)
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val data = deserializer.deserialize_bytes()
                val name = deserializer.deserialize_str()
                val header = deserializer.deserialize_bytes()
                deserializer.decrease_container_depth()
                return MyStruct(data, name, header)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
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

    let actual = emit!(MyStruct as Encoding::Bincode).unwrap();
    insta::assert_snapshot!(actual, @r#"
    data class MyStruct(
        val data: ByteArray,
        val name: String,
        val header: ByteArray,
        val optionalBytes: List<UByte>? = null,
    ) {
        fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_bytes(data)
            serializer.serialize_str(name)
            serializer.serialize_bytes(header)
            optionalBytes.serializeOptionOf(serializer) { level1 ->
                level1.serialize(serializer) {
                    serializer.serialize_u8(it)
                }
            }
            serializer.decrease_container_depth()
        }

        fun bincodeSerialize(): ByteArray {
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer._bytes
        }

        companion object {
            fun deserialize(deserializer: Deserializer): MyStruct {
                deserializer.increase_container_depth()
                val data = deserializer.deserialize_bytes()
                val name = deserializer.deserialize_str()
                val header = deserializer.deserialize_bytes()
                val optionalBytes =
                    deserializer.deserializeOptionOf {
                        deserializer.deserializeListOf {
                            deserializer.deserialize_u8()
                        }
                    }
                deserializer.decrease_container_depth()
                return MyStruct(data, name, header, optionalBytes)
            }

            @Throws(DeserializationError::class)
            fun bincodeDeserialize(input: ByteArray?): MyStruct {
                if (input == null) {
                    throw DeserializationError("Cannot deserialize null array")
                }
                val deserializer = BincodeDeserializer(input)
                val value = deserialize(deserializer)
                if (deserializer._buffer_offset < input.size) {
                    throw DeserializationError("Some input bytes were not read")
                }
                return value
            }
        }
    }
    "#);
}
