//! Snapshot tests for the C# emitter — **no serialization**.
//!
//! Every test uses the [`emit!`] macro with [`Encoding::None`], producing
//! plain MVVM types verified against `insta` inline snapshots.
//!
//! # Output shapes
//!
//! - `sealed record` — unit structs / empty structs
//! - `partial class : ObservableObject` — structs with fields
//! - `public enum` — enums with only unit variants
//! - `abstract record` + `sealed record` — enums with data variants
//!
//! # Coverage
//!
//! Structs (unit, newtype, tuple, regular, user-type fields), tuples (2/3/4),
//! enums (unit, newtype, tuple, struct, mixed), collections (`Vec`, `HashMap`,
//! `BTreeMap`, `HashSet`, `BTreeSet`, nested generics), optionals, pointers
//! (`Box`, `Rc`, `Arc`), bytes (`#[facet(fg::bytes)]`), fixed-size arrays, and
//! external-namespace types.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use facet::Facet;

use super::*;

use crate::{self as fg, emit};

#[test]
fn unit_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public sealed record UnitStruct;
    ");
}

#[test]
fn newtype_struct() {
    #[derive(Facet)]
    struct NewType(String);

    let actual = emit!(NewType as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class NewType : ObservableObject {
        [ObservableProperty]
        private string _value;
    }
    ");
}

#[test]
fn tuple_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public partial class TupleStruct : ObservableObject {
        [ObservableProperty]
        private string _field0;
        [ObservableProperty]
        private int _field1;
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

    let actual = emit!(StructWithFields as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class StructWithFields : ObservableObject {
        [ObservableProperty]
        private Unit _unit;
        [ObservableProperty]
        private bool _bool;
        [ObservableProperty]
        private sbyte _i8;
        [ObservableProperty]
        private short _i16;
        [ObservableProperty]
        private int _i32;
        [ObservableProperty]
        private long _i64;
        [ObservableProperty]
        private Int128 _i128;
        [ObservableProperty]
        private byte _u8;
        [ObservableProperty]
        private ushort _u16;
        [ObservableProperty]
        private uint _u32;
        [ObservableProperty]
        private ulong _u64;
        [ObservableProperty]
        private UInt128 _u128;
        [ObservableProperty]
        private float _f32;
        [ObservableProperty]
        private double _f64;
        [ObservableProperty]
        private char _char;
        [ObservableProperty]
        private string _string;
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

    let actual = emit!(Outer as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class Inner1 : ObservableObject {
        [ObservableProperty]
        private string _field1;
    }

    public partial class Inner2 : ObservableObject {
        [ObservableProperty]
        private string _value;
    }

    public partial class Inner3 : ObservableObject {
        [ObservableProperty]
        private string _field0;
        [ObservableProperty]
        private int _field1;
    }

    public partial class Outer : ObservableObject {
        [ObservableProperty]
        private Inner1 _one;
        [ObservableProperty]
        private Inner2 _two;
        [ObservableProperty]
        private Inner3 _three;
    }
    ");
}

#[test]
fn struct_with_field_that_is_a_2_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32),
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private (string, int) _one;
    }
    ");
}

#[test]
fn struct_with_field_that_is_a_3_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16),
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private (string, int, ushort) _one;
    }
    ");
}

#[test]
fn struct_with_field_that_is_a_4_tuple() {
    #[derive(Facet)]
    struct MyStruct {
        one: (String, i32, u16, f32),
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private (string, int, ushort, float) _one;
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

    let actual = emit!(EnumWithUnitVariants as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public enum EnumWithUnitVariants {
        Variant1,
        Variant2,
        Variant3
    }
    ");
}

#[test]
fn enum_with_unit_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 {},
    }

    let actual = emit!(MyEnum as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public enum MyEnum {
        Variant1
    }
    ");
}

#[test]
fn enum_with_1_tuple_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1(String),
    }

    let actual = emit!(MyEnum as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public abstract record MyEnum {
        public sealed record Variant1(string Value) : MyEnum;

    }
    ");
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

    let actual = emit!(MyEnum as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public abstract record MyEnum {
        public sealed record Variant1(string Value) : MyEnum;

        public sealed record Variant2(int Value) : MyEnum;

    }
    ");
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

    let actual = emit!(MyEnum as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public abstract record MyEnum {
        public sealed record Variant1(string Field0, int Field1) : MyEnum;

        public sealed record Variant2(bool Field0, double Field1, byte Field2) : MyEnum;

    }
    ");
}

#[test]
fn enum_with_struct_variants() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Variant1 { field1: String, field2: i32 },
    }

    let actual = emit!(MyEnum as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public abstract record MyEnum {
        public sealed record Variant1(string Field1, int Field2) : MyEnum;

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

    let actual = emit!(MyEnum as CSharp).unwrap();
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
fn struct_with_vec_field() {
    #[derive(Facet)]
    struct MyStruct {
        items: Vec<String>,
        numbers: Vec<i32>,
        nested_items: Vec<Vec<String>>,
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private ObservableCollection<string> _items;
        [ObservableProperty]
        private ObservableCollection<int> _numbers;
        [ObservableProperty]
        private ObservableCollection<ObservableCollection<string>> _nestedItems;
    }
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

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private string? _optionalString;
        [ObservableProperty]
        private int? _optionalNumber;
        [ObservableProperty]
        private bool? _optionalBool;
    }
    ");
}

#[test]
fn struct_with_hashmap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        int_to_bool: HashMap<i32, bool>,
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private Dictionary<string, int> _stringToInt;
        [ObservableProperty]
        private Dictionary<int, bool> _intToBool;
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

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private ObservableCollection<string>? _optionalList;
        [ObservableProperty]
        private ObservableCollection<int?> _listOfOptionals;
        [ObservableProperty]
        private Dictionary<string, ObservableCollection<bool>> _mapToList;
        [ObservableProperty]
        private Dictionary<string, int>? _optionalMap;
        [ObservableProperty]
        private HashSet<ushort> _setOfValues;
        [ObservableProperty]
        private (string, int, bool) _tupleValue;
        [ObservableProperty]
        private int[] _fixedArray;
    }
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

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private int[] _fixedArray;
        [ObservableProperty]
        private byte[] _byteArray;
        [ObservableProperty]
        private string[] _stringArray;
    }
    ");
}

#[test]
fn struct_with_bytes_field() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(fg::bytes)]
        data: Vec<u8>,
        name: String,
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private byte[] _data;
        [ObservableProperty]
        private string _name;
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

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private byte[] _data;
        [ObservableProperty]
        private string _name;
        [ObservableProperty]
        private byte[] _header;
        [ObservableProperty]
        private ObservableCollection<byte>? _optionalBytes;
    }
    ");
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        map: BTreeMap<String, i32>,
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private Dictionary<string, int> _map;
    }
    ");
}

#[test]
fn struct_with_hashset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: HashSet<String>,
        int_set: HashSet<i32>,
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [ObservableProperty]
        private HashSet<int> _intSet;
    }
    ");
}

#[test]
fn struct_with_btreeset_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [ObservableProperty]
        private HashSet<int> _intSet;
    }
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

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private string _boxedString;
        [ObservableProperty]
        private int _boxedInt;
    }
    ");
}

#[test]
fn struct_with_rc_field() {
    #[derive(Facet)]
    struct MyStruct {
        rc_string: Rc<String>,
        rc_int: Rc<i32>,
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private string _rcString;
        [ObservableProperty]
        private int _rcInt;
    }
    ");
}

#[test]
fn struct_with_arc_field() {
    #[derive(Facet)]
    struct MyStruct {
        arc_string: Arc<String>,
        arc_int: Arc<i32>,
    }

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private string _arcString;
        [ObservableProperty]
        private int _arcInt;
    }
    ");
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

    let actual = emit!(MyStruct as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class MyStruct : ObservableObject {
        [ObservableProperty]
        private ObservableCollection<HashSet<string>> _vecOfSets;
        [ObservableProperty]
        private Dictionary<string, int>? _optionalBtree;
        [ObservableProperty]
        private ObservableCollection<string> _boxedVec;
        [ObservableProperty]
        private string? _arcOption;
        [ObservableProperty]
        private int[] _arrayOfBoxes;
    }
    ");
}

#[test]
fn struct_with_external_namespace_type() {
    #[derive(Facet)]
    #[facet(fg::namespace = "external_models")]
    struct Child {
        value: String,
    }

    #[derive(Facet)]
    struct Parent {
        child: Child,
        maybe_child: Option<Child>,
        many_children: Vec<Child>,
    }

    let actual = emit!(Parent as CSharp).unwrap();
    insta::assert_snapshot!(actual, @"

    public partial class Parent : ObservableObject {
        [ObservableProperty]
        private ExternalModels.Child _child;
        [ObservableProperty]
        private ExternalModels.Child? _maybeChild;
        [ObservableProperty]
        private ObservableCollection<ExternalModels.Child> _manyChildren;
    }

    public partial class Child : ObservableObject {
        [ObservableProperty]
        private string _value;
    }
    ");
}
