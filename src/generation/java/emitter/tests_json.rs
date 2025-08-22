#![allow(clippy::too_many_lines)]
use std::collections::HashMap;

use facet::Facet;

use crate::{emit_java, generation::Encoding};

#[test]
fn unit_struct_1() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit_java!(UnitStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class UnitStruct {
        public UnitStruct() {
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static UnitStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static UnitStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            UnitStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            UnitStruct other = (UnitStruct) obj;
            return true;
        }

        public int hashCode() {
            int value = 7;
            return value;
        }

        public static final class Builder {
            public UnitStruct build() {
                return new UnitStruct(
                );
            }
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

    let actual = emit_java!(UnitStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class UnitStruct {
        public UnitStruct() {
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static UnitStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static UnitStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            UnitStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            UnitStruct other = (UnitStruct) obj;
            return true;
        }

        public int hashCode() {
            int value = 7;
            return value;
        }

        public static final class Builder {
            public UnitStruct build() {
                return new UnitStruct(
                );
            }
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

    let actual = emit_java!(NewType as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class NewType {
        public final String value;

        public NewType(String value) {
            java.util.Objects.requireNonNull(value, "value must not be null");
            this.value = value;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_str(value);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static NewType deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.value = deserializer.deserialize_str();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static NewType jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            NewType value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            NewType other = (NewType) obj;
            if (!java.util.Objects.equals(this.value, other.value)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.value != null ? this.value.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String value;

            public NewType build() {
                return new NewType(
                    value
                );
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

    let actual = emit_java!(TupleStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class TupleStruct {
        public final String field0;
        public final Integer field1;

        public TupleStruct(String field0, Integer field1) {
            java.util.Objects.requireNonNull(field0, "field0 must not be null");
            java.util.Objects.requireNonNull(field1, "field1 must not be null");
            this.field0 = field0;
            this.field1 = field1;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_str(field0);
            serializer.serialize_i32(field1);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static TupleStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.field0 = deserializer.deserialize_str();
            builder.field1 = deserializer.deserialize_i32();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static TupleStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            TupleStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            TupleStruct other = (TupleStruct) obj;
            if (!java.util.Objects.equals(this.field0, other.field0)) { return false; }
            if (!java.util.Objects.equals(this.field1, other.field1)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.field0 != null ? this.field0.hashCode() : 0);
            value = 31 * value + (this.field1 != null ? this.field1.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String field0;
            public Integer field1;

            public TupleStruct build() {
                return new TupleStruct(
                    field0,
                    field1
                );
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

    let actual = emit_java!(StructWithFields as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class StructWithFields {
        public final com.novi.serde.Unit unit;
        public final Boolean bool;
        public final Byte i8;
        public final Short i16;
        public final Integer i32;
        public final Long i64;
        public final java.math.@com.novi.serde.Int128 BigInteger i128;
        public final @com.novi.serde.Unsigned Byte u8;
        public final @com.novi.serde.Unsigned Short u16;
        public final @com.novi.serde.Unsigned Integer u32;
        public final @com.novi.serde.Unsigned Long u64;
        public final java.math.@com.novi.serde.Unsigned @com.novi.serde.Int128 BigInteger u128;
        public final Float f32;
        public final Double f64;
        public final Character char;
        public final String string;

        public StructWithFields(com.novi.serde.Unit unit, Boolean bool, Byte i8, Short i16, Integer i32, Long i64, java.math.@com.novi.serde.Int128 BigInteger i128, @com.novi.serde.Unsigned Byte u8, @com.novi.serde.Unsigned Short u16, @com.novi.serde.Unsigned Integer u32, @com.novi.serde.Unsigned Long u64, java.math.@com.novi.serde.Unsigned @com.novi.serde.Int128 BigInteger u128, Float f32, Double f64, Character char, String string) {
            java.util.Objects.requireNonNull(unit, "unit must not be null");
            java.util.Objects.requireNonNull(bool, "bool must not be null");
            java.util.Objects.requireNonNull(i8, "i8 must not be null");
            java.util.Objects.requireNonNull(i16, "i16 must not be null");
            java.util.Objects.requireNonNull(i32, "i32 must not be null");
            java.util.Objects.requireNonNull(i64, "i64 must not be null");
            java.util.Objects.requireNonNull(i128, "i128 must not be null");
            java.util.Objects.requireNonNull(u8, "u8 must not be null");
            java.util.Objects.requireNonNull(u16, "u16 must not be null");
            java.util.Objects.requireNonNull(u32, "u32 must not be null");
            java.util.Objects.requireNonNull(u64, "u64 must not be null");
            java.util.Objects.requireNonNull(u128, "u128 must not be null");
            java.util.Objects.requireNonNull(f32, "f32 must not be null");
            java.util.Objects.requireNonNull(f64, "f64 must not be null");
            java.util.Objects.requireNonNull(char, "char must not be null");
            java.util.Objects.requireNonNull(string, "string must not be null");
            this.unit = unit;
            this.bool = bool;
            this.i8 = i8;
            this.i16 = i16;
            this.i32 = i32;
            this.i64 = i64;
            this.i128 = i128;
            this.u8 = u8;
            this.u16 = u16;
            this.u32 = u32;
            this.u64 = u64;
            this.u128 = u128;
            this.f32 = f32;
            this.f64 = f64;
            this.char = char;
            this.string = string;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_unit(unit);
            serializer.serialize_bool(bool);
            serializer.serialize_i8(i8);
            serializer.serialize_i16(i16);
            serializer.serialize_i32(i32);
            serializer.serialize_i64(i64);
            serializer.serialize_i128(i128);
            serializer.serialize_u8(u8);
            serializer.serialize_u16(u16);
            serializer.serialize_u32(u32);
            serializer.serialize_u64(u64);
            serializer.serialize_u128(u128);
            serializer.serialize_f32(f32);
            serializer.serialize_f64(f64);
            serializer.serialize_char(char);
            serializer.serialize_str(string);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static StructWithFields deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.unit = deserializer.deserialize_unit();
            builder.bool = deserializer.deserialize_bool();
            builder.i8 = deserializer.deserialize_i8();
            builder.i16 = deserializer.deserialize_i16();
            builder.i32 = deserializer.deserialize_i32();
            builder.i64 = deserializer.deserialize_i64();
            builder.i128 = deserializer.deserialize_i128();
            builder.u8 = deserializer.deserialize_u8();
            builder.u16 = deserializer.deserialize_u16();
            builder.u32 = deserializer.deserialize_u32();
            builder.u64 = deserializer.deserialize_u64();
            builder.u128 = deserializer.deserialize_u128();
            builder.f32 = deserializer.deserialize_f32();
            builder.f64 = deserializer.deserialize_f64();
            builder.char = deserializer.deserialize_char();
            builder.string = deserializer.deserialize_str();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static StructWithFields jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            StructWithFields value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            StructWithFields other = (StructWithFields) obj;
            if (!java.util.Objects.equals(this.unit, other.unit)) { return false; }
            if (!java.util.Objects.equals(this.bool, other.bool)) { return false; }
            if (!java.util.Objects.equals(this.i8, other.i8)) { return false; }
            if (!java.util.Objects.equals(this.i16, other.i16)) { return false; }
            if (!java.util.Objects.equals(this.i32, other.i32)) { return false; }
            if (!java.util.Objects.equals(this.i64, other.i64)) { return false; }
            if (!java.util.Objects.equals(this.i128, other.i128)) { return false; }
            if (!java.util.Objects.equals(this.u8, other.u8)) { return false; }
            if (!java.util.Objects.equals(this.u16, other.u16)) { return false; }
            if (!java.util.Objects.equals(this.u32, other.u32)) { return false; }
            if (!java.util.Objects.equals(this.u64, other.u64)) { return false; }
            if (!java.util.Objects.equals(this.u128, other.u128)) { return false; }
            if (!java.util.Objects.equals(this.f32, other.f32)) { return false; }
            if (!java.util.Objects.equals(this.f64, other.f64)) { return false; }
            if (!java.util.Objects.equals(this.char, other.char)) { return false; }
            if (!java.util.Objects.equals(this.string, other.string)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.unit != null ? this.unit.hashCode() : 0);
            value = 31 * value + (this.bool != null ? this.bool.hashCode() : 0);
            value = 31 * value + (this.i8 != null ? this.i8.hashCode() : 0);
            value = 31 * value + (this.i16 != null ? this.i16.hashCode() : 0);
            value = 31 * value + (this.i32 != null ? this.i32.hashCode() : 0);
            value = 31 * value + (this.i64 != null ? this.i64.hashCode() : 0);
            value = 31 * value + (this.i128 != null ? this.i128.hashCode() : 0);
            value = 31 * value + (this.u8 != null ? this.u8.hashCode() : 0);
            value = 31 * value + (this.u16 != null ? this.u16.hashCode() : 0);
            value = 31 * value + (this.u32 != null ? this.u32.hashCode() : 0);
            value = 31 * value + (this.u64 != null ? this.u64.hashCode() : 0);
            value = 31 * value + (this.u128 != null ? this.u128.hashCode() : 0);
            value = 31 * value + (this.f32 != null ? this.f32.hashCode() : 0);
            value = 31 * value + (this.f64 != null ? this.f64.hashCode() : 0);
            value = 31 * value + (this.char != null ? this.char.hashCode() : 0);
            value = 31 * value + (this.string != null ? this.string.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public com.novi.serde.Unit unit;
            public Boolean bool;
            public Byte i8;
            public Short i16;
            public Integer i32;
            public Long i64;
            public java.math.@com.novi.serde.Int128 BigInteger i128;
            public @com.novi.serde.Unsigned Byte u8;
            public @com.novi.serde.Unsigned Short u16;
            public @com.novi.serde.Unsigned Integer u32;
            public @com.novi.serde.Unsigned Long u64;
            public java.math.@com.novi.serde.Unsigned @com.novi.serde.Int128 BigInteger u128;
            public Float f32;
            public Double f64;
            public Character char;
            public String string;

            public StructWithFields build() {
                return new StructWithFields(
                    unit,
                    bool,
                    i8,
                    i16,
                    i32,
                    i64,
                    i128,
                    u8,
                    u16,
                    u32,
                    u64,
                    u128,
                    f32,
                    f64,
                    char,
                    string
                );
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

    let actual = emit_java!(Outer as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class Inner1 {
        public final String field1;

        public Inner1(String field1) {
            java.util.Objects.requireNonNull(field1, "field1 must not be null");
            this.field1 = field1;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_str(field1);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static Inner1 deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.field1 = deserializer.deserialize_str();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static Inner1 jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            Inner1 value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            Inner1 other = (Inner1) obj;
            if (!java.util.Objects.equals(this.field1, other.field1)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.field1 != null ? this.field1.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String field1;

            public Inner1 build() {
                return new Inner1(
                    field1
                );
            }
        }
    }

    public final class Inner2 {
        public final String value;

        public Inner2(String value) {
            java.util.Objects.requireNonNull(value, "value must not be null");
            this.value = value;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_str(value);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static Inner2 deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.value = deserializer.deserialize_str();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static Inner2 jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            Inner2 value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            Inner2 other = (Inner2) obj;
            if (!java.util.Objects.equals(this.value, other.value)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.value != null ? this.value.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String value;

            public Inner2 build() {
                return new Inner2(
                    value
                );
            }
        }
    }

    public final class Inner3 {
        public final String field0;
        public final Integer field1;

        public Inner3(String field0, Integer field1) {
            java.util.Objects.requireNonNull(field0, "field0 must not be null");
            java.util.Objects.requireNonNull(field1, "field1 must not be null");
            this.field0 = field0;
            this.field1 = field1;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_str(field0);
            serializer.serialize_i32(field1);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static Inner3 deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.field0 = deserializer.deserialize_str();
            builder.field1 = deserializer.deserialize_i32();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static Inner3 jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            Inner3 value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            Inner3 other = (Inner3) obj;
            if (!java.util.Objects.equals(this.field0, other.field0)) { return false; }
            if (!java.util.Objects.equals(this.field1, other.field1)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.field0 != null ? this.field0.hashCode() : 0);
            value = 31 * value + (this.field1 != null ? this.field1.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String field0;
            public Integer field1;

            public Inner3 build() {
                return new Inner3(
                    field0,
                    field1
                );
            }
        }
    }

    public final class Outer {
        public final com.example.Inner1 one;
        public final com.example.Inner2 two;
        public final com.example.Inner3 three;

        public Outer(com.example.Inner1 one, com.example.Inner2 two, com.example.Inner3 three) {
            java.util.Objects.requireNonNull(one, "one must not be null");
            java.util.Objects.requireNonNull(two, "two must not be null");
            java.util.Objects.requireNonNull(three, "three must not be null");
            this.one = one;
            this.two = two;
            this.three = three;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            one.serialize(serializer);
            two.serialize(serializer);
            three.serialize(serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static Outer deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.one = com.example.Inner1.deserialize(deserializer);
            builder.two = com.example.Inner2.deserialize(deserializer);
            builder.three = com.example.Inner3.deserialize(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static Outer jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            Outer value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            Outer other = (Outer) obj;
            if (!java.util.Objects.equals(this.one, other.one)) { return false; }
            if (!java.util.Objects.equals(this.two, other.two)) { return false; }
            if (!java.util.Objects.equals(this.three, other.three)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.one != null ? this.one.hashCode() : 0);
            value = 31 * value + (this.two != null ? this.two.hashCode() : 0);
            value = 31 * value + (this.three != null ? this.three.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public com.example.Inner1 one;
            public com.example.Inner2 two;
            public com.example.Inner3 three;

            public Outer build() {
                return new Outer(
                    one,
                    two,
                    three
                );
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

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final com.novi.serde.Tuple2<String, Integer> one;

        public MyStruct(com.novi.serde.Tuple2<String, Integer> one) {
            java.util.Objects.requireNonNull(one, "one must not be null");
            this.one = one;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_tuple2_str_i32(one, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.one = com.example.TraitHelpers.deserialize_tuple2_str_i32(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.one, other.one)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.one != null ? this.one.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public com.novi.serde.Tuple2<String, Integer> one;

            public MyStruct build() {
                return new MyStruct(
                    one
                );
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

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final com.novi.serde.Tuple3<String, Integer, @com.novi.serde.Unsigned Short> one;

        public MyStruct(com.novi.serde.Tuple3<String, Integer, @com.novi.serde.Unsigned Short> one) {
            java.util.Objects.requireNonNull(one, "one must not be null");
            this.one = one;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_tuple3_str_i32_u16(one, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.one = com.example.TraitHelpers.deserialize_tuple3_str_i32_u16(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.one, other.one)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.one != null ? this.one.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public com.novi.serde.Tuple3<String, Integer, @com.novi.serde.Unsigned Short> one;

            public MyStruct build() {
                return new MyStruct(
                    one
                );
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

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final com.novi.serde.Tuple4<String, Integer, @com.novi.serde.Unsigned Short, Float> one;

        public MyStruct(com.novi.serde.Tuple4<String, Integer, @com.novi.serde.Unsigned Short, Float> one) {
            java.util.Objects.requireNonNull(one, "one must not be null");
            this.one = one;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_tuple4_str_i32_u16_f32(one, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.one = com.example.TraitHelpers.deserialize_tuple4_str_i32_u16_f32(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.one, other.one)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.one != null ? this.one.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public com.novi.serde.Tuple4<String, Integer, @com.novi.serde.Unsigned Short, Float> one;

            public MyStruct build() {
                return new MyStruct(
                    one
                );
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

    let actual = emit_java!(EnumWithUnitVariants as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public abstract class EnumWithUnitVariants {

        abstract public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError;

        public static EnumWithUnitVariants deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            int index = deserializer.deserialize_variant_index();
            switch (index) {
                case 0: return Variant1.load(deserializer);
                case 1: return Variant2.load(deserializer);
                case 2: return Variant3.load(deserializer);
                default: throw new com.novi.serde.DeserializationError("Unknown variant index for EnumWithUnitVariants: " + index);
            }
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static EnumWithUnitVariants jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            EnumWithUnitVariants value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public static final class Variant1 extends EnumWithUnitVariants {
            public Variant1() {
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(0);
                serializer.decrease_container_depth();
            }

            static Variant1 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant1 other = (Variant1) obj;
                return true;
            }

            public int hashCode() {
                int value = 7;
                return value;
            }

            public static final class Builder {
                public Variant1 build() {
                    return new Variant1(
                    );
                }
            }
        }

        public static final class Variant2 extends EnumWithUnitVariants {
            public Variant2() {
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(1);
                serializer.decrease_container_depth();
            }

            static Variant2 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant2 other = (Variant2) obj;
                return true;
            }

            public int hashCode() {
                int value = 7;
                return value;
            }

            public static final class Builder {
                public Variant2 build() {
                    return new Variant2(
                    );
                }
            }
        }

        public static final class Variant3 extends EnumWithUnitVariants {
            public Variant3() {
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(2);
                serializer.decrease_container_depth();
            }

            static Variant3 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant3 other = (Variant3) obj;
                return true;
            }

            public int hashCode() {
                int value = 7;
                return value;
            }

            public static final class Builder {
                public Variant3 build() {
                    return new Variant3(
                    );
                }
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

    let actual = emit_java!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public abstract class MyEnum {

        abstract public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError;

        public static MyEnum deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            int index = deserializer.deserialize_variant_index();
            switch (index) {
                case 0: return Variant1.load(deserializer);
                default: throw new com.novi.serde.DeserializationError("Unknown variant index for MyEnum: " + index);
            }
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyEnum jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyEnum value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public static final class Variant1 extends MyEnum {
            public Variant1() {
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(0);
                serializer.decrease_container_depth();
            }

            static Variant1 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant1 other = (Variant1) obj;
                return true;
            }

            public int hashCode() {
                int value = 7;
                return value;
            }

            public static final class Builder {
                public Variant1 build() {
                    return new Variant1(
                    );
                }
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

    let actual = emit_java!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public abstract class MyEnum {

        abstract public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError;

        public static MyEnum deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            int index = deserializer.deserialize_variant_index();
            switch (index) {
                case 0: return Variant1.load(deserializer);
                default: throw new com.novi.serde.DeserializationError("Unknown variant index for MyEnum: " + index);
            }
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyEnum jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyEnum value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public static final class Variant1 extends MyEnum {
            public final String value;

            public Variant1(String value) {
                java.util.Objects.requireNonNull(value, "value must not be null");
                this.value = value;
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(0);
                serializer.serialize_str(value);
                serializer.decrease_container_depth();
            }

            static Variant1 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                builder.value = deserializer.deserialize_str();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant1 other = (Variant1) obj;
                if (!java.util.Objects.equals(this.value, other.value)) { return false; }
                return true;
            }

            public int hashCode() {
                int value = 7;
                value = 31 * value + (this.value != null ? this.value.hashCode() : 0);
                return value;
            }

            public static final class Builder {
                public String value;

                public Variant1 build() {
                    return new Variant1(
                        value
                    );
                }
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

    let actual = emit_java!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public abstract class MyEnum {

        abstract public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError;

        public static MyEnum deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            int index = deserializer.deserialize_variant_index();
            switch (index) {
                case 0: return Variant1.load(deserializer);
                case 1: return Variant2.load(deserializer);
                default: throw new com.novi.serde.DeserializationError("Unknown variant index for MyEnum: " + index);
            }
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyEnum jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyEnum value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public static final class Variant1 extends MyEnum {
            public final String value;

            public Variant1(String value) {
                java.util.Objects.requireNonNull(value, "value must not be null");
                this.value = value;
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(0);
                serializer.serialize_str(value);
                serializer.decrease_container_depth();
            }

            static Variant1 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                builder.value = deserializer.deserialize_str();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant1 other = (Variant1) obj;
                if (!java.util.Objects.equals(this.value, other.value)) { return false; }
                return true;
            }

            public int hashCode() {
                int value = 7;
                value = 31 * value + (this.value != null ? this.value.hashCode() : 0);
                return value;
            }

            public static final class Builder {
                public String value;

                public Variant1 build() {
                    return new Variant1(
                        value
                    );
                }
            }
        }

        public static final class Variant2 extends MyEnum {
            public final Integer value;

            public Variant2(Integer value) {
                java.util.Objects.requireNonNull(value, "value must not be null");
                this.value = value;
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(1);
                serializer.serialize_i32(value);
                serializer.decrease_container_depth();
            }

            static Variant2 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                builder.value = deserializer.deserialize_i32();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant2 other = (Variant2) obj;
                if (!java.util.Objects.equals(this.value, other.value)) { return false; }
                return true;
            }

            public int hashCode() {
                int value = 7;
                value = 31 * value + (this.value != null ? this.value.hashCode() : 0);
                return value;
            }

            public static final class Builder {
                public Integer value;

                public Variant2 build() {
                    return new Variant2(
                        value
                    );
                }
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

    let actual = emit_java!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public abstract class MyEnum {

        abstract public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError;

        public static MyEnum deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            int index = deserializer.deserialize_variant_index();
            switch (index) {
                case 0: return Variant1.load(deserializer);
                case 1: return Variant2.load(deserializer);
                default: throw new com.novi.serde.DeserializationError("Unknown variant index for MyEnum: " + index);
            }
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyEnum jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyEnum value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public static final class Variant1 extends MyEnum {
            public final String field0;
            public final Integer field1;

            public Variant1(String field0, Integer field1) {
                java.util.Objects.requireNonNull(field0, "field0 must not be null");
                java.util.Objects.requireNonNull(field1, "field1 must not be null");
                this.field0 = field0;
                this.field1 = field1;
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(0);
                serializer.serialize_str(field0);
                serializer.serialize_i32(field1);
                serializer.decrease_container_depth();
            }

            static Variant1 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                builder.field0 = deserializer.deserialize_str();
                builder.field1 = deserializer.deserialize_i32();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant1 other = (Variant1) obj;
                if (!java.util.Objects.equals(this.field0, other.field0)) { return false; }
                if (!java.util.Objects.equals(this.field1, other.field1)) { return false; }
                return true;
            }

            public int hashCode() {
                int value = 7;
                value = 31 * value + (this.field0 != null ? this.field0.hashCode() : 0);
                value = 31 * value + (this.field1 != null ? this.field1.hashCode() : 0);
                return value;
            }

            public static final class Builder {
                public String field0;
                public Integer field1;

                public Variant1 build() {
                    return new Variant1(
                        field0,
                        field1
                    );
                }
            }
        }

        public static final class Variant2 extends MyEnum {
            public final Boolean field0;
            public final Double field1;
            public final @com.novi.serde.Unsigned Byte field2;

            public Variant2(Boolean field0, Double field1, @com.novi.serde.Unsigned Byte field2) {
                java.util.Objects.requireNonNull(field0, "field0 must not be null");
                java.util.Objects.requireNonNull(field1, "field1 must not be null");
                java.util.Objects.requireNonNull(field2, "field2 must not be null");
                this.field0 = field0;
                this.field1 = field1;
                this.field2 = field2;
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(1);
                serializer.serialize_bool(field0);
                serializer.serialize_f64(field1);
                serializer.serialize_u8(field2);
                serializer.decrease_container_depth();
            }

            static Variant2 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                builder.field0 = deserializer.deserialize_bool();
                builder.field1 = deserializer.deserialize_f64();
                builder.field2 = deserializer.deserialize_u8();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant2 other = (Variant2) obj;
                if (!java.util.Objects.equals(this.field0, other.field0)) { return false; }
                if (!java.util.Objects.equals(this.field1, other.field1)) { return false; }
                if (!java.util.Objects.equals(this.field2, other.field2)) { return false; }
                return true;
            }

            public int hashCode() {
                int value = 7;
                value = 31 * value + (this.field0 != null ? this.field0.hashCode() : 0);
                value = 31 * value + (this.field1 != null ? this.field1.hashCode() : 0);
                value = 31 * value + (this.field2 != null ? this.field2.hashCode() : 0);
                return value;
            }

            public static final class Builder {
                public Boolean field0;
                public Double field1;
                public @com.novi.serde.Unsigned Byte field2;

                public Variant2 build() {
                    return new Variant2(
                        field0,
                        field1,
                        field2
                    );
                }
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

    let actual = emit_java!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public abstract class MyEnum {

        abstract public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError;

        public static MyEnum deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            int index = deserializer.deserialize_variant_index();
            switch (index) {
                case 0: return Variant1.load(deserializer);
                default: throw new com.novi.serde.DeserializationError("Unknown variant index for MyEnum: " + index);
            }
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyEnum jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyEnum value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public static final class Variant1 extends MyEnum {
            public final String field1;
            public final Integer field2;

            public Variant1(String field1, Integer field2) {
                java.util.Objects.requireNonNull(field1, "field1 must not be null");
                java.util.Objects.requireNonNull(field2, "field2 must not be null");
                this.field1 = field1;
                this.field2 = field2;
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(0);
                serializer.serialize_str(field1);
                serializer.serialize_i32(field2);
                serializer.decrease_container_depth();
            }

            static Variant1 load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                builder.field1 = deserializer.deserialize_str();
                builder.field2 = deserializer.deserialize_i32();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Variant1 other = (Variant1) obj;
                if (!java.util.Objects.equals(this.field1, other.field1)) { return false; }
                if (!java.util.Objects.equals(this.field2, other.field2)) { return false; }
                return true;
            }

            public int hashCode() {
                int value = 7;
                value = 31 * value + (this.field1 != null ? this.field1.hashCode() : 0);
                value = 31 * value + (this.field2 != null ? this.field2.hashCode() : 0);
                return value;
            }

            public static final class Builder {
                public String field1;
                public Integer field2;

                public Variant1 build() {
                    return new Variant1(
                        field1,
                        field2
                    );
                }
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

    let actual = emit_java!(MyEnum as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public abstract class MyEnum {

        abstract public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError;

        public static MyEnum deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            int index = deserializer.deserialize_variant_index();
            switch (index) {
                case 0: return Unit.load(deserializer);
                case 1: return NewType.load(deserializer);
                case 2: return Tuple.load(deserializer);
                case 3: return Struct.load(deserializer);
                default: throw new com.novi.serde.DeserializationError("Unknown variant index for MyEnum: " + index);
            }
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyEnum jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyEnum value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public static final class Unit extends MyEnum {
            public Unit() {
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(0);
                serializer.decrease_container_depth();
            }

            static Unit load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Unit other = (Unit) obj;
                return true;
            }

            public int hashCode() {
                int value = 7;
                return value;
            }

            public static final class Builder {
                public Unit build() {
                    return new Unit(
                    );
                }
            }
        }

        public static final class NewType extends MyEnum {
            public final String value;

            public NewType(String value) {
                java.util.Objects.requireNonNull(value, "value must not be null");
                this.value = value;
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(1);
                serializer.serialize_str(value);
                serializer.decrease_container_depth();
            }

            static NewType load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                builder.value = deserializer.deserialize_str();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                NewType other = (NewType) obj;
                if (!java.util.Objects.equals(this.value, other.value)) { return false; }
                return true;
            }

            public int hashCode() {
                int value = 7;
                value = 31 * value + (this.value != null ? this.value.hashCode() : 0);
                return value;
            }

            public static final class Builder {
                public String value;

                public NewType build() {
                    return new NewType(
                        value
                    );
                }
            }
        }

        public static final class Tuple extends MyEnum {
            public final String field0;
            public final Integer field1;

            public Tuple(String field0, Integer field1) {
                java.util.Objects.requireNonNull(field0, "field0 must not be null");
                java.util.Objects.requireNonNull(field1, "field1 must not be null");
                this.field0 = field0;
                this.field1 = field1;
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(2);
                serializer.serialize_str(field0);
                serializer.serialize_i32(field1);
                serializer.decrease_container_depth();
            }

            static Tuple load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                builder.field0 = deserializer.deserialize_str();
                builder.field1 = deserializer.deserialize_i32();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Tuple other = (Tuple) obj;
                if (!java.util.Objects.equals(this.field0, other.field0)) { return false; }
                if (!java.util.Objects.equals(this.field1, other.field1)) { return false; }
                return true;
            }

            public int hashCode() {
                int value = 7;
                value = 31 * value + (this.field0 != null ? this.field0.hashCode() : 0);
                value = 31 * value + (this.field1 != null ? this.field1.hashCode() : 0);
                return value;
            }

            public static final class Builder {
                public String field0;
                public Integer field1;

                public Tuple build() {
                    return new Tuple(
                        field0,
                        field1
                    );
                }
            }
        }

        public static final class Struct extends MyEnum {
            public final Boolean field;

            public Struct(Boolean field) {
                java.util.Objects.requireNonNull(field, "field must not be null");
                this.field = field;
            }

            public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(3);
                serializer.serialize_bool(field);
                serializer.decrease_container_depth();
            }

            static Struct load(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
                deserializer.increase_container_depth();
                Builder builder = new Builder();
                builder.field = deserializer.deserialize_bool();
                deserializer.decrease_container_depth();
                return builder.build();
            }

            public boolean equals(Object obj) {
                if (this == obj) return true;
                if (obj == null) return false;
                if (getClass() != obj.getClass()) return false;
                Struct other = (Struct) obj;
                if (!java.util.Objects.equals(this.field, other.field)) { return false; }
                return true;
            }

            public int hashCode() {
                int value = 7;
                value = 31 * value + (this.field != null ? this.field.hashCode() : 0);
                return value;
            }

            public static final class Builder {
                public Boolean field;

                public Struct build() {
                    return new Struct(
                        field
                    );
                }
            }
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

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final java.util.List<String> items;
        public final java.util.List<Integer> numbers;
        public final java.util.List<java.util.List<String>> nested_items;

        public MyStruct(java.util.List<String> items, java.util.List<Integer> numbers, java.util.List<java.util.List<String>> nested_items) {
            java.util.Objects.requireNonNull(items, "items must not be null");
            java.util.Objects.requireNonNull(numbers, "numbers must not be null");
            java.util.Objects.requireNonNull(nested_items, "nested_items must not be null");
            this.items = items;
            this.numbers = numbers;
            this.nested_items = nested_items;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_vector_str(items, serializer);
            com.example.TraitHelpers.serialize_vector_i32(numbers, serializer);
            com.example.TraitHelpers.serialize_vector_vector_str(nested_items, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.items = com.example.TraitHelpers.deserialize_vector_str(deserializer);
            builder.numbers = com.example.TraitHelpers.deserialize_vector_i32(deserializer);
            builder.nested_items = com.example.TraitHelpers.deserialize_vector_vector_str(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.items, other.items)) { return false; }
            if (!java.util.Objects.equals(this.numbers, other.numbers)) { return false; }
            if (!java.util.Objects.equals(this.nested_items, other.nested_items)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.items != null ? this.items.hashCode() : 0);
            value = 31 * value + (this.numbers != null ? this.numbers.hashCode() : 0);
            value = 31 * value + (this.nested_items != null ? this.nested_items.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public java.util.List<String> items;
            public java.util.List<Integer> numbers;
            public java.util.List<java.util.List<String>> nested_items;

            public MyStruct build() {
                return new MyStruct(
                    items,
                    numbers,
                    nested_items
                );
            }
        }
    }
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

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final java.util.Optional<String> optional_string;
        public final java.util.Optional<Integer> optional_number;
        public final java.util.Optional<Boolean> optional_bool;

        public MyStruct(java.util.Optional<String> optional_string, java.util.Optional<Integer> optional_number, java.util.Optional<Boolean> optional_bool) {
            java.util.Objects.requireNonNull(optional_string, "optional_string must not be null");
            java.util.Objects.requireNonNull(optional_number, "optional_number must not be null");
            java.util.Objects.requireNonNull(optional_bool, "optional_bool must not be null");
            this.optional_string = optional_string;
            this.optional_number = optional_number;
            this.optional_bool = optional_bool;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_option_str(optional_string, serializer);
            com.example.TraitHelpers.serialize_option_i32(optional_number, serializer);
            com.example.TraitHelpers.serialize_option_bool(optional_bool, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.optional_string = com.example.TraitHelpers.deserialize_option_str(deserializer);
            builder.optional_number = com.example.TraitHelpers.deserialize_option_i32(deserializer);
            builder.optional_bool = com.example.TraitHelpers.deserialize_option_bool(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.optional_string, other.optional_string)) { return false; }
            if (!java.util.Objects.equals(this.optional_number, other.optional_number)) { return false; }
            if (!java.util.Objects.equals(this.optional_bool, other.optional_bool)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.optional_string != null ? this.optional_string.hashCode() : 0);
            value = 31 * value + (this.optional_number != null ? this.optional_number.hashCode() : 0);
            value = 31 * value + (this.optional_bool != null ? this.optional_bool.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public java.util.Optional<String> optional_string;
            public java.util.Optional<Integer> optional_number;
            public java.util.Optional<Boolean> optional_bool;

            public MyStruct build() {
                return new MyStruct(
                    optional_string,
                    optional_number,
                    optional_bool
                );
            }
        }
    }
    "#);
}

#[test]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: std::collections::HashMap<String, i32>,
        int_to_bool: std::collections::HashMap<i32, bool>,
    }

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final java.util.Map<String, Integer> string_to_int;
        public final java.util.Map<Integer, Boolean> int_to_bool;

        public MyStruct(java.util.Map<String, Integer> string_to_int, java.util.Map<Integer, Boolean> int_to_bool) {
            java.util.Objects.requireNonNull(string_to_int, "string_to_int must not be null");
            java.util.Objects.requireNonNull(int_to_bool, "int_to_bool must not be null");
            this.string_to_int = string_to_int;
            this.int_to_bool = int_to_bool;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_map_str_to_i32(string_to_int, serializer);
            com.example.TraitHelpers.serialize_map_i32_to_bool(int_to_bool, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.string_to_int = com.example.TraitHelpers.deserialize_map_str_to_i32(deserializer);
            builder.int_to_bool = com.example.TraitHelpers.deserialize_map_i32_to_bool(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.string_to_int, other.string_to_int)) { return false; }
            if (!java.util.Objects.equals(this.int_to_bool, other.int_to_bool)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.string_to_int != null ? this.string_to_int.hashCode() : 0);
            value = 31 * value + (this.int_to_bool != null ? this.int_to_bool.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public java.util.Map<String, Integer> string_to_int;
            public java.util.Map<Integer, Boolean> int_to_bool;

            public MyStruct build() {
                return new MyStruct(
                    string_to_int,
                    int_to_bool
                );
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
        map_to_list: std::collections::HashMap<String, Vec<bool>>,
        optional_map: Option<std::collections::HashMap<String, i32>>,
        complex: Vec<Option<std::collections::HashMap<String, Vec<bool>>>>,
    }

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final java.util.Optional<java.util.List<String>> optional_list;
        public final java.util.List<java.util.Optional<Integer>> list_of_optionals;
        public final java.util.Map<String, java.util.List<Boolean>> map_to_list;
        public final java.util.Optional<java.util.Map<String, Integer>> optional_map;
        public final java.util.List<java.util.Optional<java.util.Map<String, java.util.List<Boolean>>>> complex;

        public MyStruct(java.util.Optional<java.util.List<String>> optional_list, java.util.List<java.util.Optional<Integer>> list_of_optionals, java.util.Map<String, java.util.List<Boolean>> map_to_list, java.util.Optional<java.util.Map<String, Integer>> optional_map, java.util.List<java.util.Optional<java.util.Map<String, java.util.List<Boolean>>>> complex) {
            java.util.Objects.requireNonNull(optional_list, "optional_list must not be null");
            java.util.Objects.requireNonNull(list_of_optionals, "list_of_optionals must not be null");
            java.util.Objects.requireNonNull(map_to_list, "map_to_list must not be null");
            java.util.Objects.requireNonNull(optional_map, "optional_map must not be null");
            java.util.Objects.requireNonNull(complex, "complex must not be null");
            this.optional_list = optional_list;
            this.list_of_optionals = list_of_optionals;
            this.map_to_list = map_to_list;
            this.optional_map = optional_map;
            this.complex = complex;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_option_vector_str(optional_list, serializer);
            com.example.TraitHelpers.serialize_vector_option_i32(list_of_optionals, serializer);
            com.example.TraitHelpers.serialize_map_str_to_vector_bool(map_to_list, serializer);
            com.example.TraitHelpers.serialize_option_map_str_to_i32(optional_map, serializer);
            com.example.TraitHelpers.serialize_vector_option_map_str_to_vector_bool(complex, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.optional_list = com.example.TraitHelpers.deserialize_option_vector_str(deserializer);
            builder.list_of_optionals = com.example.TraitHelpers.deserialize_vector_option_i32(deserializer);
            builder.map_to_list = com.example.TraitHelpers.deserialize_map_str_to_vector_bool(deserializer);
            builder.optional_map = com.example.TraitHelpers.deserialize_option_map_str_to_i32(deserializer);
            builder.complex = com.example.TraitHelpers.deserialize_vector_option_map_str_to_vector_bool(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.optional_list, other.optional_list)) { return false; }
            if (!java.util.Objects.equals(this.list_of_optionals, other.list_of_optionals)) { return false; }
            if (!java.util.Objects.equals(this.map_to_list, other.map_to_list)) { return false; }
            if (!java.util.Objects.equals(this.optional_map, other.optional_map)) { return false; }
            if (!java.util.Objects.equals(this.complex, other.complex)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.optional_list != null ? this.optional_list.hashCode() : 0);
            value = 31 * value + (this.list_of_optionals != null ? this.list_of_optionals.hashCode() : 0);
            value = 31 * value + (this.map_to_list != null ? this.map_to_list.hashCode() : 0);
            value = 31 * value + (this.optional_map != null ? this.optional_map.hashCode() : 0);
            value = 31 * value + (this.complex != null ? this.complex.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public java.util.Optional<java.util.List<String>> optional_list;
            public java.util.List<java.util.Optional<Integer>> list_of_optionals;
            public java.util.Map<String, java.util.List<Boolean>> map_to_list;
            public java.util.Optional<java.util.Map<String, Integer>> optional_map;
            public java.util.List<java.util.Optional<java.util.Map<String, java.util.List<Boolean>>>> complex;

            public MyStruct build() {
                return new MyStruct(
                    optional_list,
                    list_of_optionals,
                    map_to_list,
                    optional_map,
                    complex
                );
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

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final java.util.@com.novi.serde.ArrayLen(length=5) List<Integer> fixed_array;
        public final java.util.@com.novi.serde.ArrayLen(length=32) List<@com.novi.serde.Unsigned Byte> byte_array;
        public final java.util.@com.novi.serde.ArrayLen(length=3) List<String> string_array;

        public MyStruct(java.util.@com.novi.serde.ArrayLen(length=5) List<Integer> fixed_array, java.util.@com.novi.serde.ArrayLen(length=32) List<@com.novi.serde.Unsigned Byte> byte_array, java.util.@com.novi.serde.ArrayLen(length=3) List<String> string_array) {
            java.util.Objects.requireNonNull(fixed_array, "fixed_array must not be null");
            java.util.Objects.requireNonNull(byte_array, "byte_array must not be null");
            java.util.Objects.requireNonNull(string_array, "string_array must not be null");
            this.fixed_array = fixed_array;
            this.byte_array = byte_array;
            this.string_array = string_array;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_array5_i32_array(fixed_array, serializer);
            com.example.TraitHelpers.serialize_array32_u8_array(byte_array, serializer);
            com.example.TraitHelpers.serialize_array3_str_array(string_array, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.fixed_array = com.example.TraitHelpers.deserialize_array5_i32_array(deserializer);
            builder.byte_array = com.example.TraitHelpers.deserialize_array32_u8_array(deserializer);
            builder.string_array = com.example.TraitHelpers.deserialize_array3_str_array(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.fixed_array, other.fixed_array)) { return false; }
            if (!java.util.Objects.equals(this.byte_array, other.byte_array)) { return false; }
            if (!java.util.Objects.equals(this.string_array, other.string_array)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.fixed_array != null ? this.fixed_array.hashCode() : 0);
            value = 31 * value + (this.byte_array != null ? this.byte_array.hashCode() : 0);
            value = 31 * value + (this.string_array != null ? this.string_array.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public java.util.@com.novi.serde.ArrayLen(length=5) List<Integer> fixed_array;
            public java.util.@com.novi.serde.ArrayLen(length=32) List<@com.novi.serde.Unsigned Byte> byte_array;
            public java.util.@com.novi.serde.ArrayLen(length=3) List<String> string_array;

            public MyStruct build() {
                return new MyStruct(
                    fixed_array,
                    byte_array,
                    string_array
                );
            }
        }
    }
    "#);
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: std::collections::BTreeMap<String, i32>,
        int_to_bool: std::collections::BTreeMap<i32, bool>,
    }

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final java.util.Map<String, Integer> string_to_int;
        public final java.util.Map<Integer, Boolean> int_to_bool;

        public MyStruct(java.util.Map<String, Integer> string_to_int, java.util.Map<Integer, Boolean> int_to_bool) {
            java.util.Objects.requireNonNull(string_to_int, "string_to_int must not be null");
            java.util.Objects.requireNonNull(int_to_bool, "int_to_bool must not be null");
            this.string_to_int = string_to_int;
            this.int_to_bool = int_to_bool;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_map_str_to_i32(string_to_int, serializer);
            com.example.TraitHelpers.serialize_map_i32_to_bool(int_to_bool, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.string_to_int = com.example.TraitHelpers.deserialize_map_str_to_i32(deserializer);
            builder.int_to_bool = com.example.TraitHelpers.deserialize_map_i32_to_bool(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.string_to_int, other.string_to_int)) { return false; }
            if (!java.util.Objects.equals(this.int_to_bool, other.int_to_bool)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.string_to_int != null ? this.string_to_int.hashCode() : 0);
            value = 31 * value + (this.int_to_bool != null ? this.int_to_bool.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public java.util.Map<String, Integer> string_to_int;
            public java.util.Map<Integer, Boolean> int_to_bool;

            public MyStruct build() {
                return new MyStruct(
                    string_to_int,
                    int_to_bool
                );
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
        string_set: std::collections::HashSet<String>,
        int_set: std::collections::HashSet<i32>,
    }

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final java.util.List<String> string_set;
        public final java.util.List<Integer> int_set;

        public MyStruct(java.util.List<String> string_set, java.util.List<Integer> int_set) {
            java.util.Objects.requireNonNull(string_set, "string_set must not be null");
            java.util.Objects.requireNonNull(int_set, "int_set must not be null");
            this.string_set = string_set;
            this.int_set = int_set;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_set_str(string_set, serializer);
            com.example.TraitHelpers.serialize_set_i32(int_set, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.string_set = com.example.TraitHelpers.deserialize_set_str(deserializer);
            builder.int_set = com.example.TraitHelpers.deserialize_set_i32(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.string_set, other.string_set)) { return false; }
            if (!java.util.Objects.equals(this.int_set, other.int_set)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.string_set != null ? this.string_set.hashCode() : 0);
            value = 31 * value + (this.int_set != null ? this.int_set.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public java.util.List<String> string_set;
            public java.util.List<Integer> int_set;

            public MyStruct build() {
                return new MyStruct(
                    string_set,
                    int_set
                );
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
        string_set: std::collections::BTreeSet<String>,
        int_set: std::collections::BTreeSet<i32>,
    }

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final java.util.List<String> string_set;
        public final java.util.List<Integer> int_set;

        public MyStruct(java.util.List<String> string_set, java.util.List<Integer> int_set) {
            java.util.Objects.requireNonNull(string_set, "string_set must not be null");
            java.util.Objects.requireNonNull(int_set, "int_set must not be null");
            this.string_set = string_set;
            this.int_set = int_set;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_set_str(string_set, serializer);
            com.example.TraitHelpers.serialize_set_i32(int_set, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.string_set = com.example.TraitHelpers.deserialize_set_str(deserializer);
            builder.int_set = com.example.TraitHelpers.deserialize_set_i32(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.string_set, other.string_set)) { return false; }
            if (!java.util.Objects.equals(this.int_set, other.int_set)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.string_set != null ? this.string_set.hashCode() : 0);
            value = 31 * value + (this.int_set != null ? this.int_set.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public java.util.List<String> string_set;
            public java.util.List<Integer> int_set;

            public MyStruct build() {
                return new MyStruct(
                    string_set,
                    int_set
                );
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

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final String boxed_string;
        public final Integer boxed_int;

        public MyStruct(String boxed_string, Integer boxed_int) {
            java.util.Objects.requireNonNull(boxed_string, "boxed_string must not be null");
            java.util.Objects.requireNonNull(boxed_int, "boxed_int must not be null");
            this.boxed_string = boxed_string;
            this.boxed_int = boxed_int;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_str(boxed_string);
            serializer.serialize_i32(boxed_int);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.boxed_string = deserializer.deserialize_str();
            builder.boxed_int = deserializer.deserialize_i32();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.boxed_string, other.boxed_string)) { return false; }
            if (!java.util.Objects.equals(this.boxed_int, other.boxed_int)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.boxed_string != null ? this.boxed_string.hashCode() : 0);
            value = 31 * value + (this.boxed_int != null ? this.boxed_int.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String boxed_string;
            public Integer boxed_int;

            public MyStruct build() {
                return new MyStruct(
                    boxed_string,
                    boxed_int
                );
            }
        }
    }
    "#);
}

#[test]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: std::rc::Rc<String>,
        rc_int: std::rc::Rc<i32>,
    }

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final String rc_string;
        public final Integer rc_int;

        public MyStruct(String rc_string, Integer rc_int) {
            java.util.Objects.requireNonNull(rc_string, "rc_string must not be null");
            java.util.Objects.requireNonNull(rc_int, "rc_int must not be null");
            this.rc_string = rc_string;
            this.rc_int = rc_int;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_str(rc_string);
            serializer.serialize_i32(rc_int);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.rc_string = deserializer.deserialize_str();
            builder.rc_int = deserializer.deserialize_i32();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.rc_string, other.rc_string)) { return false; }
            if (!java.util.Objects.equals(this.rc_int, other.rc_int)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.rc_string != null ? this.rc_string.hashCode() : 0);
            value = 31 * value + (this.rc_int != null ? this.rc_int.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String rc_string;
            public Integer rc_int;

            public MyStruct build() {
                return new MyStruct(
                    rc_string,
                    rc_int
                );
            }
        }
    }
    "#);
}

#[test]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: std::sync::Arc<String>,
        arc_int: std::sync::Arc<i32>,
    }

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final String arc_string;
        public final Integer arc_int;

        public MyStruct(String arc_string, Integer arc_int) {
            java.util.Objects.requireNonNull(arc_string, "arc_string must not be null");
            java.util.Objects.requireNonNull(arc_int, "arc_int must not be null");
            this.arc_string = arc_string;
            this.arc_int = arc_int;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_str(arc_string);
            serializer.serialize_i32(arc_int);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.arc_string = deserializer.deserialize_str();
            builder.arc_int = deserializer.deserialize_i32();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.arc_string, other.arc_string)) { return false; }
            if (!java.util.Objects.equals(this.arc_int, other.arc_int)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.arc_string != null ? this.arc_string.hashCode() : 0);
            value = 31 * value + (this.arc_int != null ? this.arc_int.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public String arc_string;
            public Integer arc_int;

            public MyStruct build() {
                return new MyStruct(
                    arc_string,
                    arc_int
                );
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
        vec_of_sets: Vec<std::collections::HashSet<String>>,
        optional_btree: Option<std::collections::BTreeMap<String, i32>>,
        boxed_vec: Box<Vec<String>>,
        arc_option: std::sync::Arc<Option<String>>,
        array_of_boxes: [Box<i32>; 3],
    }

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final java.util.List<java.util.List<String>> vec_of_sets;
        public final java.util.Optional<java.util.Map<String, Integer>> optional_btree;
        public final java.util.List<String> boxed_vec;
        public final java.util.Optional<String> arc_option;
        public final java.util.@com.novi.serde.ArrayLen(length=3) List<Integer> array_of_boxes;

        public MyStruct(java.util.List<java.util.List<String>> vec_of_sets, java.util.Optional<java.util.Map<String, Integer>> optional_btree, java.util.List<String> boxed_vec, java.util.Optional<String> arc_option, java.util.@com.novi.serde.ArrayLen(length=3) List<Integer> array_of_boxes) {
            java.util.Objects.requireNonNull(vec_of_sets, "vec_of_sets must not be null");
            java.util.Objects.requireNonNull(optional_btree, "optional_btree must not be null");
            java.util.Objects.requireNonNull(boxed_vec, "boxed_vec must not be null");
            java.util.Objects.requireNonNull(arc_option, "arc_option must not be null");
            java.util.Objects.requireNonNull(array_of_boxes, "array_of_boxes must not be null");
            this.vec_of_sets = vec_of_sets;
            this.optional_btree = optional_btree;
            this.boxed_vec = boxed_vec;
            this.arc_option = arc_option;
            this.array_of_boxes = array_of_boxes;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            com.example.TraitHelpers.serialize_vector_set_str(vec_of_sets, serializer);
            com.example.TraitHelpers.serialize_option_map_str_to_i32(optional_btree, serializer);
            com.example.TraitHelpers.serialize_vector_str(boxed_vec, serializer);
            com.example.TraitHelpers.serialize_option_str(arc_option, serializer);
            com.example.TraitHelpers.serialize_array3_i32_array(array_of_boxes, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.vec_of_sets = com.example.TraitHelpers.deserialize_vector_set_str(deserializer);
            builder.optional_btree = com.example.TraitHelpers.deserialize_option_map_str_to_i32(deserializer);
            builder.boxed_vec = com.example.TraitHelpers.deserialize_vector_str(deserializer);
            builder.arc_option = com.example.TraitHelpers.deserialize_option_str(deserializer);
            builder.array_of_boxes = com.example.TraitHelpers.deserialize_array3_i32_array(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.vec_of_sets, other.vec_of_sets)) { return false; }
            if (!java.util.Objects.equals(this.optional_btree, other.optional_btree)) { return false; }
            if (!java.util.Objects.equals(this.boxed_vec, other.boxed_vec)) { return false; }
            if (!java.util.Objects.equals(this.arc_option, other.arc_option)) { return false; }
            if (!java.util.Objects.equals(this.array_of_boxes, other.array_of_boxes)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.vec_of_sets != null ? this.vec_of_sets.hashCode() : 0);
            value = 31 * value + (this.optional_btree != null ? this.optional_btree.hashCode() : 0);
            value = 31 * value + (this.boxed_vec != null ? this.boxed_vec.hashCode() : 0);
            value = 31 * value + (this.arc_option != null ? this.arc_option.hashCode() : 0);
            value = 31 * value + (this.array_of_boxes != null ? this.array_of_boxes.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public java.util.List<java.util.List<String>> vec_of_sets;
            public java.util.Optional<java.util.Map<String, Integer>> optional_btree;
            public java.util.List<String> boxed_vec;
            public java.util.Optional<String> arc_option;
            public java.util.@com.novi.serde.ArrayLen(length=3) List<Integer> array_of_boxes;

            public MyStruct build() {
                return new MyStruct(
                    vec_of_sets,
                    optional_btree,
                    boxed_vec,
                    arc_option,
                    array_of_boxes
                );
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

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final com.novi.serde.Bytes data;
        public final String name;
        public final com.novi.serde.Bytes header;

        public MyStruct(com.novi.serde.Bytes data, String name, com.novi.serde.Bytes header) {
            java.util.Objects.requireNonNull(data, "data must not be null");
            java.util.Objects.requireNonNull(name, "name must not be null");
            java.util.Objects.requireNonNull(header, "header must not be null");
            this.data = data;
            this.name = name;
            this.header = header;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_bytes(data);
            serializer.serialize_str(name);
            serializer.serialize_bytes(header);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.data = deserializer.deserialize_bytes();
            builder.name = deserializer.deserialize_str();
            builder.header = deserializer.deserialize_bytes();
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.data, other.data)) { return false; }
            if (!java.util.Objects.equals(this.name, other.name)) { return false; }
            if (!java.util.Objects.equals(this.header, other.header)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.data != null ? this.data.hashCode() : 0);
            value = 31 * value + (this.name != null ? this.name.hashCode() : 0);
            value = 31 * value + (this.header != null ? this.header.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public com.novi.serde.Bytes data;
            public String name;
            public com.novi.serde.Bytes header;

            public MyStruct build() {
                return new MyStruct(
                    data,
                    name,
                    header
                );
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

    let actual = emit_java!(MyStruct as Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public final class MyStruct {
        public final com.novi.serde.Bytes data;
        public final String name;
        public final com.novi.serde.Bytes header;
        public final java.util.Optional<java.util.List<@com.novi.serde.Unsigned Byte>> optional_bytes;

        public MyStruct(com.novi.serde.Bytes data, String name, com.novi.serde.Bytes header, java.util.Optional<java.util.List<@com.novi.serde.Unsigned Byte>> optional_bytes) {
            java.util.Objects.requireNonNull(data, "data must not be null");
            java.util.Objects.requireNonNull(name, "name must not be null");
            java.util.Objects.requireNonNull(header, "header must not be null");
            java.util.Objects.requireNonNull(optional_bytes, "optional_bytes must not be null");
            this.data = data;
            this.name = name;
            this.header = header;
            this.optional_bytes = optional_bytes;
        }

        public void serialize(com.novi.serde.Serializer serializer) throws com.novi.serde.SerializationError {
            serializer.increase_container_depth();
            serializer.serialize_bytes(data);
            serializer.serialize_str(name);
            serializer.serialize_bytes(header);
            com.example.TraitHelpers.serialize_option_vector_u8(optional_bytes, serializer);
            serializer.decrease_container_depth();
        }

        public byte[] jsonSerialize() throws com.novi.serde.SerializationError {
            com.novi.serde.Serializer serializer = new com.novi.json.JsonSerializer();
            serialize(serializer);
            return serializer.get_bytes();
        }

        public static MyStruct deserialize(com.novi.serde.Deserializer deserializer) throws com.novi.serde.DeserializationError {
            deserializer.increase_container_depth();
            Builder builder = new Builder();
            builder.data = deserializer.deserialize_bytes();
            builder.name = deserializer.deserialize_str();
            builder.header = deserializer.deserialize_bytes();
            builder.optional_bytes = com.example.TraitHelpers.deserialize_option_vector_u8(deserializer);
            deserializer.decrease_container_depth();
            return builder.build();
        }

        public static MyStruct jsonDeserialize(byte[] input) throws com.novi.serde.DeserializationError {
            if (input == null) {
                 throw new com.novi.serde.DeserializationError("Cannot deserialize null array");
            }
            com.novi.serde.Deserializer deserializer = new com.novi.json.JsonDeserializer(input);
            MyStruct value = deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.length) {
                 throw new com.novi.serde.DeserializationError("Some input bytes were not read");
            }
            return value;
        }

        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (obj == null) return false;
            if (getClass() != obj.getClass()) return false;
            MyStruct other = (MyStruct) obj;
            if (!java.util.Objects.equals(this.data, other.data)) { return false; }
            if (!java.util.Objects.equals(this.name, other.name)) { return false; }
            if (!java.util.Objects.equals(this.header, other.header)) { return false; }
            if (!java.util.Objects.equals(this.optional_bytes, other.optional_bytes)) { return false; }
            return true;
        }

        public int hashCode() {
            int value = 7;
            value = 31 * value + (this.data != null ? this.data.hashCode() : 0);
            value = 31 * value + (this.name != null ? this.name.hashCode() : 0);
            value = 31 * value + (this.header != null ? this.header.hashCode() : 0);
            value = 31 * value + (this.optional_bytes != null ? this.optional_bytes.hashCode() : 0);
            return value;
        }

        public static final class Builder {
            public com.novi.serde.Bytes data;
            public String name;
            public com.novi.serde.Bytes header;
            public java.util.Optional<java.util.List<@com.novi.serde.Unsigned Byte>> optional_bytes;

            public MyStruct build() {
                return new MyStruct(
                    data,
                    name,
                    header,
                    optional_bytes
                );
            }
        }
    }
    "#);
}
