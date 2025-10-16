//! these tests will be updated once the Swift emitter is converted
//! to use the new Emitter<Language> trait
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

    let actual = emit!(UnitStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    public struct UnitStruct: Hashable {
        public init() {
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> UnitStruct {
            try deserializer.increase_container_depth()
            try deserializer.decrease_container_depth()
            return UnitStruct()
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> UnitStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
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

    let actual = emit!(UnitStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    public struct UnitStruct: Hashable {
        public init() {
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> UnitStruct {
            try deserializer.increase_container_depth()
            try deserializer.decrease_container_depth()
            return UnitStruct()
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> UnitStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
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

    let actual = emit!(NewType as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    public struct NewType: Hashable {
        public var value: String

        public init(value: String) {
            self.value = value
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: self.value)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> NewType {
            try deserializer.increase_container_depth()
            let value = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return NewType(value: value)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> NewType {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
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

    let actual = emit!(TupleStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    public struct TupleStruct: Hashable {
        public var field0: String
        public var field1: Int32

        public init(field0: String, field1: Int32) {
            self.field0 = field0
            self.field1 = field1
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: self.field0)
            try serializer.serialize_i32(value: self.field1)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> TupleStruct {
            try deserializer.increase_container_depth()
            let field0 = try deserializer.deserialize_str()
            let field1 = try deserializer.deserialize_i32()
            try deserializer.decrease_container_depth()
            return TupleStruct(field0: field0, field1: field1)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> TupleStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
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

    let actual = emit!(StructWithFields as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line 1
    /// line 2
    public struct StructWithFields: Hashable {
        /// unit type
        public var unit: ()
        /// boolean
        public var bool: Bool
        public var i8: Int8
        public var i16: Int16
        public var i32: Int32
        public var i64: Int64
        public var i128: Int128
        public var u8: UInt8
        public var u16: UInt16
        public var u32: UInt32
        public var u64: UInt64
        public var u128: UInt128
        public var f32: Float
        public var f64: Double
        public var char: Character
        public var string: String

        public init(unit: (), bool: Bool, i8: Int8, i16: Int16, i32: Int32, i64: Int64, i128: Int128, u8: UInt8, u16: UInt16, u32: UInt32, u64: UInt64, u128: UInt128, f32: Float, f64: Double, char: Character, string: String) {
            self.unit = unit
            self.bool = bool
            self.i8 = i8
            self.i16 = i16
            self.i32 = i32
            self.i64 = i64
            self.i128 = i128
            self.u8 = u8
            self.u16 = u16
            self.u32 = u32
            self.u64 = u64
            self.u128 = u128
            self.f32 = f32
            self.f64 = f64
            self.char = char
            self.string = string
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_unit(value: self.unit)
            try serializer.serialize_bool(value: self.bool)
            try serializer.serialize_i8(value: self.i8)
            try serializer.serialize_i16(value: self.i16)
            try serializer.serialize_i32(value: self.i32)
            try serializer.serialize_i64(value: self.i64)
            try serializer.serialize_i128(value: self.i128)
            try serializer.serialize_u8(value: self.u8)
            try serializer.serialize_u16(value: self.u16)
            try serializer.serialize_u32(value: self.u32)
            try serializer.serialize_u64(value: self.u64)
            try serializer.serialize_u128(value: self.u128)
            try serializer.serialize_f32(value: self.f32)
            try serializer.serialize_f64(value: self.f64)
            try serializer.serialize_char(value: self.char)
            try serializer.serialize_str(value: self.string)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> StructWithFields {
            try deserializer.increase_container_depth()
            let unit = try deserializer.deserialize_unit()
            let bool = try deserializer.deserialize_bool()
            let i8 = try deserializer.deserialize_i8()
            let i16 = try deserializer.deserialize_i16()
            let i32 = try deserializer.deserialize_i32()
            let i64 = try deserializer.deserialize_i64()
            let i128 = try deserializer.deserialize_i128()
            let u8 = try deserializer.deserialize_u8()
            let u16 = try deserializer.deserialize_u16()
            let u32 = try deserializer.deserialize_u32()
            let u64 = try deserializer.deserialize_u64()
            let u128 = try deserializer.deserialize_u128()
            let f32 = try deserializer.deserialize_f32()
            let f64 = try deserializer.deserialize_f64()
            let char = try deserializer.deserialize_char()
            let string = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return StructWithFields(unit: unit, bool: bool, i8: i8, i16: i16, i32: i32, i64: i64, i128: i128, u8: u8, u16: u16, u32: u32, u64: u64, u128: u128, f32: f32, f64: f64, char: char, string: string)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> StructWithFields {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
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

    let actual = emit!(Outer as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct Inner1: Hashable {
        public var field1: String

        public init(field1: String) {
            self.field1 = field1
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: self.field1)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> Inner1 {
            try deserializer.increase_container_depth()
            let field1 = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return Inner1(field1: field1)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> Inner1 {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }

    public struct Inner2: Hashable {
        public var value: String

        public init(value: String) {
            self.value = value
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: self.value)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> Inner2 {
            try deserializer.increase_container_depth()
            let value = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return Inner2(value: value)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> Inner2 {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }

    public struct Inner3: Hashable {
        public var field0: String
        public var field1: Int32

        public init(field0: String, field1: Int32) {
            self.field0 = field0
            self.field1 = field1
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: self.field0)
            try serializer.serialize_i32(value: self.field1)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> Inner3 {
            try deserializer.increase_container_depth()
            let field0 = try deserializer.deserialize_str()
            let field1 = try deserializer.deserialize_i32()
            try deserializer.decrease_container_depth()
            return Inner3(field0: field0, field1: field1)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> Inner3 {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }

    public struct Outer: Hashable {
        public var one: Inner1
        public var two: Inner2
        public var three: Inner3

        public init(one: Inner1, two: Inner2, three: Inner3) {
            self.one = one
            self.two = two
            self.three = three
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try self.one.serialize(serializer: serializer)
            try self.two.serialize(serializer: serializer)
            try self.three.serialize(serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> Outer {
            try deserializer.increase_container_depth()
            let one = try Inner1.deserialize(deserializer: deserializer)
            let two = try Inner2.deserialize(deserializer: deserializer)
            let three = try Inner3.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return Outer(one: one, two: two, three: three)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> Outer {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
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

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        public var one: (String, Int32)

        public init(one: (String, Int32)) {
            self.one = one
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: one.0)
            try serializer.serialize_i32(value: one.1)
            try serializer.decrease_container_depth()
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            try deserializer.increase_container_depth()
            let one0 = try deserializer.deserialize_str()
            let one1 = try deserializer.deserialize_i32()
            let one = (one0, one1)
            try deserializer.decrease_container_depth()
            try deserializer.decrease_container_depth()
            return MyStruct(one: one)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
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

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        public var one: (String, Int32, UInt16)

        public init(one: (String, Int32, UInt16)) {
            self.one = one
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: one.0)
            try serializer.serialize_i32(value: one.1)
            try serializer.serialize_u16(value: one.2)
            try serializer.decrease_container_depth()
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            try deserializer.increase_container_depth()
            let one0 = try deserializer.deserialize_str()
            let one1 = try deserializer.deserialize_i32()
            let one2 = try deserializer.deserialize_u16()
            let one = (one0, one1, one2)
            try deserializer.decrease_container_depth()
            try deserializer.decrease_container_depth()
            return MyStruct(one: one)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
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

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        public var one: (String, Int32, UInt16, Float)

        public init(one: (String, Int32, UInt16, Float)) {
            self.one = one
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: one.0)
            try serializer.serialize_i32(value: one.1)
            try serializer.serialize_u16(value: one.2)
            try serializer.serialize_f32(value: one.3)
            try serializer.decrease_container_depth()
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            try deserializer.increase_container_depth()
            let one0 = try deserializer.deserialize_str()
            let one1 = try deserializer.deserialize_i32()
            let one2 = try deserializer.deserialize_u16()
            let one3 = try deserializer.deserialize_f32()
            let one = (one0, one1, one2, one3)
            try deserializer.decrease_container_depth()
            try deserializer.decrease_container_depth()
            return MyStruct(one: one)
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
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

    let actual = emit!(EnumWithUnitVariants as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    /// line one
    /// line two
    public enum EnumWithUnitVariants: Hashable {
        /// variant one
        case variant1
        /// variant two
        case variant2
        /// variant three
        case variant3

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            switch self {
            case .variant1:
                try serializer.serialize_variant_index(value: 0)
            case .variant2:
                try serializer.serialize_variant_index(value: 1)
            case .variant3:
                try serializer.serialize_variant_index(value: 2)
            }
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> Array<UInt8> {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> EnumWithUnitVariants {
            let index = try deserializer.deserialize_variant_index()
            try deserializer.increase_container_depth()
            switch index {
            case 0:
                try deserializer.decrease_container_depth()
                return .variant1
            case 1:
                try deserializer.decrease_container_depth()
                return .variant2
            case 2:
                try deserializer.decrease_container_depth()
                return .variant3
            default: throw DeserializationError.invalidInput(issue: "Unknown variant index for EnumWithUnitVariants: \(index)")
            }
        }

        public static func jsonDeserialize(input: Array<UInt8>) throws -> EnumWithUnitVariants {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn enum_with_unit_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 {},
    }

    let actual = emit!(MyEnum as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    indirect public enum MyEnum: Hashable {
        case variant1

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            switch self {
            case .variant1:
                try serializer.serialize_variant_index(value: 0)
            }
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyEnum {
            let index = try deserializer.deserialize_variant_index()
            try deserializer.increase_container_depth()
            switch index {
            case 0:
                try deserializer.decrease_container_depth()
                return .variant1
            default: throw DeserializationError.invalidInput(issue: "Unknown variant index for MyEnum: \(index)")
            }
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn enum_with_1_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
    }

    let actual = emit!(MyEnum as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    indirect public enum MyEnum: Hashable {
        case variant1(String)

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            switch self {
            case .variant1(let x):
                try serializer.serialize_variant_index(value: 0)
                try serializer.serialize_str(value: x)
            }
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyEnum {
            let index = try deserializer.deserialize_variant_index()
            try deserializer.increase_container_depth()
            switch index {
            case 0:
                let x = try deserializer.deserialize_str()
                try deserializer.decrease_container_depth()
                return .variant1(x)
            default: throw DeserializationError.invalidInput(issue: "Unknown variant index for MyEnum: \(index)")
            }
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn enum_with_newtype_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
        Variant2(i32),
    }

    let actual = emit!(MyEnum as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    indirect public enum MyEnum: Hashable {
        case variant1(String)
        case variant2(Int32)

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            switch self {
            case .variant1(let x):
                try serializer.serialize_variant_index(value: 0)
                try serializer.serialize_str(value: x)
            case .variant2(let x):
                try serializer.serialize_variant_index(value: 1)
                try serializer.serialize_i32(value: x)
            }
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyEnum {
            let index = try deserializer.deserialize_variant_index()
            try deserializer.increase_container_depth()
            switch index {
            case 0:
                let x = try deserializer.deserialize_str()
                try deserializer.decrease_container_depth()
                return .variant1(x)
            case 1:
                let x = try deserializer.deserialize_i32()
                try deserializer.decrease_container_depth()
                return .variant2(x)
            default: throw DeserializationError.invalidInput(issue: "Unknown variant index for MyEnum: \(index)")
            }
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn enum_with_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String, i32),
        Variant2(bool, f64, u8),
    }

    let actual = emit!(MyEnum as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    indirect public enum MyEnum: Hashable {
        case variant1(String, Int32)
        case variant2(Bool, Double, UInt8)

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            switch self {
            case .variant1(let x0, let x1):
                try serializer.serialize_variant_index(value: 0)
                try serializer.serialize_str(value: x0)
                try serializer.serialize_i32(value: x1)
            case .variant2(let x0, let x1, let x2):
                try serializer.serialize_variant_index(value: 1)
                try serializer.serialize_bool(value: x0)
                try serializer.serialize_f64(value: x1)
                try serializer.serialize_u8(value: x2)
            }
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyEnum {
            let index = try deserializer.deserialize_variant_index()
            try deserializer.increase_container_depth()
            switch index {
            case 0:
                let x0 = try deserializer.deserialize_str()
                let x1 = try deserializer.deserialize_i32()
                try deserializer.decrease_container_depth()
                return .variant1(x0, x1)
            case 1:
                let x0 = try deserializer.deserialize_bool()
                let x1 = try deserializer.deserialize_f64()
                let x2 = try deserializer.deserialize_u8()
                try deserializer.decrease_container_depth()
                return .variant2(x0, x1, x2)
            default: throw DeserializationError.invalidInput(issue: "Unknown variant index for MyEnum: \(index)")
            }
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn enum_with_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 { field1: String, field2: i32 },
    }

    let actual = emit!(MyEnum as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    indirect public enum MyEnum: Hashable {
        case variant1(field1: String, field2: Int32)

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            switch self {
            case .variant1(let field1, let field2):
                try serializer.serialize_variant_index(value: 0)
                try serializer.serialize_str(value: field1)
                try serializer.serialize_i32(value: field2)
            }
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyEnum {
            let index = try deserializer.deserialize_variant_index()
            try deserializer.increase_container_depth()
            switch index {
            case 0:
                let field1 = try deserializer.deserialize_str()
                let field2 = try deserializer.deserialize_i32()
                try deserializer.decrease_container_depth()
                return .variant1(field1: field1, field2: field2)
            default: throw DeserializationError.invalidInput(issue: "Unknown variant index for MyEnum: \(index)")
            }
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
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

    let actual = emit!(MyEnum as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    indirect public enum MyEnum: Hashable {
        case unit
        case newType(String)
        case tuple(String, Int32)
        case struct(field: Bool)

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            switch self {
            case .unit:
                try serializer.serialize_variant_index(value: 0)
            case .newType(let x):
                try serializer.serialize_variant_index(value: 1)
                try serializer.serialize_str(value: x)
            case .tuple(let x0, let x1):
                try serializer.serialize_variant_index(value: 2)
                try serializer.serialize_str(value: x0)
                try serializer.serialize_i32(value: x1)
            case .struct(let field):
                try serializer.serialize_variant_index(value: 3)
                try serializer.serialize_bool(value: field)
            }
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyEnum {
            let index = try deserializer.deserialize_variant_index()
            try deserializer.increase_container_depth()
            switch index {
            case 0:
                try deserializer.decrease_container_depth()
                return .unit
            case 1:
                let x = try deserializer.deserialize_str()
                try deserializer.decrease_container_depth()
                return .newType(x)
            case 2:
                let x0 = try deserializer.deserialize_str()
                let x1 = try deserializer.deserialize_i32()
                try deserializer.decrease_container_depth()
                return .tuple(x0, x1)
            case 3:
                let field = try deserializer.deserialize_bool()
                try deserializer.decrease_container_depth()
                return .struct(field: field)
            default: throw DeserializationError.invalidInput(issue: "Unknown variant index for MyEnum: \(index)")
            }
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_vec_field() {
    #[derive(Facet)]
    struct MyStruct {
        items: Vec<String>,
        numbers: Vec<i32>,
        nested_items: Vec<Vec<String>>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var items: [String]
        @Indirect public var numbers: [Int32]
        @Indirect public var nestedItems: [[String]]

        public init(items: [String], numbers: [Int32], nestedItems: [[String]]) {
            self.items = items
            self.numbers = numbers
            self.nestedItems = nestedItems
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serialize_vector_str(value: self.items, serializer: serializer)
            try serialize_vector_i32(value: self.numbers, serializer: serializer)
            try serialize_vector_vector_str(value: self.nestedItems, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let items = try deserialize_vector_str(deserializer: deserializer)
            let numbers = try deserialize_vector_i32(deserializer: deserializer)
            let nestedItems = try deserialize_vector_vector_str(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(items: items, numbers: numbers, nestedItems: nestedItems)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_option_field() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct MyStruct {
        optional_string: Option<String>,
        optional_number: Option<i32>,
        optional_bool: Option<bool>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var optionalString: String?
        @Indirect public var optionalNumber: Int32?
        @Indirect public var optionalBool: Bool?

        public init(optionalString: String?, optionalNumber: Int32?, optionalBool: Bool?) {
            self.optionalString = optionalString
            self.optionalNumber = optionalNumber
            self.optionalBool = optionalBool
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serialize_option_str(value: self.optionalString, serializer: serializer)
            try serialize_option_i32(value: self.optionalNumber, serializer: serializer)
            try serialize_option_bool(value: self.optionalBool, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let optionalString = try deserialize_option_str(deserializer: deserializer)
            let optionalNumber = try deserialize_option_i32(deserializer: deserializer)
            let optionalBool = try deserialize_option_bool(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(optionalString: optionalString, optionalNumber: optionalNumber, optionalBool: optionalBool)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        int_to_bool: HashMap<i32, bool>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var stringToInt: [String: Int32]
        @Indirect public var intToBool: [Int32: Bool]

        public init(stringToInt: [String: Int32], intToBool: [Int32: Bool]) {
            self.stringToInt = stringToInt
            self.intToBool = intToBool
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serialize_map_str_to_i32(value: self.stringToInt, serializer: serializer)
            try serialize_map_i32_to_bool(value: self.intToBool, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let stringToInt = try deserialize_map_str_to_i32(deserializer: deserializer)
            let intToBool = try deserialize_map_i32_to_bool(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(stringToInt: stringToInt, intToBool: intToBool)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_nested_generics() {
    #[derive(Facet)]
    struct MyStruct {
        optional_list: Option<Vec<String>>,
        list_of_optionals: Vec<Option<i32>>,
        map_to_list: HashMap<String, Vec<bool>>,
        optional_map: Option<HashMap<String, i32>>,
        complex: Vec<Option<HashMap<String, Vec<bool>>>>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var optionalList: [String]?
        @Indirect public var listOfOptionals: [Int32?]
        @Indirect public var mapToList: [String: [Bool]]
        @Indirect public var optionalMap: [String: Int32]?
        @Indirect public var complex: [[String: [Bool]]?]

        public init(optionalList: [String]?, listOfOptionals: [Int32?], mapToList: [String: [Bool]], optionalMap: [String: Int32]?, complex: [[String: [Bool]]?]) {
            self.optionalList = optionalList
            self.listOfOptionals = listOfOptionals
            self.mapToList = mapToList
            self.optionalMap = optionalMap
            self.complex = complex
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serialize_option_vector_str(value: self.optionalList, serializer: serializer)
            try serialize_vector_option_i32(value: self.listOfOptionals, serializer: serializer)
            try serialize_map_str_to_vector_bool(value: self.mapToList, serializer: serializer)
            try serialize_option_map_str_to_i32(value: self.optionalMap, serializer: serializer)
            try serialize_vector_option_map_str_to_vector_bool(value: self.complex, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let optionalList = try deserialize_option_vector_str(deserializer: deserializer)
            let listOfOptionals = try deserialize_vector_option_i32(deserializer: deserializer)
            let mapToList = try deserialize_map_str_to_vector_bool(deserializer: deserializer)
            let optionalMap = try deserialize_option_map_str_to_i32(deserializer: deserializer)
            let complex = try deserialize_vector_option_map_str_to_vector_bool(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(optionalList: optionalList, listOfOptionals: listOfOptionals, mapToList: mapToList, optionalMap: optionalMap, complex: complex)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_array_field() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct MyStruct {
        fixed_array: [i32; 5],
        byte_array: [u8; 32],
        string_array: [String; 3],
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var fixedArray: [Int32]
        @Indirect public var byteArray: [UInt8]
        @Indirect public var stringArray: [String]

        public init(fixedArray: [Int32], byteArray: [UInt8], stringArray: [String]) {
            self.fixedArray = fixedArray
            self.byteArray = byteArray
            self.stringArray = stringArray
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serialize_array5_i32_array(value: self.fixedArray, serializer: serializer)
            try serialize_array32_u8_array(value: self.byteArray, serializer: serializer)
            try serialize_array3_str_array(value: self.stringArray, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let fixedArray = try deserialize_array5_i32_array(deserializer: deserializer)
            let byteArray = try deserialize_array32_u8_array(deserializer: deserializer)
            let stringArray = try deserialize_array3_str_array(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(fixedArray: fixedArray, byteArray: byteArray, stringArray: stringArray)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var stringToInt: [String: Int32]
        @Indirect public var intToBool: [Int32: Bool]

        public init(stringToInt: [String: Int32], intToBool: [Int32: Bool]) {
            self.stringToInt = stringToInt
            self.intToBool = intToBool
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serialize_map_str_to_i32(value: self.stringToInt, serializer: serializer)
            try serialize_map_i32_to_bool(value: self.intToBool, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let stringToInt = try deserialize_map_str_to_i32(deserializer: deserializer)
            let intToBool = try deserialize_map_i32_to_bool(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(stringToInt: stringToInt, intToBool: intToBool)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_hashset_field() {
    // NOTE: HashSet<T> now maps to Set<T> in Kotlin with the new Format::Set variant.
    // This preserves the uniqueness constraint and provides better type safety.
    #[derive(Facet)]
    struct MyStruct {
        string_set: HashSet<String>,
        int_set: HashSet<i32>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var stringSet: [String]
        @Indirect public var intSet: [Int32]

        public init(stringSet: [String], intSet: [Int32]) {
            self.stringSet = stringSet
            self.intSet = intSet
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serialize_set_str(value: self.stringSet, serializer: serializer)
            try serialize_set_i32(value: self.intSet, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let stringSet = try deserialize_set_str(deserializer: deserializer)
            let intSet = try deserialize_set_i32(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(stringSet: stringSet, intSet: intSet)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_btreeset_field() {
    // NOTE: BTreeSet<T> now maps to Set<T> in Kotlin with the new Format::Set variant.
    // This preserves the uniqueness constraint and provides better type safety.
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var stringSet: [String]
        @Indirect public var intSet: [Int32]

        public init(stringSet: [String], intSet: [Int32]) {
            self.stringSet = stringSet
            self.intSet = intSet
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serialize_set_str(value: self.stringSet, serializer: serializer)
            try serialize_set_i32(value: self.intSet, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let stringSet = try deserialize_set_str(deserializer: deserializer)
            let intSet = try deserialize_set_i32(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(stringSet: stringSet, intSet: intSet)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_box_field() {
    #[derive(Facet)]
    #[allow(clippy::box_collection)]
    struct MyStruct {
        boxed_string: Box<String>,
        boxed_int: Box<i32>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var boxedString: String
        @Indirect public var boxedInt: Int32

        public init(boxedString: String, boxedInt: Int32) {
            self.boxedString = boxedString
            self.boxedInt = boxedInt
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: self.boxedString)
            try serializer.serialize_i32(value: self.boxedInt)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let boxedString = try deserializer.deserialize_str()
            let boxedInt = try deserializer.deserialize_i32()
            try deserializer.decrease_container_depth()
            return MyStruct.init(boxedString: boxedString, boxedInt: boxedInt)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: Rc<String>,
        rc_int: Rc<i32>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var rcString: String
        @Indirect public var rcInt: Int32

        public init(rcString: String, rcInt: Int32) {
            self.rcString = rcString
            self.rcInt = rcInt
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: self.rcString)
            try serializer.serialize_i32(value: self.rcInt)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let rcString = try deserializer.deserialize_str()
            let rcInt = try deserializer.deserialize_i32()
            try deserializer.decrease_container_depth()
            return MyStruct.init(rcString: rcString, rcInt: rcInt)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: Arc<String>,
        arc_int: Arc<i32>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var arcString: String
        @Indirect public var arcInt: Int32

        public init(arcString: String, arcInt: Int32) {
            self.arcString = arcString
            self.arcInt = arcInt
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: self.arcString)
            try serializer.serialize_i32(value: self.arcInt)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let arcString = try deserializer.deserialize_str()
            let arcInt = try deserializer.deserialize_i32()
            try deserializer.decrease_container_depth()
            return MyStruct.init(arcString: arcString, arcInt: arcInt)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
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

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var vecOfSets: [[String]]
        @Indirect public var optionalBtree: [String: Int32]?
        @Indirect public var boxedVec: [String]
        @Indirect public var arcOption: String?
        @Indirect public var arrayOfBoxes: [Int32]

        public init(vecOfSets: [[String]], optionalBtree: [String: Int32]?, boxedVec: [String], arcOption: String?, arrayOfBoxes: [Int32]) {
            self.vecOfSets = vecOfSets
            self.optionalBtree = optionalBtree
            self.boxedVec = boxedVec
            self.arcOption = arcOption
            self.arrayOfBoxes = arrayOfBoxes
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serialize_vector_set_str(value: self.vecOfSets, serializer: serializer)
            try serialize_option_map_str_to_i32(value: self.optionalBtree, serializer: serializer)
            try serialize_vector_str(value: self.boxedVec, serializer: serializer)
            try serialize_option_str(value: self.arcOption, serializer: serializer)
            try serialize_array3_i32_array(value: self.arrayOfBoxes, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let vecOfSets = try deserialize_vector_set_str(deserializer: deserializer)
            let optionalBtree = try deserialize_option_map_str_to_i32(deserializer: deserializer)
            let boxedVec = try deserialize_vector_str(deserializer: deserializer)
            let arcOption = try deserialize_option_str(deserializer: deserializer)
            let arrayOfBoxes = try deserialize_array3_i32_array(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(vecOfSets: vecOfSets, optionalBtree: optionalBtree, boxedVec: boxedVec, arcOption: arcOption, arrayOfBoxes: arrayOfBoxes)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn struct_with_bytes_field() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(bytes)]
        data: Vec<u8>,
        name: String,
        #[facet(bytes)]
        header: Vec<u8>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var data: [UInt8]
        @Indirect public var name: String
        @Indirect public var header: [UInt8]

        public init(data: [UInt8], name: String, header: [UInt8]) {
            self.data = data
            self.name = name
            self.header = header
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_bytes(value: self.data)
            try serializer.serialize_str(value: self.name)
            try serializer.serialize_bytes(value: self.header)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let data = try deserializer.deserialize_bytes()
            let name = try deserializer.deserialize_str()
            let header = try deserializer.deserialize_bytes()
            try deserializer.decrease_container_depth()
            return MyStruct.init(data: data, name: name, header: header)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
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

    let actual = emit!(MyStruct as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct MyStruct: Hashable {
        @Indirect public var data: [UInt8]
        @Indirect public var name: String
        @Indirect public var header: [UInt8]
        @Indirect public var optionalBytes: [UInt8]?

        public init(data: [UInt8], name: String, header: [UInt8], optionalBytes: [UInt8]?) {
            self.data = data
            self.name = name
            self.header = header
            self.optionalBytes = optionalBytes
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_bytes(value: self.data)
            try serializer.serialize_str(value: self.name)
            try serializer.serialize_bytes(value: self.header)
            try serialize_option_vector_u8(value: self.optionalBytes, serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
            try deserializer.increase_container_depth()
            let data = try deserializer.deserialize_bytes()
            let name = try deserializer.deserialize_str()
            let header = try deserializer.deserialize_bytes()
            let optionalBytes = try deserialize_option_vector_u8(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return MyStruct.init(data: data, name: name, header: header, optionalBytes: optionalBytes)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}

#[test]
#[ignore = "unimplemented"]
fn namespaced_child() {
    #[derive(Facet)]
    #[facet(namespace = "Test")]
    struct Child {
        test: String,
    }

    #[derive(Facet)]
    struct Parent {
        child: Child,
    }

    let actual = emit!(Parent as Swift with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"
    public struct Parent: Hashable {
        @Indirect public var child: Child

        public init(child: Child) {
            self.child = child
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try self.child.serialize(serializer: serializer)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> Parent {
            try deserializer.increase_container_depth()
            let child = try Test.Child.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return Parent.init(child: child)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Parent {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }

    public struct Child: Hashable {
        @Indirect public var test: String

        public init(test: String) {
            self.test = test
        }

        public func serialize<S: Serializer>(serializer: S) throws {
            try serializer.increase_container_depth()
            try serializer.serialize_str(value: self.test)
            try serializer.decrease_container_depth()
        }

        public func jsonSerialize() throws -> [UInt8] {
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }

        public static func deserialize<D: Deserializer>(deserializer: D) throws -> Child {
            try deserializer.increase_container_depth()
            let test = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return Child.init(test: test)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Child {
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }
            return obj
        }
    }
    "#);
}
