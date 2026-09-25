//! Snapshot tests for the Swift emitter — **JSON encoding**.
//!
//! Mirrors the structure of [`tests`](super::tests) but uses `JsonPlugin`, so
//! every generated type conforms to `Codable`, with explicit `CodingKeys` and,
//! where Swift's synthesized coding would not match `serde_json`, hand-written
//! `init(from:)` / `encode(to:)`, plus `jsonSerialize` / `jsonDeserialize`
//! convenience wrappers.

#![allow(clippy::too_many_lines)]

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use facet::Facet;

use super::*;
use crate::{self as fg, emit, generation::json::JsonPlugin};

#[test]
fn unit_struct_1() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct UnitStruct: Hashable, Equatable, Codable {
        public init() {
        }

        public init(from decoder: Decoder) throws {
            if try decoder.singleValueContainer().decodeNil() {
                return
            }
            _ = try decoder.container(keyedBy: Serde.JsonKey.self)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.singleValueContainer()
            try container.encodeNil()
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> UnitStruct {
            return try Serde.jsonDeserialize(UnitStruct.self, from: input)
        }
    }
    ");
}

#[test]
fn unit_struct_2() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct {}

    let actual = emit!(UnitStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct UnitStruct: Hashable, Equatable, Codable {
        public init() {
        }

        public init(from decoder: Decoder) throws {
            if try decoder.singleValueContainer().decodeNil() {
                return
            }
            _ = try decoder.container(keyedBy: Serde.JsonKey.self)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.singleValueContainer()
            try container.encodeNil()
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> UnitStruct {
            return try Serde.jsonDeserialize(UnitStruct.self, from: input)
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

    let actual = emit!(NewType as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct NewType: Hashable, Equatable, Codable {
        public var value: String

        public init(value: String) {
            self.value = value
        }

        public init(from decoder: Decoder) throws {
            self.value = try decoder.singleValueContainer().decode(String.self)
        }

        public func encode(to encoder: Encoder) throws {
            var single0 = encoder.singleValueContainer()
            try single0.encode(self.value)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> NewType {
            return try Serde.jsonDeserialize(NewType.self, from: input)
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

    let actual = emit!(TupleStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct TupleStruct: Hashable, Equatable, Codable {
        public var field0: String
        public var field1: Int32

        public init(field0: String, field1: Int32) {
            self.field0 = field0
            self.field1 = field1
        }

        public init(from decoder: Decoder) throws {
            var container = try decoder.unkeyedContainer()
            self.field0 = try container.decode(String.self)
            self.field1 = try container.decode(Int32.self)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.unkeyedContainer()
            try container.encode(self.field0)
            try container.encode(self.field1)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> TupleStruct {
            return try Serde.jsonDeserialize(TupleStruct.self, from: input)
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

    let actual = emit!(StructWithFields as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct StructWithFields: Codable {
        /// unit type
        public var unit: Void
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

        public init(unit: Void, bool: Bool, i8: Int8, i16: Int16, i32: Int32, i64: Int64, i128: Int128, u8: UInt8, u16: UInt16, u32: UInt32, u64: UInt64, u128: UInt128, f32: Float, f64: Double, char: Character, string: String) {
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

        enum CodingKeys: String, CodingKey {
            case unit
            case bool
            case i8
            case i16
            case i32
            case i64
            case i128
            case u8
            case u16
            case u32
            case u64
            case u128
            case f32
            case f64
            case char
            case string
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.unit = try container.decode(Serde.JsonUnit.self, forKey: .unit).value
            self.bool = try container.decode(Bool.self, forKey: .bool)
            self.i8 = try container.decode(Int8.self, forKey: .i8)
            self.i16 = try container.decode(Int16.self, forKey: .i16)
            self.i32 = try container.decode(Int32.self, forKey: .i32)
            self.i64 = try container.decode(Int64.self, forKey: .i64)
            self.i128 = try container.decode(Int128.self, forKey: .i128)
            self.u8 = try container.decode(UInt8.self, forKey: .u8)
            self.u16 = try container.decode(UInt16.self, forKey: .u16)
            self.u32 = try container.decode(UInt32.self, forKey: .u32)
            self.u64 = try container.decode(UInt64.self, forKey: .u64)
            self.u128 = try container.decode(UInt128.self, forKey: .u128)
            self.f32 = try container.decode(Float.self, forKey: .f32)
            self.f64 = try container.decode(Double.self, forKey: .f64)
            self.char = try container.decode(Serde.JsonChar.self, forKey: .char).value
            self.string = try container.decode(String.self, forKey: .string)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(Serde.JsonUnit(), forKey: .unit)
            try container.encode(self.bool, forKey: .bool)
            try container.encode(self.i8, forKey: .i8)
            try container.encode(self.i16, forKey: .i16)
            try container.encode(self.i32, forKey: .i32)
            try container.encode(self.i64, forKey: .i64)
            try container.encode(self.i128, forKey: .i128)
            try container.encode(self.u8, forKey: .u8)
            try container.encode(self.u16, forKey: .u16)
            try container.encode(self.u32, forKey: .u32)
            try container.encode(self.u64, forKey: .u64)
            try container.encode(self.u128, forKey: .u128)
            try container.encode(self.f32, forKey: .f32)
            try container.encode(self.f64, forKey: .f64)
            try container.encode(Serde.JsonChar(self.char), forKey: .char)
            try container.encode(self.string, forKey: .string)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> StructWithFields {
            return try Serde.jsonDeserialize(StructWithFields.self, from: input)
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

    let actual = emit!(Outer as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct Inner1: Hashable, Equatable, Codable {
        public var field1: String

        public init(field1: String) {
            self.field1 = field1
        }

        enum CodingKeys: String, CodingKey {
            case field1
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Inner1 {
            return try Serde.jsonDeserialize(Inner1.self, from: input)
        }
    }

    public struct Inner2: Hashable, Equatable, Codable {
        public var value: String

        public init(value: String) {
            self.value = value
        }

        public init(from decoder: Decoder) throws {
            self.value = try decoder.singleValueContainer().decode(String.self)
        }

        public func encode(to encoder: Encoder) throws {
            var single0 = encoder.singleValueContainer()
            try single0.encode(self.value)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Inner2 {
            return try Serde.jsonDeserialize(Inner2.self, from: input)
        }
    }

    public struct Inner3: Hashable, Equatable, Codable {
        public var field0: String
        public var field1: Int32

        public init(field0: String, field1: Int32) {
            self.field0 = field0
            self.field1 = field1
        }

        public init(from decoder: Decoder) throws {
            var container = try decoder.unkeyedContainer()
            self.field0 = try container.decode(String.self)
            self.field1 = try container.decode(Int32.self)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.unkeyedContainer()
            try container.encode(self.field0)
            try container.encode(self.field1)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Inner3 {
            return try Serde.jsonDeserialize(Inner3.self, from: input)
        }
    }

    public struct Outer: Hashable, Equatable, Codable {
        public var one: Inner1
        public var two: Inner2
        public var three: Inner3

        public init(one: Inner1, two: Inner2, three: Inner3) {
            self.one = one
            self.two = two
            self.three = three
        }

        enum CodingKeys: String, CodingKey {
            case one
            case two
            case three
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Outer {
            return try Serde.jsonDeserialize(Outer.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Equatable, Codable {
        public var one: (String, Int32)

        public init(one: (String, Int32)) {
            self.one = one
        }

        enum CodingKeys: String, CodingKey {
            case one
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.one = try { () throws -> (String, Int32) in
                var nested0 = try container.nestedUnkeyedContainer(forKey: .one)
                return (
                    try nested0.decode(String.self),
                    try nested0.decode(Int32.self)
                )
            }()
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            do {
                var nested0 = container.nestedUnkeyedContainer(forKey: .one)
                try nested0.encode(self.one.0)
                try nested0.encode(self.one.1)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
        }

        public static func == (lhs: MyStruct, rhs: MyStruct) -> Bool {
            return lhs.one == rhs.one
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Equatable, Codable {
        public var one: (String, Int32, UInt16)

        public init(one: (String, Int32, UInt16)) {
            self.one = one
        }

        enum CodingKeys: String, CodingKey {
            case one
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.one = try { () throws -> (String, Int32, UInt16) in
                var nested0 = try container.nestedUnkeyedContainer(forKey: .one)
                return (
                    try nested0.decode(String.self),
                    try nested0.decode(Int32.self),
                    try nested0.decode(UInt16.self)
                )
            }()
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            do {
                var nested0 = container.nestedUnkeyedContainer(forKey: .one)
                try nested0.encode(self.one.0)
                try nested0.encode(self.one.1)
                try nested0.encode(self.one.2)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
        }

        public static func == (lhs: MyStruct, rhs: MyStruct) -> Bool {
            return lhs.one == rhs.one
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

    // TODO: The NTuple4 struct should be emitted in the preamble if required, e.g.
    // data class NTuple4<T1, T2, T3, T4>(val t1: T1, val t2: T2, val t3: T3, val t4: T4)

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Equatable, Codable {
        public var one: (String, Int32, UInt16, Float)

        public init(one: (String, Int32, UInt16, Float)) {
            self.one = one
        }

        enum CodingKeys: String, CodingKey {
            case one
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.one = try { () throws -> (String, Int32, UInt16, Float) in
                var nested0 = try container.nestedUnkeyedContainer(forKey: .one)
                return (
                    try nested0.decode(String.self),
                    try nested0.decode(Int32.self),
                    try nested0.decode(UInt16.self),
                    try nested0.decode(Float.self)
                )
            }()
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            do {
                var nested0 = container.nestedUnkeyedContainer(forKey: .one)
                try nested0.encode(self.one.0)
                try nested0.encode(self.one.1)
                try nested0.encode(self.one.2)
                try nested0.encode(self.one.3)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
        }

        public static func == (lhs: MyStruct, rhs: MyStruct) -> Bool {
            return lhs.one == rhs.one
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

    let actual = emit!(EnumWithUnitVariants as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    /// line one
    /// line two
    indirect public enum EnumWithUnitVariants: Hashable, Equatable, Codable {
        /// variant one
        case variant1
        /// variant two
        case variant2
        /// variant three
        case variant3

        enum CodingKeys: String, CodingKey {
            case variant1 = "Variant1"
            case variant2 = "Variant2"
            case variant3 = "Variant3"
        }

        public init(from decoder: Decoder) throws {
            if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
                switch name {
                case "Variant1":
                    self = .variant1
                case "Variant2":
                    self = .variant2
                case "Variant3":
                    self = .variant3
                default:
                    throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for EnumWithUnitVariants")
                }
                return
            }
            let container = try decoder.container(keyedBy: CodingKeys.self)
            guard container.allKeys.count == 1, let key = container.allKeys.first else {
                throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of EnumWithUnitVariants"))
            }
            switch key {
            case .variant1:
                self = .variant1
            case .variant2:
                self = .variant2
            case .variant3:
                self = .variant3
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .variant1:
                var container = encoder.singleValueContainer()
                try container.encode("Variant1")
            case .variant2:
                var container = encoder.singleValueContainer()
                try container.encode("Variant2")
            case .variant3:
                var container = encoder.singleValueContainer()
                try container.encode("Variant3")
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> EnumWithUnitVariants {
            return try Serde.jsonDeserialize(EnumWithUnitVariants.self, from: input)
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

    let actual = emit!(MyEnum as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    indirect public enum MyEnum: Hashable, Equatable, Codable {
        case variant1

        enum CodingKeys: String, CodingKey {
            case variant1 = "Variant1"
        }

        public init(from decoder: Decoder) throws {
            if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
                switch name {
                case "Variant1":
                    self = .variant1
                default:
                    throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for MyEnum")
                }
                return
            }
            let container = try decoder.container(keyedBy: CodingKeys.self)
            guard container.allKeys.count == 1, let key = container.allKeys.first else {
                throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of MyEnum"))
            }
            switch key {
            case .variant1:
                self = .variant1
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .variant1:
                var container = encoder.singleValueContainer()
                try container.encode("Variant1")
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            return try Serde.jsonDeserialize(MyEnum.self, from: input)
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

    let actual = emit!(MyEnum as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    indirect public enum MyEnum: Hashable, Equatable, Codable {
        case variant1(String)

        enum CodingKeys: String, CodingKey {
            case variant1 = "Variant1"
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            guard container.allKeys.count == 1, let key = container.allKeys.first else {
                throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of MyEnum"))
            }
            switch key {
            case .variant1:
                self = .variant1(
                    try container.decode(String.self, forKey: .variant1)
                )
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .variant1(let payload0):
                var container = encoder.container(keyedBy: CodingKeys.self)
                try container.encode(payload0, forKey: .variant1)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            return try Serde.jsonDeserialize(MyEnum.self, from: input)
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

    let actual = emit!(MyEnum as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    indirect public enum MyEnum: Hashable, Equatable, Codable {
        case variant1(String)
        case variant2(Int32)

        enum CodingKeys: String, CodingKey {
            case variant1 = "Variant1"
            case variant2 = "Variant2"
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            guard container.allKeys.count == 1, let key = container.allKeys.first else {
                throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of MyEnum"))
            }
            switch key {
            case .variant1:
                self = .variant1(
                    try container.decode(String.self, forKey: .variant1)
                )
            case .variant2:
                self = .variant2(
                    try container.decode(Int32.self, forKey: .variant2)
                )
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .variant1(let payload0):
                var container = encoder.container(keyedBy: CodingKeys.self)
                try container.encode(payload0, forKey: .variant1)
            case .variant2(let payload0):
                var container = encoder.container(keyedBy: CodingKeys.self)
                try container.encode(payload0, forKey: .variant2)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            return try Serde.jsonDeserialize(MyEnum.self, from: input)
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

    let actual = emit!(MyEnum as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    indirect public enum MyEnum: Hashable, Equatable, Codable {
        case variant1(String, Int32)
        case variant2(Bool, Double, UInt8)

        enum CodingKeys: String, CodingKey {
            case variant1 = "Variant1"
            case variant2 = "Variant2"
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            guard container.allKeys.count == 1, let key = container.allKeys.first else {
                throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of MyEnum"))
            }
            switch key {
            case .variant1:
                var nested = try container.nestedUnkeyedContainer(forKey: .variant1)
                self = .variant1(
                    try nested.decode(String.self),
                    try nested.decode(Int32.self)
                )
            case .variant2:
                var nested = try container.nestedUnkeyedContainer(forKey: .variant2)
                self = .variant2(
                    try nested.decode(Bool.self),
                    try nested.decode(Double.self),
                    try nested.decode(UInt8.self)
                )
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .variant1(let payload0, let payload1):
                var container = encoder.container(keyedBy: CodingKeys.self)
                var nested = container.nestedUnkeyedContainer(forKey: .variant1)
                try nested.encode(payload0)
                try nested.encode(payload1)
            case .variant2(let payload0, let payload1, let payload2):
                var container = encoder.container(keyedBy: CodingKeys.self)
                var nested = container.nestedUnkeyedContainer(forKey: .variant2)
                try nested.encode(payload0)
                try nested.encode(payload1)
                try nested.encode(payload2)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            return try Serde.jsonDeserialize(MyEnum.self, from: input)
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

    let actual = emit!(MyEnum as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    indirect public enum MyEnum: Hashable, Equatable, Codable {
        case variant1(field1: String, field2: Int32)

        enum CodingKeys: String, CodingKey {
            case variant1 = "Variant1"
        }

        enum Variant1CodingKeys: String, CodingKey {
            case field1
            case field2
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            guard container.allKeys.count == 1, let key = container.allKeys.first else {
                throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of MyEnum"))
            }
            switch key {
            case .variant1:
                let nested = try container.nestedContainer(keyedBy: Variant1CodingKeys.self, forKey: .variant1)
                self = .variant1(
                    field1: try nested.decode(String.self, forKey: .field1),
                    field2: try nested.decode(Int32.self, forKey: .field2)
                )
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .variant1(let payload0, let payload1):
                var container = encoder.container(keyedBy: CodingKeys.self)
                var nested = container.nestedContainer(keyedBy: Variant1CodingKeys.self, forKey: .variant1)
                try nested.encode(payload0, forKey: .field1)
                try nested.encode(payload1, forKey: .field2)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            return try Serde.jsonDeserialize(MyEnum.self, from: input)
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

    let actual = emit!(MyEnum as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    indirect public enum MyEnum: Hashable, Equatable, Codable {
        case unit
        case newType(String)
        case tuple(String, Int32)
        case `struct`(field: Bool)

        enum CodingKeys: String, CodingKey {
            case unit = "Unit"
            case newType = "NewType"
            case tuple = "Tuple"
            case `struct` = "Struct"
        }

        enum StructCodingKeys: String, CodingKey {
            case field
        }

        public init(from decoder: Decoder) throws {
            if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
                switch name {
                case "Unit":
                    self = .unit
                default:
                    throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for MyEnum")
                }
                return
            }
            let container = try decoder.container(keyedBy: CodingKeys.self)
            guard container.allKeys.count == 1, let key = container.allKeys.first else {
                throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of MyEnum"))
            }
            switch key {
            case .unit:
                self = .unit
            case .newType:
                self = .newType(
                    try container.decode(String.self, forKey: .newType)
                )
            case .tuple:
                var nested = try container.nestedUnkeyedContainer(forKey: .tuple)
                self = .tuple(
                    try nested.decode(String.self),
                    try nested.decode(Int32.self)
                )
            case .`struct`:
                let nested = try container.nestedContainer(keyedBy: StructCodingKeys.self, forKey: .`struct`)
                self = .`struct`(
                    field: try nested.decode(Bool.self, forKey: .field)
                )
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .unit:
                var container = encoder.singleValueContainer()
                try container.encode("Unit")
            case .newType(let payload0):
                var container = encoder.container(keyedBy: CodingKeys.self)
                try container.encode(payload0, forKey: .newType)
            case .tuple(let payload0, let payload1):
                var container = encoder.container(keyedBy: CodingKeys.self)
                var nested = container.nestedUnkeyedContainer(forKey: .tuple)
                try nested.encode(payload0)
                try nested.encode(payload1)
            case .`struct`(let payload0):
                var container = encoder.container(keyedBy: CodingKeys.self)
                var nested = container.nestedContainer(keyedBy: StructCodingKeys.self, forKey: .`struct`)
                try nested.encode(payload0, forKey: .field)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            return try Serde.jsonDeserialize(MyEnum.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var items: [String]
        public var numbers: [Int32]
        public var nestedItems: [[String]]

        public init(items: [String], numbers: [Int32], nestedItems: [[String]]) {
            self.items = items
            self.numbers = numbers
            self.nestedItems = nestedItems
        }

        enum CodingKeys: String, CodingKey {
            case items
            case numbers
            case nestedItems = "nested_items"
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var optionalString: String?
        public var optionalNumber: Int32?
        public var optionalBool: Bool?

        public init(optionalString: String?, optionalNumber: Int32?, optionalBool: Bool?) {
            self.optionalString = optionalString
            self.optionalNumber = optionalNumber
            self.optionalBool = optionalBool
        }

        enum CodingKeys: String, CodingKey {
            case optionalString = "optional_string"
            case optionalNumber = "optional_number"
            case optionalBool = "optional_bool"
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.optionalString = try container.decodeIfPresent(String.self, forKey: .optionalString)
            self.optionalNumber = try container.decodeIfPresent(Int32.self, forKey: .optionalNumber)
            self.optionalBool = try container.decodeIfPresent(Bool.self, forKey: .optionalBool)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(self.optionalString, forKey: .optionalString)
            try container.encode(self.optionalNumber, forKey: .optionalNumber)
            try container.encode(self.optionalBool, forKey: .optionalBool)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var stringToInt: [String: Int32]
        public var intToBool: [Int32: Bool]

        public init(stringToInt: [String: Int32], intToBool: [Int32: Bool]) {
            self.stringToInt = stringToInt
            self.intToBool = intToBool
        }

        enum CodingKeys: String, CodingKey {
            case stringToInt = "string_to_int"
            case intToBool = "int_to_bool"
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.stringToInt = try container.decode([String: Int32].self, forKey: .stringToInt)
            self.intToBool = try { () throws -> [Int32: Bool] in
                let nested0 = try container.nestedContainer(keyedBy: Serde.JsonKey.self, forKey: .intToBool)
                var result0: [Int32: Bool] = [:]
                for key0 in nested0.allKeys {
                    let mapKey0 = try Serde.jsonMapKey(key0.stringValue, as: Int32.self)
                    result0.updateValue(try nested0.decode(Bool.self, forKey: key0), forKey: mapKey0)
                }
                return result0
            }()
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(self.stringToInt, forKey: .stringToInt)
            do {
                var nested0 = container.nestedContainer(keyedBy: Serde.JsonKey.self, forKey: .intToBool)
                for (key0, value0) in self.intToBool {
                    let objectKey0 = Serde.JsonKey(try Serde.jsonMapKey(key0))
                    try nested0.encode(value0, forKey: objectKey0)
                }
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var optionalList: [String]?
        public var listOfOptionals: [Int32?]
        public var mapToList: [String: [Bool]]
        public var optionalMap: [String: Int32]?
        public var complex: [[String: [Bool]]?]

        public init(optionalList: [String]?, listOfOptionals: [Int32?], mapToList: [String: [Bool]], optionalMap: [String: Int32]?, complex: [[String: [Bool]]?]) {
            self.optionalList = optionalList
            self.listOfOptionals = listOfOptionals
            self.mapToList = mapToList
            self.optionalMap = optionalMap
            self.complex = complex
        }

        enum CodingKeys: String, CodingKey {
            case optionalList = "optional_list"
            case listOfOptionals = "list_of_optionals"
            case mapToList = "map_to_list"
            case optionalMap = "optional_map"
            case complex
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.optionalList = try container.decodeIfPresent([String].self, forKey: .optionalList)
            self.listOfOptionals = try container.decode([Int32?].self, forKey: .listOfOptionals)
            self.mapToList = try container.decode([String: [Bool]].self, forKey: .mapToList)
            self.optionalMap = try container.decodeIfPresent([String: Int32].self, forKey: .optionalMap)
            self.complex = try container.decode([[String: [Bool]]?].self, forKey: .complex)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(self.optionalList, forKey: .optionalList)
            try container.encode(self.listOfOptionals, forKey: .listOfOptionals)
            try container.encode(self.mapToList, forKey: .mapToList)
            try container.encode(self.optionalMap, forKey: .optionalMap)
            try container.encode(self.complex, forKey: .complex)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var fixedArray: [Int32]
        public var byteArray: [UInt8]
        public var stringArray: [String]

        public init(fixedArray: [Int32], byteArray: [UInt8], stringArray: [String]) {
            self.fixedArray = fixedArray
            self.byteArray = byteArray
            self.stringArray = stringArray
        }

        enum CodingKeys: String, CodingKey {
            case fixedArray = "fixed_array"
            case byteArray = "byte_array"
            case stringArray = "string_array"
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var stringToInt: [String: Int32]
        public var intToBool: [Int32: Bool]

        public init(stringToInt: [String: Int32], intToBool: [Int32: Bool]) {
            self.stringToInt = stringToInt
            self.intToBool = intToBool
        }

        enum CodingKeys: String, CodingKey {
            case stringToInt = "string_to_int"
            case intToBool = "int_to_bool"
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.stringToInt = try container.decode([String: Int32].self, forKey: .stringToInt)
            self.intToBool = try { () throws -> [Int32: Bool] in
                let nested0 = try container.nestedContainer(keyedBy: Serde.JsonKey.self, forKey: .intToBool)
                var result0: [Int32: Bool] = [:]
                for key0 in nested0.allKeys {
                    let mapKey0 = try Serde.jsonMapKey(key0.stringValue, as: Int32.self)
                    result0.updateValue(try nested0.decode(Bool.self, forKey: key0), forKey: mapKey0)
                }
                return result0
            }()
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(self.stringToInt, forKey: .stringToInt)
            do {
                var nested0 = container.nestedContainer(keyedBy: Serde.JsonKey.self, forKey: .intToBool)
                for (key0, value0) in self.intToBool {
                    let objectKey0 = Serde.JsonKey(try Serde.jsonMapKey(key0))
                    try nested0.encode(value0, forKey: objectKey0)
                }
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var stringSet: Set<String>
        public var intSet: Set<Int32>

        public init(stringSet: Set<String>, intSet: Set<Int32>) {
            self.stringSet = stringSet
            self.intSet = intSet
        }

        enum CodingKeys: String, CodingKey {
            case stringSet = "string_set"
            case intSet = "int_set"
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var stringSet: Set<String>
        public var intSet: Set<Int32>

        public init(stringSet: Set<String>, intSet: Set<Int32>) {
            self.stringSet = stringSet
            self.intSet = intSet
        }

        enum CodingKeys: String, CodingKey {
            case stringSet = "string_set"
            case intSet = "int_set"
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var boxedString: String
        public var boxedInt: Int32

        public init(boxedString: String, boxedInt: Int32) {
            self.boxedString = boxedString
            self.boxedInt = boxedInt
        }

        enum CodingKeys: String, CodingKey {
            case boxedString = "boxed_string"
            case boxedInt = "boxed_int"
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var rcString: String
        public var rcInt: Int32

        public init(rcString: String, rcInt: Int32) {
            self.rcString = rcString
            self.rcInt = rcInt
        }

        enum CodingKeys: String, CodingKey {
            case rcString = "rc_string"
            case rcInt = "rc_int"
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var arcString: String
        public var arcInt: Int32

        public init(arcString: String, arcInt: Int32) {
            self.arcString = arcString
            self.arcInt = arcInt
        }

        enum CodingKeys: String, CodingKey {
            case arcString = "arc_string"
            case arcInt = "arc_int"
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var vecOfSets: [Set<String>]
        public var optionalBtree: [String: Int32]?
        public var boxedVec: [String]
        public var arcOption: String?
        public var arrayOfBoxes: [Int32]

        public init(vecOfSets: [Set<String>], optionalBtree: [String: Int32]?, boxedVec: [String], arcOption: String?, arrayOfBoxes: [Int32]) {
            self.vecOfSets = vecOfSets
            self.optionalBtree = optionalBtree
            self.boxedVec = boxedVec
            self.arcOption = arcOption
            self.arrayOfBoxes = arrayOfBoxes
        }

        enum CodingKeys: String, CodingKey {
            case vecOfSets = "vec_of_sets"
            case optionalBtree = "optional_btree"
            case boxedVec = "boxed_vec"
            case arcOption = "arc_option"
            case arrayOfBoxes = "array_of_boxes"
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.vecOfSets = try container.decode([Set<String>].self, forKey: .vecOfSets)
            self.optionalBtree = try container.decodeIfPresent([String: Int32].self, forKey: .optionalBtree)
            self.boxedVec = try container.decode([String].self, forKey: .boxedVec)
            self.arcOption = try container.decodeIfPresent(String.self, forKey: .arcOption)
            self.arrayOfBoxes = try container.decode([Int32].self, forKey: .arrayOfBoxes)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(self.vecOfSets, forKey: .vecOfSets)
            try container.encode(self.optionalBtree, forKey: .optionalBtree)
            try container.encode(self.boxedVec, forKey: .boxedVec)
            try container.encode(self.arcOption, forKey: .arcOption)
            try container.encode(self.arrayOfBoxes, forKey: .arrayOfBoxes)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
        }
    }
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var data: [UInt8]
        public var name: String
        public var header: [UInt8]

        public init(data: [UInt8], name: String, header: [UInt8]) {
            self.data = data
            self.name = name
            self.header = header
        }

        enum CodingKeys: String, CodingKey {
            case data
            case name
            case header
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
        }
    }
    ");
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

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Hashable, Equatable, Codable {
        public var data: [UInt8]
        public var name: String
        public var header: [UInt8]
        public var optionalBytes: [UInt8]?

        public init(data: [UInt8], name: String, header: [UInt8], optionalBytes: [UInt8]?) {
            self.data = data
            self.name = name
            self.header = header
            self.optionalBytes = optionalBytes
        }

        enum CodingKeys: String, CodingKey {
            case data
            case name
            case header
            case optionalBytes = "optional_bytes"
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.data = try container.decode([UInt8].self, forKey: .data)
            self.name = try container.decode(String.self, forKey: .name)
            self.header = try container.decode([UInt8].self, forKey: .header)
            self.optionalBytes = try container.decodeIfPresent([UInt8].self, forKey: .optionalBytes)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(self.data, forKey: .data)
            try container.encode(self.name, forKey: .name)
            try container.encode(self.header, forKey: .header)
            try container.encode(self.optionalBytes, forKey: .optionalBytes)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
        }
    }
    "#);
}

#[test]
fn namespaced_child() {
    #[derive(Facet)]
    #[facet(fg::namespace = "Test")]
    struct Child {
        test: String,
    }

    #[derive(Facet)]
    struct Parent {
        child: Child,
    }

    let actual = emit!(Parent as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct Parent: Hashable, Equatable, Codable {
        public var child: Test.Child

        public init(child: Test.Child) {
            self.child = child
        }

        enum CodingKeys: String, CodingKey {
            case child
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Parent {
            return try Serde.jsonDeserialize(Parent.self, from: input)
        }
    }

    public struct Child: Hashable, Equatable, Codable {
        public var test: String

        public init(test: String) {
            self.test = test
        }

        enum CodingKeys: String, CodingKey {
            case test
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Child {
            return try Serde.jsonDeserialize(Child.self, from: input)
        }
    }
    ");
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

    let actual = emit!(KeywordFields as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct KeywordFields: Hashable, Equatable, Codable {
        public var `default`: String
        public var `in`: Int32
        public var object: Bool
        public var `import`: Bool

        public init(`default`: String, `in`: Int32, object: Bool, `import`: Bool) {
            self.`default` = `default`
            self.`in` = `in`
            self.object = object
            self.`import` = `import`
        }

        enum CodingKeys: String, CodingKey {
            case `default`
            case `in`
            case object
            case `import`
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> KeywordFields {
            return try Serde.jsonDeserialize(KeywordFields.self, from: input)
        }
    }
    ");
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

    let actual = emit!(KeywordEnum as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    indirect public enum KeywordEnum: Hashable, Equatable, Codable {
        case `default`
        case `switch`(String)
        case `where`(`in`: Int32, `default`: String)

        enum CodingKeys: String, CodingKey {
            case `default` = "Default"
            case `switch` = "Switch"
            case `where` = "Where"
        }

        enum WhereCodingKeys: String, CodingKey {
            case `in`
            case `default`
        }

        public init(from decoder: Decoder) throws {
            if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
                switch name {
                case "Default":
                    self = .`default`
                default:
                    throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for KeywordEnum")
                }
                return
            }
            let container = try decoder.container(keyedBy: CodingKeys.self)
            guard container.allKeys.count == 1, let key = container.allKeys.first else {
                throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of KeywordEnum"))
            }
            switch key {
            case .`default`:
                self = .`default`
            case .`switch`:
                self = .`switch`(
                    try container.decode(String.self, forKey: .`switch`)
                )
            case .`where`:
                let nested = try container.nestedContainer(keyedBy: WhereCodingKeys.self, forKey: .`where`)
                self = .`where`(
                    in: try nested.decode(Int32.self, forKey: .`in`),
                    default: try nested.decode(String.self, forKey: .`default`)
                )
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .`default`:
                var container = encoder.singleValueContainer()
                try container.encode("Default")
            case .`switch`(let payload0):
                var container = encoder.container(keyedBy: CodingKeys.self)
                try container.encode(payload0, forKey: .`switch`)
            case .`where`(let payload0, let payload1):
                var container = encoder.container(keyedBy: CodingKeys.self)
                var nested = container.nestedContainer(keyedBy: WhereCodingKeys.self, forKey: .`where`)
                try nested.encode(payload0, forKey: .`in`)
                try nested.encode(payload1, forKey: .`default`)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> KeywordEnum {
            return try Serde.jsonDeserialize(KeywordEnum.self, from: input)
        }
    }
    "#);
}

#[test]
fn internally_tagged_enum() {
    #[derive(Facet)]
    struct Point {
        x: i32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    #[facet(tag = "type")]
    enum MyEnum {
        Unit,
        Wrapped(Point),
        Scalar(u8),
        Struct { field: bool },
    }

    let actual = emit!(MyEnum as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    indirect public enum MyEnum: Hashable, Equatable, Codable {
        case unit
        case wrapped(Point)
        case scalar(UInt8)
        case `struct`(field: Bool)

        enum StructCodingKeys: String, CodingKey {
            case field
        }

        public init(from decoder: Decoder) throws {
            let tagContainer = try decoder.container(keyedBy: Serde.JsonKey.self)
            let tag = try tagContainer.decode(String.self, forKey: Serde.JsonKey("type"))
            switch tag {
            case "Unit":
                self = .unit
            case "Wrapped":
                self = .wrapped(
                    try decoder.singleValueContainer().decode(Point.self)
                )
            case "Scalar":
                throw DecodingError.dataCorruptedError(forKey: Serde.JsonKey("type"), in: tagContainer, debugDescription: "MyEnum.Scalar cannot be internally tagged: its payload is not written as an object")
            case "Struct":
                let nested = try decoder.container(keyedBy: StructCodingKeys.self)
                self = .`struct`(
                    field: try nested.decode(Bool.self, forKey: .field)
                )
            default:
                throw DecodingError.dataCorruptedError(forKey: Serde.JsonKey("type"), in: tagContainer, debugDescription: "Unknown variant \(tag) for MyEnum")
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .unit:
                var tagContainer = encoder.container(keyedBy: Serde.JsonKey.self)
                try tagContainer.encode("Unit", forKey: Serde.JsonKey("type"))
            case .wrapped(let payload0):
                var tagContainer = encoder.container(keyedBy: Serde.JsonKey.self)
                try tagContainer.encode("Wrapped", forKey: Serde.JsonKey("type"))
                try payload0.encode(to: encoder)
            case .scalar:
                throw EncodingError.invalidValue(self, .init(codingPath: encoder.codingPath, debugDescription: "MyEnum.Scalar cannot be internally tagged: its payload is not written as an object"))
            case .`struct`(let payload0):
                var tagContainer = encoder.container(keyedBy: Serde.JsonKey.self)
                try tagContainer.encode("Struct", forKey: Serde.JsonKey("type"))
                var nested = encoder.container(keyedBy: StructCodingKeys.self)
                try nested.encode(payload0, forKey: .field)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            return try Serde.jsonDeserialize(MyEnum.self, from: input)
        }
    }

    public struct Point: Hashable, Equatable, Codable {
        public var x: Int32

        public init(x: Int32) {
            self.x = x
        }

        enum CodingKeys: String, CodingKey {
            case x
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Point {
            return try Serde.jsonDeserialize(Point.self, from: input)
        }
    }
    "#);
}

#[test]
fn adjacently_tagged_enum() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    #[facet(tag = "t", content = "c")]
    enum MyEnum {
        Unit,
        NewType(Option<char>),
        Tuple(u8, String),
        Struct { field: bool },
    }

    let actual = emit!(MyEnum as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    indirect public enum MyEnum: Hashable, Equatable, Codable {
        case unit
        case newType(Character?)
        case tuple(UInt8, String)
        case `struct`(field: Bool)

        enum StructCodingKeys: String, CodingKey {
            case field
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: Serde.JsonKey.self)
            let tag = try container.decode(String.self, forKey: Serde.JsonKey("t"))
            switch tag {
            case "Unit":
                self = .unit
            case "NewType":
                self = .newType(
                    try { () throws -> Character? in
                        guard container.contains(Serde.JsonKey("c")), try !container.decodeNil(forKey: Serde.JsonKey("c")) else { return nil }
                        return try container.decode(Serde.JsonChar.self, forKey: Serde.JsonKey("c")).value
                    }()
                )
            case "Tuple":
                var nested = try container.nestedUnkeyedContainer(forKey: Serde.JsonKey("c"))
                self = .tuple(
                    try nested.decode(UInt8.self),
                    try nested.decode(String.self)
                )
            case "Struct":
                let nested = try container.nestedContainer(keyedBy: StructCodingKeys.self, forKey: Serde.JsonKey("c"))
                self = .`struct`(
                    field: try nested.decode(Bool.self, forKey: .field)
                )
            default:
                throw DecodingError.dataCorruptedError(forKey: Serde.JsonKey("t"), in: container, debugDescription: "Unknown variant \(tag) for MyEnum")
            }
        }

        public func encode(to encoder: Encoder) throws {
            switch self {
            case .unit:
                var container = encoder.container(keyedBy: Serde.JsonKey.self)
                try container.encode("Unit", forKey: Serde.JsonKey("t"))
            case .newType(let payload0):
                var container = encoder.container(keyedBy: Serde.JsonKey.self)
                try container.encode("NewType", forKey: Serde.JsonKey("t"))
                if let value0 = payload0 {
                    try container.encode(Serde.JsonChar(value0), forKey: Serde.JsonKey("c"))
                } else {
                    try container.encodeNil(forKey: Serde.JsonKey("c"))
                }
            case .tuple(let payload0, let payload1):
                var container = encoder.container(keyedBy: Serde.JsonKey.self)
                try container.encode("Tuple", forKey: Serde.JsonKey("t"))
                var nested = container.nestedUnkeyedContainer(forKey: Serde.JsonKey("c"))
                try nested.encode(payload0)
                try nested.encode(payload1)
            case .`struct`(let payload0):
                var container = encoder.container(keyedBy: Serde.JsonKey.self)
                try container.encode("Struct", forKey: Serde.JsonKey("t"))
                var nested = container.nestedContainer(keyedBy: StructCodingKeys.self, forKey: Serde.JsonKey("c"))
                try nested.encode(payload0, forKey: .field)
            }
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyEnum {
            return try Serde.jsonDeserialize(MyEnum.self, from: input)
        }
    }
    "#);
}

#[test]
fn struct_with_values_swift_codes_differently() {
    #[derive(Facet)]
    #[allow(clippy::option_option)]
    struct MyStruct {
        unit: (),
        letter: char,
        pair: (u8, Option<char>),
        by_int: BTreeMap<u32, Vec<()>>,
        big: u128,
        maybe: Option<Option<u8>>,
    }

    let actual = emit!(MyStruct as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public struct MyStruct: Codable {
        public var unit: Void
        public var letter: Character
        public var pair: (UInt8, Character?)
        public var byInt: [UInt32: [Void]]
        public var big: UInt128
        public var maybe: UInt8??

        public init(unit: Void, letter: Character, pair: (UInt8, Character?), byInt: [UInt32: [Void]], big: UInt128, maybe: UInt8??) {
            self.unit = unit
            self.letter = letter
            self.pair = pair
            self.byInt = byInt
            self.big = big
            self.maybe = maybe
        }

        enum CodingKeys: String, CodingKey {
            case unit
            case letter
            case pair
            case byInt = "by_int"
            case big
            case maybe
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.unit = try container.decode(Serde.JsonUnit.self, forKey: .unit).value
            self.letter = try container.decode(Serde.JsonChar.self, forKey: .letter).value
            self.pair = try { () throws -> (UInt8, Character?) in
                var nested0 = try container.nestedUnkeyedContainer(forKey: .pair)
                return (
                    try nested0.decode(UInt8.self),
                    try { () throws -> Character? in
                        guard try !nested0.decodeNil() else { return nil }
                        return try nested0.decode(Serde.JsonChar.self).value
                    }()
                )
            }()
            self.byInt = try { () throws -> [UInt32: [Void]] in
                let nested0 = try container.nestedContainer(keyedBy: Serde.JsonKey.self, forKey: .byInt)
                var result0: [UInt32: [Void]] = [:]
                for key0 in nested0.allKeys {
                    let mapKey0 = try Serde.jsonMapKey(key0.stringValue, as: UInt32.self)
                    result0.updateValue(try { () throws -> [Void] in
                        var nested1 = try nested0.nestedUnkeyedContainer(forKey: key0)
                        var result1: [Void] = []
                        while !nested1.isAtEnd {
                            result1.append(try nested1.decode(Serde.JsonUnit.self).value)
                        }
                        return result1
                    }(), forKey: mapKey0)
                }
                return result0
            }()
            self.big = try container.decode(UInt128.self, forKey: .big)
            self.maybe = try container.decodeIfPresent(UInt8?.self, forKey: .maybe)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(Serde.JsonUnit(), forKey: .unit)
            try container.encode(Serde.JsonChar(self.letter), forKey: .letter)
            do {
                var nested0 = container.nestedUnkeyedContainer(forKey: .pair)
                try nested0.encode(self.pair.0)
                if let value1 = self.pair.1 {
                    try nested0.encode(Serde.JsonChar(value1))
                } else {
                    try nested0.encodeNil()
                }
            }
            do {
                var nested0 = container.nestedContainer(keyedBy: Serde.JsonKey.self, forKey: .byInt)
                for (key0, value0) in self.byInt {
                    let objectKey0 = Serde.JsonKey(try Serde.jsonMapKey(key0))
                    do {
                        var nested1 = nested0.nestedUnkeyedContainer(forKey: objectKey0)
                        for _ in value0 {
                            try nested1.encode(Serde.JsonUnit())
                        }
                    }
                }
            }
            try container.encode(self.big, forKey: .big)
            try container.encode(self.maybe, forKey: .maybe)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
        }
    }
    "#);
}

#[test]
fn recursive_struct() {
    #[derive(Facet)]
    struct Node {
        value: u32,
        next: Option<Box<Node>>,
    }

    let actual = emit!(Node as Swift with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct Node: Hashable, Equatable, Codable {
        public var value: UInt32
        @Indirect public var next: Node?

        public init(value: UInt32, next: Node?) {
            self.value = value
            self.next = next
        }

        enum CodingKeys: String, CodingKey {
            case value
            case next
        }

        public init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            self.value = try container.decode(UInt32.self, forKey: .value)
            self.next = try container.decodeIfPresent(Node.self, forKey: .next)
        }

        public func encode(to encoder: Encoder) throws {
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(self.value, forKey: .value)
            try container.encode(self.next, forKey: .next)
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> Node {
            return try Serde.jsonDeserialize(Node.self, from: input)
        }
    }
    ");
}
