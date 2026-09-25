//! Snapshot tests for the TypeScript emitter — **JSON encoding**.
//!
//! Mirrors the structure of [`tests`](super::tests) but uses [`JsonPlugin`],
//! so that every class has static `toJson` / `fromJson` and `jsonSerialize` /
//! `jsonDeserialize` methods, and every enum the same functions beside it,
//! writing and reading the JSON `serde_json` does.

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
    #[derive(Facet)]
    struct UnitStruct;

    let actual = emit!(UnitStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class UnitStruct {
        constructor () {
        }

        static toJson(value: UnitStruct): $json.JsonValue {
            return null;
        }

        static fromJson(json: unknown): UnitStruct {
            $json.readUnitStruct(json, "UnitStruct");
            return new UnitStruct();
        }

        static jsonSerialize(value: UnitStruct): string {
            return $json.stringify(UnitStruct.toJson(value));
        }

        static jsonDeserialize(text: string): UnitStruct {
            return UnitStruct.fromJson($json.parse(text));
        }
    }
    "#);
}

#[test]
fn unit_struct_2() {
    #[derive(Facet)]
    struct UnitStruct {}

    let actual = emit!(UnitStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class UnitStruct {
        constructor () {
        }

        static toJson(value: UnitStruct): $json.JsonValue {
            return null;
        }

        static fromJson(json: unknown): UnitStruct {
            $json.readUnitStruct(json, "UnitStruct");
            return new UnitStruct();
        }

        static jsonSerialize(value: UnitStruct): string {
            return $json.stringify(UnitStruct.toJson(value));
        }

        static jsonDeserialize(text: string): UnitStruct {
            return UnitStruct.fromJson($json.parse(text));
        }
    }
    "#);
}

#[test]
fn newtype_struct() {
    #[derive(Facet)]
    struct NewType(String);

    let actual = emit!(NewType as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"


    export class NewType {
        constructor (public value: str) {
        }

        static toJson(value: NewType): $json.JsonValue {
            return value.value;
        }

        static fromJson(json: unknown): NewType {
            return new NewType($json.readStr(json));
        }

        static jsonSerialize(value: NewType): string {
            return $json.stringify(NewType.toJson(value));
        }

        static jsonDeserialize(text: string): NewType {
            return NewType.fromJson($json.parse(text));
        }
    }
    ");
}

#[test]
fn tuple_struct() {
    #[derive(Facet)]
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @"


    export class TupleStruct {
        constructor (public field0: str, public field1: int32) {
        }

        static toJson(value: TupleStruct): $json.JsonValue {
            return [value.field0, value.field1];
        }

        static fromJson(json: unknown): TupleStruct {
            return new TupleStruct(...$json.readTuple<[str, int32]>(json, [$json.readStr, $json.readI32]));
        }

        static jsonSerialize(value: TupleStruct): string {
            return $json.stringify(TupleStruct.toJson(value));
        }

        static jsonDeserialize(text: string): TupleStruct {
            return TupleStruct.fromJson($json.parse(text));
        }
    }
    ");
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

    let actual = emit!(StructWithFields as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class StructWithFields {
        constructor (public unit: unit, public bool: bool, public i8: int8, public i16: int16, public i32: int32, public i64: int64, public i128: int128, public u8: uint8, public u16: uint16, public u32: uint32, public u64: uint64, public u128: uint128, public f32: float32, public f64: float64, public char: char, public string: str) {
        }

        static toJson(value: StructWithFields): $json.JsonValue {
            return {
                "unit": null,
                "bool": value.bool,
                "i8": value.i8,
                "i16": value.i16,
                "i32": value.i32,
                "i64": $json.writeBigInt(value.i64),
                "i128": $json.writeBigInt(value.i128),
                "u8": value.u8,
                "u16": value.u16,
                "u32": value.u32,
                "u64": $json.writeBigInt(value.u64),
                "u128": $json.writeBigInt(value.u128),
                "f32": $json.writeFloat(value.f32),
                "f64": $json.writeFloat(value.f64),
                "char": value.char,
                "string": value.string,
            };
        }

        static fromJson(json: unknown): StructWithFields {
            const obj = $json.readObject(json, "StructWithFields");
            return new StructWithFields(
                $json.readUnit($json.field(obj, "unit")),
                $json.readBool($json.field(obj, "bool")),
                $json.readI8($json.field(obj, "i8")),
                $json.readI16($json.field(obj, "i16")),
                $json.readI32($json.field(obj, "i32")),
                $json.readI64($json.field(obj, "i64")),
                $json.readI128($json.field(obj, "i128")),
                $json.readU8($json.field(obj, "u8")),
                $json.readU16($json.field(obj, "u16")),
                $json.readU32($json.field(obj, "u32")),
                $json.readU64($json.field(obj, "u64")),
                $json.readU128($json.field(obj, "u128")),
                $json.readF32($json.field(obj, "f32")),
                $json.readF64($json.field(obj, "f64")),
                $json.readChar($json.field(obj, "char")),
                $json.readStr($json.field(obj, "string")),
            );
        }

        static jsonSerialize(value: StructWithFields): string {
            return $json.stringify(StructWithFields.toJson(value));
        }

        static jsonDeserialize(text: string): StructWithFields {
            return StructWithFields.fromJson($json.parse(text));
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

    let actual = emit!(Outer as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class Inner1 {
        constructor (public field1: str) {
        }

        static toJson(value: Inner1): $json.JsonValue {
            return {
                "field1": value.field1,
            };
        }

        static fromJson(json: unknown): Inner1 {
            const obj = $json.readObject(json, "Inner1");
            return new Inner1(
                $json.readStr($json.field(obj, "field1")),
            );
        }

        static jsonSerialize(value: Inner1): string {
            return $json.stringify(Inner1.toJson(value));
        }

        static jsonDeserialize(text: string): Inner1 {
            return Inner1.fromJson($json.parse(text));
        }
    }


    export class Inner2 {
        constructor (public value: str) {
        }

        static toJson(value: Inner2): $json.JsonValue {
            return value.value;
        }

        static fromJson(json: unknown): Inner2 {
            return new Inner2($json.readStr(json));
        }

        static jsonSerialize(value: Inner2): string {
            return $json.stringify(Inner2.toJson(value));
        }

        static jsonDeserialize(text: string): Inner2 {
            return Inner2.fromJson($json.parse(text));
        }
    }


    export class Inner3 {
        constructor (public field0: str, public field1: int32) {
        }

        static toJson(value: Inner3): $json.JsonValue {
            return [value.field0, value.field1];
        }

        static fromJson(json: unknown): Inner3 {
            return new Inner3(...$json.readTuple<[str, int32]>(json, [$json.readStr, $json.readI32]));
        }

        static jsonSerialize(value: Inner3): string {
            return $json.stringify(Inner3.toJson(value));
        }

        static jsonDeserialize(text: string): Inner3 {
            return Inner3.fromJson($json.parse(text));
        }
    }


    export class Outer {
        constructor (public one: Inner1, public two: Inner2, public three: Inner3) {
        }

        static toJson(value: Outer): $json.JsonValue {
            return {
                "one": Inner1.toJson(value.one),
                "two": Inner2.toJson(value.two),
                "three": Inner3.toJson(value.three),
            };
        }

        static fromJson(json: unknown): Outer {
            const obj = $json.readObject(json, "Outer");
            return new Outer(
                Inner1.fromJson($json.field(obj, "one")),
                Inner2.fromJson($json.field(obj, "two")),
                Inner3.fromJson($json.field(obj, "three")),
            );
        }

        static jsonSerialize(value: Outer): string {
            return $json.stringify(Outer.toJson(value));
        }

        static jsonDeserialize(text: string): Outer {
            return Outer.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public one: Tuple<[str, int32]>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "one": [value.one[0], value.one[1]],
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readTuple<[str, int32]>($json.field(obj, "one"), [$json.readStr, $json.readI32]),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public one: Tuple<[str, int32, uint16]>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "one": [value.one[0], value.one[1], value.one[2]],
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readTuple<[str, int32, uint16]>($json.field(obj, "one"), [$json.readStr, $json.readI32, $json.readU16]),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public one: Tuple<[str, int32, uint16, float32]>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "one": [value.one[0], value.one[1], value.one[2], $json.writeFloat(value.one[3])],
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readTuple<[str, int32, uint16, float32]>($json.field(obj, "one"), [$json.readStr, $json.readI32, $json.readU16, $json.readF32]),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
        }
    }
    "#);
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

    let actual = emit!(EnumWithUnitVariants as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type EnumWithUnitVariants =
        | { kind: "Variant1" }
        | { kind: "Variant2" }
        | { kind: "Variant3" };

    export const enumWithUnitVariantsVariant1 = (): EnumWithUnitVariants => ({ kind: "Variant1" });

    export const enumWithUnitVariantsVariant2 = (): EnumWithUnitVariants => ({ kind: "Variant2" });

    export const enumWithUnitVariantsVariant3 = (): EnumWithUnitVariants => ({ kind: "Variant3" });

    export function matchEnumWithUnitVariants<R>(value: EnumWithUnitVariants, cases: {
        Variant1: (v: Extract<EnumWithUnitVariants, { kind: "Variant1" }>) => R;
        Variant2: (v: Extract<EnumWithUnitVariants, { kind: "Variant2" }>) => R;
        Variant3: (v: Extract<EnumWithUnitVariants, { kind: "Variant3" }>) => R;
    }): R {
        return cases[value.kind as EnumWithUnitVariants["kind"]](value as never);
    }

    export function toJsonEnumWithUnitVariants(value: EnumWithUnitVariants): $json.JsonValue {
        switch (value.kind) {
            case "Variant1": return "Variant1";
            case "Variant2": return "Variant2";
            case "Variant3": return "Variant3";
            default: throw $json.unknownVariant("EnumWithUnitVariants", value);
        }
    }

    export function fromJsonEnumWithUnitVariants(json: unknown): EnumWithUnitVariants {
        const [variant] = $json.readExternal(json, "EnumWithUnitVariants", ["Variant1", "Variant2", "Variant3"]);
        switch (variant) {
            case "Variant1": return { kind: "Variant1" };
            case "Variant2": return { kind: "Variant2" };
            case "Variant3": return { kind: "Variant3" };
            default: throw $json.unknownVariant("EnumWithUnitVariants", variant);
        }
    }

    export function jsonSerializeEnumWithUnitVariants(value: EnumWithUnitVariants): string {
        return $json.stringify(toJsonEnumWithUnitVariants(value));
    }

    export function jsonDeserializeEnumWithUnitVariants(text: string): EnumWithUnitVariants {
        return fromJsonEnumWithUnitVariants($json.parse(text));
    }
    "#);
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

    let actual = emit!(MyEnum as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type MyEnum =
        | { kind: "Variant1" };

    export const myEnumVariant1 = (): MyEnum => ({ kind: "Variant1" });

    export function matchMyEnum<R>(value: MyEnum, cases: {
        Variant1: (v: Extract<MyEnum, { kind: "Variant1" }>) => R;
    }): R {
        return cases[value.kind as MyEnum["kind"]](value as never);
    }

    export function toJsonMyEnum(value: MyEnum): $json.JsonValue {
        switch (value.kind) {
            case "Variant1": return "Variant1";
            default: throw $json.unknownVariant("MyEnum", value);
        }
    }

    export function fromJsonMyEnum(json: unknown): MyEnum {
        const [variant] = $json.readExternal(json, "MyEnum", ["Variant1"]);
        switch (variant) {
            case "Variant1": return { kind: "Variant1" };
            default: throw $json.unknownVariant("MyEnum", variant);
        }
    }

    export function jsonSerializeMyEnum(value: MyEnum): string {
        return $json.stringify(toJsonMyEnum(value));
    }

    export function jsonDeserializeMyEnum(text: string): MyEnum {
        return fromJsonMyEnum($json.parse(text));
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

    let actual = emit!(MyEnum as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type MyEnum =
        | { kind: "Variant1"; value: str };

    export const myEnumVariant1 = (value: str): MyEnum => ({ kind: "Variant1", value });

    export function matchMyEnum<R>(value: MyEnum, cases: {
        Variant1: (v: Extract<MyEnum, { kind: "Variant1" }>) => R;
    }): R {
        return cases[value.kind as MyEnum["kind"]](value as never);
    }

    export function toJsonMyEnum(value: MyEnum): $json.JsonValue {
        switch (value.kind) {
            case "Variant1": return { "Variant1": value.value };
            default: throw $json.unknownVariant("MyEnum", value);
        }
    }

    export function fromJsonMyEnum(json: unknown): MyEnum {
        const [variant, content] = $json.readExternal(json, "MyEnum", []);
        switch (variant) {
            case "Variant1": return { kind: "Variant1", value: $json.readStr(content) };
            default: throw $json.unknownVariant("MyEnum", variant);
        }
    }

    export function jsonSerializeMyEnum(value: MyEnum): string {
        return $json.stringify(toJsonMyEnum(value));
    }

    export function jsonDeserializeMyEnum(text: string): MyEnum {
        return fromJsonMyEnum($json.parse(text));
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

    let actual = emit!(MyEnum as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type MyEnum =
        | { kind: "Variant1"; value: str }
        | { kind: "Variant2"; value: int32 };

    export const myEnumVariant1 = (value: str): MyEnum => ({ kind: "Variant1", value });

    export const myEnumVariant2 = (value: int32): MyEnum => ({ kind: "Variant2", value });

    export function matchMyEnum<R>(value: MyEnum, cases: {
        Variant1: (v: Extract<MyEnum, { kind: "Variant1" }>) => R;
        Variant2: (v: Extract<MyEnum, { kind: "Variant2" }>) => R;
    }): R {
        return cases[value.kind as MyEnum["kind"]](value as never);
    }

    export function toJsonMyEnum(value: MyEnum): $json.JsonValue {
        switch (value.kind) {
            case "Variant1": return { "Variant1": value.value };
            case "Variant2": return { "Variant2": value.value };
            default: throw $json.unknownVariant("MyEnum", value);
        }
    }

    export function fromJsonMyEnum(json: unknown): MyEnum {
        const [variant, content] = $json.readExternal(json, "MyEnum", []);
        switch (variant) {
            case "Variant1": return { kind: "Variant1", value: $json.readStr(content) };
            case "Variant2": return { kind: "Variant2", value: $json.readI32(content) };
            default: throw $json.unknownVariant("MyEnum", variant);
        }
    }

    export function jsonSerializeMyEnum(value: MyEnum): string {
        return $json.stringify(toJsonMyEnum(value));
    }

    export function jsonDeserializeMyEnum(text: string): MyEnum {
        return fromJsonMyEnum($json.parse(text));
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

    let actual = emit!(MyEnum as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type MyEnum =
        | { kind: "Variant1"; field0: str; field1: int32 }
        | { kind: "Variant2"; field0: bool; field1: float64; field2: uint8 };

    export const myEnumVariant1 = (field0: str, field1: int32): MyEnum => ({ kind: "Variant1", field0, field1 });

    export const myEnumVariant2 = (field0: bool, field1: float64, field2: uint8): MyEnum => ({ kind: "Variant2", field0, field1, field2 });

    export function matchMyEnum<R>(value: MyEnum, cases: {
        Variant1: (v: Extract<MyEnum, { kind: "Variant1" }>) => R;
        Variant2: (v: Extract<MyEnum, { kind: "Variant2" }>) => R;
    }): R {
        return cases[value.kind as MyEnum["kind"]](value as never);
    }

    export function toJsonMyEnum(value: MyEnum): $json.JsonValue {
        switch (value.kind) {
            case "Variant1": return { "Variant1": [value.field0, value.field1] };
            case "Variant2": return { "Variant2": [value.field0, $json.writeFloat(value.field1), value.field2] };
            default: throw $json.unknownVariant("MyEnum", value);
        }
    }

    export function fromJsonMyEnum(json: unknown): MyEnum {
        const [variant, content] = $json.readExternal(json, "MyEnum", []);
        switch (variant) {
            case "Variant1": {
                const items = $json.readTuple<[str, int32]>(content, [$json.readStr, $json.readI32]);
                return { kind: "Variant1", field0: items[0], field1: items[1] };
            }
            case "Variant2": {
                const items = $json.readTuple<[bool, float64, uint8]>(content, [$json.readBool, $json.readF64, $json.readU8]);
                return { kind: "Variant2", field0: items[0], field1: items[1], field2: items[2] };
            }
            default: throw $json.unknownVariant("MyEnum", variant);
        }
    }

    export function jsonSerializeMyEnum(value: MyEnum): string {
        return $json.stringify(toJsonMyEnum(value));
    }

    export function jsonDeserializeMyEnum(text: string): MyEnum {
        return fromJsonMyEnum($json.parse(text));
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

    let actual = emit!(MyEnum as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type MyEnum =
        | { kind: "Variant1"; field1: str; field2: int32 };

    export const myEnumVariant1 = (field1: str, field2: int32): MyEnum => ({ kind: "Variant1", field1, field2 });

    export function matchMyEnum<R>(value: MyEnum, cases: {
        Variant1: (v: Extract<MyEnum, { kind: "Variant1" }>) => R;
    }): R {
        return cases[value.kind as MyEnum["kind"]](value as never);
    }

    export function toJsonMyEnum(value: MyEnum): $json.JsonValue {
        switch (value.kind) {
            case "Variant1": return {
                "Variant1": {
                    "field1": value.field1,
                    "field2": value.field2,
                },
            };
            default: throw $json.unknownVariant("MyEnum", value);
        }
    }

    export function fromJsonMyEnum(json: unknown): MyEnum {
        const [variant, content] = $json.readExternal(json, "MyEnum", []);
        switch (variant) {
            case "Variant1": {
                const obj = $json.readObject(content, "MyEnum::Variant1");
                return {
                    kind: "Variant1",
                    field1: $json.readStr($json.field(obj, "field1")),
                    field2: $json.readI32($json.field(obj, "field2")),
                };
            }
            default: throw $json.unknownVariant("MyEnum", variant);
        }
    }

    export function jsonSerializeMyEnum(value: MyEnum): string {
        return $json.stringify(toJsonMyEnum(value));
    }

    export function jsonDeserializeMyEnum(text: string): MyEnum {
        return fromJsonMyEnum($json.parse(text));
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

    let actual = emit!(MyEnum as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type MyEnum =
        | { kind: "Unit" }
        | { kind: "NewType"; value: str }
        | { kind: "Tuple"; field0: str; field1: int32 }
        | { kind: "Struct"; field: bool };

    export const myEnumUnit = (): MyEnum => ({ kind: "Unit" });

    export const myEnumNewType = (value: str): MyEnum => ({ kind: "NewType", value });

    export const myEnumTuple = (field0: str, field1: int32): MyEnum => ({ kind: "Tuple", field0, field1 });

    export const myEnumStruct = (field: bool): MyEnum => ({ kind: "Struct", field });

    export function matchMyEnum<R>(value: MyEnum, cases: {
        Unit: (v: Extract<MyEnum, { kind: "Unit" }>) => R;
        NewType: (v: Extract<MyEnum, { kind: "NewType" }>) => R;
        Tuple: (v: Extract<MyEnum, { kind: "Tuple" }>) => R;
        Struct: (v: Extract<MyEnum, { kind: "Struct" }>) => R;
    }): R {
        return cases[value.kind as MyEnum["kind"]](value as never);
    }

    export function toJsonMyEnum(value: MyEnum): $json.JsonValue {
        switch (value.kind) {
            case "Unit": return "Unit";
            case "NewType": return { "NewType": value.value };
            case "Tuple": return { "Tuple": [value.field0, value.field1] };
            case "Struct": return {
                "Struct": {
                    "field": value.field,
                },
            };
            default: throw $json.unknownVariant("MyEnum", value);
        }
    }

    export function fromJsonMyEnum(json: unknown): MyEnum {
        const [variant, content] = $json.readExternal(json, "MyEnum", ["Unit"]);
        switch (variant) {
            case "Unit": return { kind: "Unit" };
            case "NewType": return { kind: "NewType", value: $json.readStr(content) };
            case "Tuple": {
                const items = $json.readTuple<[str, int32]>(content, [$json.readStr, $json.readI32]);
                return { kind: "Tuple", field0: items[0], field1: items[1] };
            }
            case "Struct": {
                const obj = $json.readObject(content, "MyEnum::Struct");
                return {
                    kind: "Struct",
                    field: $json.readBool($json.field(obj, "field")),
                };
            }
            default: throw $json.unknownVariant("MyEnum", variant);
        }
    }

    export function jsonSerializeMyEnum(value: MyEnum): string {
        return $json.stringify(toJsonMyEnum(value));
    }

    export function jsonDeserializeMyEnum(text: string): MyEnum {
        return fromJsonMyEnum($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public items: Seq<str>, public numbers: Seq<int32>, public nested_items: Seq<Seq<str>>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "items": value.items,
                "numbers": value.numbers,
                "nested_items": value.nested_items,
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readSeq($json.field(obj, "items"), $json.readStr),
                $json.readSeq($json.field(obj, "numbers"), $json.readI32),
                $json.readSeq($json.field(obj, "nested_items"), (j0) => $json.readSeq(j0, $json.readStr)),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public optional_string: Optional<str>, public optional_number: Optional<int32>, public optional_bool: Optional<bool>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "optional_string": value.optional_string,
                "optional_number": value.optional_number,
                "optional_bool": value.optional_bool,
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readOption($json.field(obj, "optional_string"), $json.readStr),
                $json.readOption($json.field(obj, "optional_number"), $json.readI32),
                $json.readOption($json.field(obj, "optional_bool"), $json.readBool),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public string_to_int: Map<str,int32>, public int_to_bool: Map<int32,bool>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "string_to_int": $json.writeMap(value.string_to_int, (k0) => k0, (v0) => v0),
                "int_to_bool": $json.writeMap(value.int_to_bool, (k0) => `${k0}`, (v0) => v0),
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readMap($json.field(obj, "string_to_int"), (k0) => k0, $json.readI32),
                $json.readMap($json.field(obj, "int_to_bool"), (k0) => $json.readI32($json.keyLiteral(k0)), $json.readBool),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public optional_list: Optional<Seq<str>>, public list_of_optionals: Seq<Optional<int32>>, public map_to_list: Map<str,Seq<bool>>, public optional_map: Optional<Map<str,int32>>, public complex: Seq<Optional<Map<str,Seq<bool>>>>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "optional_list": value.optional_list,
                "list_of_optionals": value.list_of_optionals,
                "map_to_list": $json.writeMap(value.map_to_list, (k0) => k0, (v0) => v0),
                "optional_map": (value.optional_map === null ? null : $json.writeMap(value.optional_map, (k0) => k0, (v0) => v0)),
                "complex": value.complex.map((v0) => (v0 === null ? null : $json.writeMap(v0, (k1) => k1, (v1) => v1))),
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readOption($json.field(obj, "optional_list"), (j0) => $json.readSeq(j0, $json.readStr)),
                $json.readSeq($json.field(obj, "list_of_optionals"), (j0) => $json.readOption(j0, $json.readI32)),
                $json.readMap($json.field(obj, "map_to_list"), (k0) => k0, (j0) => $json.readSeq(j0, $json.readBool)),
                $json.readOption($json.field(obj, "optional_map"), (j0) => $json.readMap(j0, (k1) => k1, $json.readI32)),
                $json.readSeq($json.field(obj, "complex"), (j0) => $json.readOption(j0, (j1) => $json.readMap(j1, (k2) => k2, (j2) => $json.readSeq(j2, $json.readBool)))),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public fixed_array: ListTuple<[int32]>, public byte_array: ListTuple<[uint8]>, public string_array: ListTuple<[str]>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "fixed_array": value.fixed_array.map((v0) => v0[0]),
                "byte_array": value.byte_array.map((v0) => v0[0]),
                "string_array": value.string_array.map((v0) => v0[0]),
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readSeq($json.field(obj, "fixed_array"), (j0): [int32] => [$json.readI32(j0)], 5),
                $json.readSeq($json.field(obj, "byte_array"), (j0): [uint8] => [$json.readU8(j0)], 32),
                $json.readSeq($json.field(obj, "string_array"), (j0): [str] => [$json.readStr(j0)], 3),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public string_to_int: Map<str,int32>, public int_to_bool: Map<int32,bool>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "string_to_int": $json.writeMap(value.string_to_int, (k0) => k0, (v0) => v0),
                "int_to_bool": $json.writeMap(value.int_to_bool, (k0) => `${k0}`, (v0) => v0),
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readMap($json.field(obj, "string_to_int"), (k0) => k0, $json.readI32),
                $json.readMap($json.field(obj, "int_to_bool"), (k0) => $json.readI32($json.keyLiteral(k0)), $json.readBool),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
        }
    }
    "#);
}

#[test]
fn struct_with_hashset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: HashSet<String>,
        int_set: HashSet<i32>,
    }

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public string_set: Seq<str>, public int_set: Seq<int32>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "string_set": value.string_set,
                "int_set": value.int_set,
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readSeq($json.field(obj, "string_set"), $json.readStr),
                $json.readSeq($json.field(obj, "int_set"), $json.readI32),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
        }
    }
    "#);
}

#[test]
fn struct_with_btreeset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public string_set: Seq<str>, public int_set: Seq<int32>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "string_set": value.string_set,
                "int_set": value.int_set,
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readSeq($json.field(obj, "string_set"), $json.readStr),
                $json.readSeq($json.field(obj, "int_set"), $json.readI32),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public boxed_string: str, public boxed_int: int32) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "boxed_string": value.boxed_string,
                "boxed_int": value.boxed_int,
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readStr($json.field(obj, "boxed_string")),
                $json.readI32($json.field(obj, "boxed_int")),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public rc_string: str, public rc_int: int32) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "rc_string": value.rc_string,
                "rc_int": value.rc_int,
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readStr($json.field(obj, "rc_string")),
                $json.readI32($json.field(obj, "rc_int")),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public arc_string: str, public arc_int: int32) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "arc_string": value.arc_string,
                "arc_int": value.arc_int,
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readStr($json.field(obj, "arc_string")),
                $json.readI32($json.field(obj, "arc_int")),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public vec_of_sets: Seq<Seq<str>>, public optional_btree: Optional<Map<str,int32>>, public boxed_vec: Seq<str>, public arc_option: Optional<str>, public array_of_boxes: ListTuple<[int32]>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "vec_of_sets": value.vec_of_sets,
                "optional_btree": (value.optional_btree === null ? null : $json.writeMap(value.optional_btree, (k0) => k0, (v0) => v0)),
                "boxed_vec": value.boxed_vec,
                "arc_option": value.arc_option,
                "array_of_boxes": value.array_of_boxes.map((v0) => v0[0]),
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readSeq($json.field(obj, "vec_of_sets"), (j0) => $json.readSeq(j0, $json.readStr)),
                $json.readOption($json.field(obj, "optional_btree"), (j0) => $json.readMap(j0, (k1) => k1, $json.readI32)),
                $json.readSeq($json.field(obj, "boxed_vec"), $json.readStr),
                $json.readOption($json.field(obj, "arc_option"), $json.readStr),
                $json.readSeq($json.field(obj, "array_of_boxes"), (j0): [int32] => [$json.readI32(j0)], 3),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
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

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public data: bytes, public name: str, public header: bytes) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "data": $json.writeBytes(value.data),
                "name": value.name,
                "header": $json.writeBytes(value.header),
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readBytes($json.field(obj, "data")),
                $json.readStr($json.field(obj, "name")),
                $json.readBytes($json.field(obj, "header")),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
        }
    }
    "#);
}

#[test]
fn struct_with_bytes_field_and_slice() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(fg::bytes)]
        data: &'static [u8],
        name: String,
        #[facet(fg::bytes)]
        header: Vec<u8>,
        optional_bytes: Option<Vec<u8>>,
    }

    let actual = emit!(MyStruct as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class MyStruct {
        constructor (public data: bytes, public name: str, public header: bytes, public optional_bytes: Optional<Seq<uint8>>) {
        }

        static toJson(value: MyStruct): $json.JsonValue {
            return {
                "data": $json.writeBytes(value.data),
                "name": value.name,
                "header": $json.writeBytes(value.header),
                "optional_bytes": value.optional_bytes,
            };
        }

        static fromJson(json: unknown): MyStruct {
            const obj = $json.readObject(json, "MyStruct");
            return new MyStruct(
                $json.readBytes($json.field(obj, "data")),
                $json.readStr($json.field(obj, "name")),
                $json.readBytes($json.field(obj, "header")),
                $json.readOption($json.field(obj, "optional_bytes"), (j0) => $json.readSeq(j0, $json.readU8)),
            );
        }

        static jsonSerialize(value: MyStruct): string {
            return $json.stringify(MyStruct.toJson(value));
        }

        static jsonDeserialize(text: string): MyStruct {
            return MyStruct.fromJson($json.parse(text));
        }
    }
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

    let actual = emit!(KeywordFields as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class KeywordFields {
        public default: str;
        public in: int32;
        public object: bool;
        public import: bool;

        constructor (default_: str, in_: int32, object: bool, import_: bool) {
            this.default = default_;
            this.in = in_;
            this.object = object;
            this.import = import_;
        }

        static toJson(value: KeywordFields): $json.JsonValue {
            return {
                "default": value.default,
                "in": value.in,
                "object": value.object,
                "import": value.import,
            };
        }

        static fromJson(json: unknown): KeywordFields {
            const obj = $json.readObject(json, "KeywordFields");
            return new KeywordFields(
                $json.readStr($json.field(obj, "default")),
                $json.readI32($json.field(obj, "in")),
                $json.readBool($json.field(obj, "object")),
                $json.readBool($json.field(obj, "import")),
            );
        }

        static jsonSerialize(value: KeywordFields): string {
            return $json.stringify(KeywordFields.toJson(value));
        }

        static jsonDeserialize(text: string): KeywordFields {
            return KeywordFields.fromJson($json.parse(text));
        }
    }
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

    let actual = emit!(KeywordEnum as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type KeywordEnum =
        | { kind: "Default" }
        | { kind: "Switch"; value: str }
        | { kind: "Where"; in: int32; default: str };

    export const keywordEnumDefault = (): KeywordEnum => ({ kind: "Default" });

    export const keywordEnumSwitch = (value: str): KeywordEnum => ({ kind: "Switch", value });

    export const keywordEnumWhere = (in_: int32, default_: str): KeywordEnum => ({ kind: "Where", in: in_, default: default_ });

    export function matchKeywordEnum<R>(value: KeywordEnum, cases: {
        Default: (v: Extract<KeywordEnum, { kind: "Default" }>) => R;
        Switch: (v: Extract<KeywordEnum, { kind: "Switch" }>) => R;
        Where: (v: Extract<KeywordEnum, { kind: "Where" }>) => R;
    }): R {
        return cases[value.kind as KeywordEnum["kind"]](value as never);
    }

    export function toJsonKeywordEnum(value: KeywordEnum): $json.JsonValue {
        switch (value.kind) {
            case "Default": return "Default";
            case "Switch": return { "Switch": value.value };
            case "Where": return {
                "Where": {
                    "in": value.in,
                    "default": value.default,
                },
            };
            default: throw $json.unknownVariant("KeywordEnum", value);
        }
    }

    export function fromJsonKeywordEnum(json: unknown): KeywordEnum {
        const [variant, content] = $json.readExternal(json, "KeywordEnum", ["Default"]);
        switch (variant) {
            case "Default": return { kind: "Default" };
            case "Switch": return { kind: "Switch", value: $json.readStr(content) };
            case "Where": {
                const obj = $json.readObject(content, "KeywordEnum::Where");
                return {
                    kind: "Where",
                    in: $json.readI32($json.field(obj, "in")),
                    default: $json.readStr($json.field(obj, "default")),
                };
            }
            default: throw $json.unknownVariant("KeywordEnum", variant);
        }
    }

    export function jsonSerializeKeywordEnum(value: KeywordEnum): string {
        return $json.stringify(toJsonKeywordEnum(value));
    }

    export function jsonDeserializeKeywordEnum(text: string): KeywordEnum {
        return fromJsonKeywordEnum($json.parse(text));
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
    #[facet(tag = "type")]
    #[allow(unused)]
    enum Shape {
        Unit,
        Wrapped(Point),
        Struct { x: i32, c: char },
    }

    let actual = emit!(Shape as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class Point {
        constructor (public x: int32) {
        }

        static toJson(value: Point): $json.JsonValue {
            return {
                "x": value.x,
            };
        }

        static fromJson(json: unknown): Point {
            const obj = $json.readObject(json, "Point");
            return new Point(
                $json.readI32($json.field(obj, "x")),
            );
        }

        static jsonSerialize(value: Point): string {
            return $json.stringify(Point.toJson(value));
        }

        static jsonDeserialize(text: string): Point {
            return Point.fromJson($json.parse(text));
        }
    }


    export type Shape =
        | { type: "Unit" }
        | { type: "Wrapped" } & Point
        | { type: "Struct"; x: int32; c: char };

    export const shapeUnit = (): Shape => ({ type: "Unit" });

    export const shapeWrapped = (value: Point): Shape => ({ type: "Wrapped", ...value });

    export const shapeStruct = (x: int32, c: char): Shape => ({ type: "Struct", x, c });

    export function matchShape<R>(value: Shape, cases: {
        Unit: (v: Extract<Shape, { type: "Unit" }>) => R;
        Wrapped: (v: Extract<Shape, { type: "Wrapped" }>) => R;
        Struct: (v: Extract<Shape, { type: "Struct" }>) => R;
    }): R {
        return cases[value.type as Shape["type"]](value as never);
    }

    export function toJsonShape(value: Shape): $json.JsonValue {
        switch (value.type) {
            case "Unit": return { "type": "Unit" };
            case "Wrapped": return $json.writeTagged("type", "Wrapped", Point.toJson(value));
            case "Struct": return {
                "type": "Struct",
                "x": value.x,
                "c": value.c,
            };
            default: throw $json.unknownVariant("Shape", value);
        }
    }

    export function fromJsonShape(json: unknown): Shape {
        const [variant, obj] = $json.readInternal(json, "type", "Shape");
        switch (variant) {
            case "Unit": return { type: "Unit" };
            case "Wrapped": return { type: "Wrapped", ...Point.fromJson(obj) };
            case "Struct": return {
                type: "Struct",
                x: $json.readI32($json.field(obj, "x")),
                c: $json.readChar($json.field(obj, "c")),
            };
            default: throw $json.unknownVariant("Shape", variant);
        }
    }

    export function jsonSerializeShape(value: Shape): string {
        return $json.stringify(toJsonShape(value));
    }

    export function jsonDeserializeShape(text: string): Shape {
        return fromJsonShape($json.parse(text));
    }
    "#);
}

#[test]
fn internally_tagged_unit_enum() {
    #[derive(Facet)]
    #[repr(C)]
    #[facet(tag = "kind")]
    #[allow(unused)]
    enum Mode {
        Fast,
        Slow,
    }

    let actual = emit!(Mode as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type Mode =
        | { kind: "Fast" }
        | { kind: "Slow" };

    export const modeFast = (): Mode => ({ kind: "Fast" });

    export const modeSlow = (): Mode => ({ kind: "Slow" });

    export function matchMode<R>(value: Mode, cases: {
        Fast: (v: Extract<Mode, { kind: "Fast" }>) => R;
        Slow: (v: Extract<Mode, { kind: "Slow" }>) => R;
    }): R {
        return cases[value.kind as Mode["kind"]](value as never);
    }

    export function toJsonMode(value: Mode): $json.JsonValue {
        switch (value.kind) {
            case "Fast": return { "kind": "Fast" };
            case "Slow": return { "kind": "Slow" };
            default: throw $json.unknownVariant("Mode", value);
        }
    }

    export function fromJsonMode(json: unknown): Mode {
        const [variant] = $json.readInternal(json, "kind", "Mode");
        switch (variant) {
            case "Fast": return { kind: "Fast" };
            case "Slow": return { kind: "Slow" };
            default: throw $json.unknownVariant("Mode", variant);
        }
    }

    export function jsonSerializeMode(value: Mode): string {
        return $json.stringify(toJsonMode(value));
    }

    export function jsonDeserializeMode(text: string): Mode {
        return fromJsonMode($json.parse(text));
    }
    "#);
}

#[test]
fn adjacently_tagged_enum() {
    #[derive(Facet)]
    #[repr(C)]
    #[facet(tag = "t", content = "c")]
    #[allow(unused)]
    enum Message {
        Unit,
        NewType(Option<u8>),
        Tuple(u8, String),
        Struct { name: String },
    }

    let actual = emit!(Message as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type Message =
        | { t: "Unit" }
        | { t: "NewType"; c: Optional<uint8> }
        | { t: "Tuple"; c: [uint8, str] }
        | { t: "Struct"; c: { name: str; } };

    export const messageUnit = (): Message => ({ t: "Unit" });

    export const messageNewType = (value: Optional<uint8>): Message => ({ t: "NewType", c: value });

    export const messageTuple = (field0: uint8, field1: str): Message => ({ t: "Tuple", c: [field0, field1] });

    export const messageStruct = (name: str): Message => ({ t: "Struct", c: { name } });

    export function matchMessage<R>(value: Message, cases: {
        Unit: (v: Extract<Message, { t: "Unit" }>) => R;
        NewType: (v: Extract<Message, { t: "NewType" }>) => R;
        Tuple: (v: Extract<Message, { t: "Tuple" }>) => R;
        Struct: (v: Extract<Message, { t: "Struct" }>) => R;
    }): R {
        return cases[value.t as Message["t"]](value as never);
    }

    export function toJsonMessage(value: Message): $json.JsonValue {
        switch (value.t) {
            case "Unit": return { "t": "Unit" };
            case "NewType": return { "t": "NewType", "c": value.c };
            case "Tuple": return { "t": "Tuple", "c": [value.c[0], value.c[1]] };
            case "Struct": return {
                "t": "Struct",
                "c": {
                    "name": value.c.name,
                },
            };
            default: throw $json.unknownVariant("Message", value);
        }
    }

    export function fromJsonMessage(json: unknown): Message {
        const [variant, content] = $json.readAdjacent(json, "t", "c", "Message", ["Unit"]);
        switch (variant) {
            case "Unit": return { t: "Unit" };
            case "NewType": return { t: "NewType", c: $json.readOption(content, $json.readU8) };
            case "Tuple": return { t: "Tuple", c: $json.readTuple<[uint8, str]>(content, [$json.readU8, $json.readStr]) };
            case "Struct": {
                const obj = $json.readObject(content, "Message::Struct");
                return {
                    t: "Struct",
                    c: {
                        name: $json.readStr($json.field(obj, "name")),
                    },
                };
            }
            default: throw $json.unknownVariant("Message", variant);
        }
    }

    export function jsonSerializeMessage(value: Message): string {
        return $json.stringify(toJsonMessage(value));
    }

    export function jsonDeserializeMessage(text: string): Message {
        return fromJsonMessage($json.parse(text));
    }
    "#);
}

#[test]
fn renamed_fields_and_variants() {
    #[derive(Facet)]
    #[facet(rename_all = "camelCase")]
    struct Renamed {
        snake_case_field: u8,
        #[facet(rename = "with-dash")]
        dashed: u8,
        #[facet(rename = "$ref")]
        reference: String,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Choice {
        #[facet(rename = "HIGH")]
        High,
        #[facet(rename = "Other")]
        Renamed {
            #[facet(rename = "bee")]
            b: Option<String>,
            #[facet(rename = "with-dash")]
            dashed: u8,
        },
    }

    let actual = emit!(Renamed, Choice as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export type Choice =
        | { kind: "HIGH" }
        | { kind: "Other"; bee: Optional<str>; "with-dash": uint8 };

    export const choiceHigh = (): Choice => ({ kind: "HIGH" });

    export const choiceOther = (bee: Optional<str>, with_dash: uint8): Choice => ({ kind: "Other", bee, "with-dash": with_dash });

    export function matchChoice<R>(value: Choice, cases: {
        HIGH: (v: Extract<Choice, { kind: "HIGH" }>) => R;
        Other: (v: Extract<Choice, { kind: "Other" }>) => R;
    }): R {
        return cases[value.kind as Choice["kind"]](value as never);
    }

    export function toJsonChoice(value: Choice): $json.JsonValue {
        switch (value.kind) {
            case "HIGH": return "HIGH";
            case "Other": return {
                "Other": {
                    "bee": value.bee,
                    "with-dash": value["with-dash"],
                },
            };
            default: throw $json.unknownVariant("Choice", value);
        }
    }

    export function fromJsonChoice(json: unknown): Choice {
        const [variant, content] = $json.readExternal(json, "Choice", ["HIGH"]);
        switch (variant) {
            case "HIGH": return { kind: "HIGH" };
            case "Other": {
                const obj = $json.readObject(content, "Choice::Other");
                return {
                    kind: "Other",
                    bee: $json.readOption($json.field(obj, "bee"), $json.readStr),
                    "with-dash": $json.readU8($json.field(obj, "with-dash")),
                };
            }
            default: throw $json.unknownVariant("Choice", variant);
        }
    }

    export function jsonSerializeChoice(value: Choice): string {
        return $json.stringify(toJsonChoice(value));
    }

    export function jsonDeserializeChoice(text: string): Choice {
        return fromJsonChoice($json.parse(text));
    }


    export class Renamed {
        public snakeCaseField: uint8;
        public "with-dash": uint8;
        public $ref: str;

        constructor (snakeCaseField: uint8, with_dash: uint8, $ref: str) {
            this.snakeCaseField = snakeCaseField;
            this["with-dash"] = with_dash;
            this.$ref = $ref;
        }

        static toJson(value: Renamed): $json.JsonValue {
            return {
                "snakeCaseField": value.snakeCaseField,
                "with-dash": value["with-dash"],
                "$ref": value.$ref,
            };
        }

        static fromJson(json: unknown): Renamed {
            const obj = $json.readObject(json, "Renamed");
            return new Renamed(
                $json.readU8($json.field(obj, "snakeCaseField")),
                $json.readU8($json.field(obj, "with-dash")),
                $json.readStr($json.field(obj, "$ref")),
            );
        }

        static jsonSerialize(value: Renamed): string {
            return $json.stringify(Renamed.toJson(value));
        }

        static jsonDeserialize(text: string): Renamed {
            return Renamed.fromJson($json.parse(text));
        }
    }
    "#);
}

#[test]
fn struct_with_map_keys_rust_writes_as_strings() {
    #[derive(Facet, PartialEq, Eq, PartialOrd, Ord)]
    struct Key(String);

    #[derive(Facet, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(C)]
    #[allow(unused)]
    enum Level {
        Low,
        High,
    }

    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
    struct Maps {
        by_big: BTreeMap<u128, u8>,
        by_bool: BTreeMap<bool, u8>,
        by_char: BTreeMap<char, u8>,
        by_level: BTreeMap<Level, u8>,
        by_key: BTreeMap<Key, Option<i128>>,
        by_uuid: BTreeMap<uuid::Uuid, u8>,
    }

    let actual = emit!(Maps as TypeScript with JsonPlugin).unwrap();
    insta::assert_snapshot!(actual, @r#"


    export class Key {
        constructor (public value: str) {
        }

        static toJson(value: Key): $json.JsonValue {
            return value.value;
        }

        static fromJson(json: unknown): Key {
            return new Key($json.readStr(json));
        }

        static jsonSerialize(value: Key): string {
            return $json.stringify(Key.toJson(value));
        }

        static jsonDeserialize(text: string): Key {
            return Key.fromJson($json.parse(text));
        }
    }


    export type Level =
        | { kind: "Low" }
        | { kind: "High" };

    export const levelLow = (): Level => ({ kind: "Low" });

    export const levelHigh = (): Level => ({ kind: "High" });

    export function matchLevel<R>(value: Level, cases: {
        Low: (v: Extract<Level, { kind: "Low" }>) => R;
        High: (v: Extract<Level, { kind: "High" }>) => R;
    }): R {
        return cases[value.kind as Level["kind"]](value as never);
    }

    export function toJsonLevel(value: Level): $json.JsonValue {
        switch (value.kind) {
            case "Low": return "Low";
            case "High": return "High";
            default: throw $json.unknownVariant("Level", value);
        }
    }

    export function fromJsonLevel(json: unknown): Level {
        const [variant] = $json.readExternal(json, "Level", ["Low", "High"]);
        switch (variant) {
            case "Low": return { kind: "Low" };
            case "High": return { kind: "High" };
            default: throw $json.unknownVariant("Level", variant);
        }
    }

    export function jsonSerializeLevel(value: Level): string {
        return $json.stringify(toJsonLevel(value));
    }

    export function jsonDeserializeLevel(text: string): Level {
        return fromJsonLevel($json.parse(text));
    }


    export class Maps {
        constructor (public by_big: Map<uint128,uint8>, public by_bool: Map<bool,uint8>, public by_char: Map<char,uint8>, public by_level: Map<Level,uint8>, public by_key: Map<Key,Optional<int128>>, public by_uuid: Map<Uuid,uint8>) {
        }

        static toJson(value: Maps): $json.JsonValue {
            return {
                "by_big": $json.writeMap(value.by_big, (k0) => `${k0}`, (v0) => v0),
                "by_bool": $json.writeMap(value.by_bool, (k0) => `${k0}`, (v0) => v0),
                "by_char": $json.writeMap(value.by_char, (k0) => k0, (v0) => v0),
                "by_level": $json.writeMap(value.by_level, (k0) => $json.writeKey(toJsonLevel(k0)), (v0) => v0),
                "by_key": $json.writeMap(value.by_key, (k0) => $json.writeKey(Key.toJson(k0)), (v0) => (v0 === null ? null : $json.writeBigInt(v0))),
                "by_uuid": $json.writeMap(value.by_uuid, (k0) => k0, (v0) => v0),
            };
        }

        static fromJson(json: unknown): Maps {
            const obj = $json.readObject(json, "Maps");
            return new Maps(
                $json.readMap($json.field(obj, "by_big"), (k0) => $json.readU128($json.keyLiteral(k0)), $json.readU8),
                $json.readMap($json.field(obj, "by_bool"), (k0) => $json.readBool($json.keyLiteral(k0)), $json.readU8),
                $json.readMap($json.field(obj, "by_char"), (k0) => $json.readChar(k0), $json.readU8),
                $json.readMap($json.field(obj, "by_level"), (k0) => $json.readKey(k0, fromJsonLevel), $json.readU8),
                $json.readMap($json.field(obj, "by_key"), (k0) => $json.readKey(k0, Key.fromJson), (j0) => $json.readOption(j0, $json.readI128)),
                $json.readMap($json.field(obj, "by_uuid"), (k0) => $json.readUuid(k0) as Uuid, $json.readU8),
            );
        }

        static jsonSerialize(value: Maps): string {
            return $json.stringify(Maps.toJson(value));
        }

        static jsonDeserialize(text: string): Maps {
            return Maps.fromJson($json.parse(text));
        }
    }
    "#);
}

/// The module for `[T; N]` in every nested position, with JSON (#190).
#[test]
fn fixed_size_arrays_module() {
    insta::assert_snapshot!(super::tests::grid_module(vec![Arc::new(JsonPlugin)]), @r#"
    import * as $json from "./serde/json";
    type bool = boolean;
    type int32 = number;
    type ListTuple<T extends any[]> = T[];
    type Optional<T> = T | null;
    type Seq<T> = T[];
    type str = string;
    type uint16 = number;
    type uint8 = number;

    export type Cell =
        | { kind: "Empty" }
        | { kind: "Filled"; value: ListTuple<[uint8]> }
        | { kind: "Named"; values: ListTuple<[str]> };

    export const cellEmpty = (): Cell => ({ kind: "Empty" });

    export const cellFilled = (value: ListTuple<[uint8]>): Cell => ({ kind: "Filled", value });

    export const cellNamed = (values: ListTuple<[str]>): Cell => ({ kind: "Named", values });

    export function matchCell<R>(value: Cell, cases: {
        Empty: (v: Extract<Cell, { kind: "Empty" }>) => R;
        Filled: (v: Extract<Cell, { kind: "Filled" }>) => R;
        Named: (v: Extract<Cell, { kind: "Named" }>) => R;
    }): R {
        return cases[value.kind as Cell["kind"]](value as never);
    }

    export function toJsonCell(value: Cell): $json.JsonValue {
        switch (value.kind) {
            case "Empty": return "Empty";
            case "Filled": return { "Filled": value.value.map((v0) => v0[0]) };
            case "Named": return {
                "Named": {
                    "values": value.values.map((v0) => v0[0]),
                },
            };
            default: throw $json.unknownVariant("Cell", value);
        }
    }

    export function fromJsonCell(json: unknown): Cell {
        const [variant, content] = $json.readExternal(json, "Cell", ["Empty"]);
        switch (variant) {
            case "Empty": return { kind: "Empty" };
            case "Filled": return { kind: "Filled", value: $json.readSeq(content, (j0): [uint8] => [$json.readU8(j0)], 2) };
            case "Named": {
                const obj = $json.readObject(content, "Cell::Named");
                return {
                    kind: "Named",
                    values: $json.readSeq($json.field(obj, "values"), (j0): [str] => [$json.readStr(j0)], 2),
                };
            }
            default: throw $json.unknownVariant("Cell", variant);
        }
    }

    export function jsonSerializeCell(value: Cell): string {
        return $json.stringify(toJsonCell(value));
    }

    export function jsonDeserializeCell(text: string): Cell {
        return fromJsonCell($json.parse(text));
    }

    export class Grid {
        constructor (public cells: ListTuple<[uint8]>, public maybe: Optional<ListTuple<[uint16]>>, public rows: Seq<ListTuple<[int32]>>, public by_name: Map<str,ListTuple<[bool]>>, public nested: ListTuple<[ListTuple<[uint8]>]>, public cell: Cell) {
        }

        static toJson(value: Grid): $json.JsonValue {
            return {
                "cells": value.cells.map((v0) => v0[0]),
                "maybe": (value.maybe === null ? null : value.maybe.map((v0) => v0[0])),
                "rows": value.rows.map((v0) => v0.map((v1) => v1[0])),
                "by_name": $json.writeMap(value.by_name, (k0) => k0, (v0) => v0.map((v1) => v1[0])),
                "nested": value.nested.map((v0) => v0[0].map((v1) => v1[0])),
                "cell": toJsonCell(value.cell),
            };
        }

        static fromJson(json: unknown): Grid {
            const obj = $json.readObject(json, "Grid");
            return new Grid(
                $json.readSeq($json.field(obj, "cells"), (j0): [uint8] => [$json.readU8(j0)], 4),
                $json.readOption($json.field(obj, "maybe"), (j0) => $json.readSeq(j0, (j1): [uint16] => [$json.readU16(j1)], 2)),
                $json.readSeq($json.field(obj, "rows"), (j0) => $json.readSeq(j0, (j1): [int32] => [$json.readI32(j1)], 3)),
                $json.readMap($json.field(obj, "by_name"), (k0) => k0, (j0) => $json.readSeq(j0, (j1): [bool] => [$json.readBool(j1)], 2)),
                $json.readSeq($json.field(obj, "nested"), (j0): [ListTuple<[uint8]>] => [$json.readSeq(j0, (j1): [uint8] => [$json.readU8(j1)], 2)], 3),
                fromJsonCell($json.field(obj, "cell")),
            );
        }

        static jsonSerialize(value: Grid): string {
            return $json.stringify(Grid.toJson(value));
        }

        static jsonDeserialize(text: string): Grid {
            return Grid.fromJson($json.parse(text));
        }
    }
    "#);
}
