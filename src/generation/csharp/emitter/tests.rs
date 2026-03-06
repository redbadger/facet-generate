#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, HashMap, HashSet};

use facet::Facet;

use super::*;
use crate::emit;

#[test]
fn unit_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as CSharp with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public partial class UnitStruct : ObservableObject {}
    ");
}

#[test]
fn newtype_struct() {
    #[derive(Facet)]
    struct NewType(String);

    let actual = emit!(NewType as CSharp with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class NewType : ObservableObject {
        public string Value { get; set; }
    }
    ");
}

#[test]
fn struct_with_fields_of_primitive_types() {
    #[derive(Facet)]
    #[allow(clippy::struct_field_names)]
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

    let actual = emit!(StructWithFields as CSharp with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class StructWithFields : ObservableObject {
        public Unit Unit { get; set; }
        public bool Bool { get; set; }
        public sbyte I8 { get; set; }
        public short I16 { get; set; }
        public int I32 { get; set; }
        public long I64 { get; set; }
        public Int128 I128 { get; set; }
        public byte U8 { get; set; }
        public ushort U16 { get; set; }
        public uint U32 { get; set; }
        public ulong U64 { get; set; }
        public UInt128 U128 { get; set; }
        public float F32 { get; set; }
        public double F64 { get; set; }
        public char Char { get; set; }
        public string String { get; set; }
    }
    ");
}

#[test]
fn struct_with_nested_types() {
    #[derive(Facet)]
    struct MyStruct {
        optional_list: Option<Vec<String>>,
        list_of_optionals: Vec<Option<i32>>,
        map_to_list: HashMap<String, Vec<bool>>,
        optional_map: Option<HashMap<String, i32>>,
        set_of_values: HashSet<u16>,
        tuple_value: (String, i32, bool),
        fixed_array: [i32; 4],
    }

    let actual = emit!(MyStruct as CSharp with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        public Option<ObservableCollection<string>> OptionalList { get; set; }
        public ObservableCollection<Option<int>> ListOfOptionals { get; set; }
        public Dictionary<string, ObservableCollection<bool>> MapToList { get; set; }
        public Option<Dictionary<string, int>> OptionalMap { get; set; }
        public HashSet<ushort> SetOfValues { get; set; }
        public (string, int, bool) TupleValue { get; set; }
        public int[] FixedArray { get; set; }
    }
    ");
}

#[test]
fn struct_with_bytes_field() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(bytes)]
        data: Vec<u8>,
        name: String,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        public byte[] Data { get; set; }
        public string Name { get; set; }
    }
    ");
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

    let actual = emit!(EnumWithUnitVariants as CSharp with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public enum EnumWithUnitVariants {
        Variant1,
        Variant2,
        Variant3
    }
    ");
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

    let actual = emit!(MyEnum as CSharp with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public abstract record MyEnum {
        public sealed record Unit() : MyEnum;

        public sealed record NewType(string Value) : MyEnum;

        public sealed record Tuple(string Field0, int Field1) : MyEnum;

        public sealed record Struct(bool Field) : MyEnum;

    }
    ");
}

#[test]
fn struct_with_external_namespace_type() {
    #[derive(Facet)]
    #[facet(namespace = "external_models")]
    struct Child {
        value: String,
    }

    #[derive(Facet)]
    struct Parent {
        child: Child,
        maybe_child: Option<Child>,
        many_children: Vec<Child>,
    }

    let actual = emit!(Parent as CSharp with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class Parent : ObservableObject {
        public ExternalModels.Child Child { get; set; }
        public Option<ExternalModels.Child> MaybeChild { get; set; }
        public ObservableCollection<ExternalModels.Child> ManyChildren { get; set; }
    }

    public partial class Child : ObservableObject {
        public string Value { get; set; }
    }
    ");
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        map: BTreeMap<String, i32>,
    }

    let actual = emit!(MyStruct as CSharp with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        public Dictionary<string, int> Map { get; set; }
    }
    ");
}
