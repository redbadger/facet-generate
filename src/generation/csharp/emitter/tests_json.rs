//! Snapshot tests for the C# emitter — **JSON encoding**.
//!
//! Mirrors [`super::tests`] but with [`Encoding::Json`]. Generated types
//! include `[JsonPropertyName]` attributes on fields,
//! `[JsonConverter(typeof(JsonStringEnumConverter))]` on unit enums,
//! `[JsonPolymorphic]`/`[JsonDerivedType]` on variant hierarchies, and
//! `JsonSerialize`/`JsonDeserialize` convenience methods backed by the
//! `JsonSerde` static helper.

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use facet::Facet;

use super::*;
use crate::emit;

#[test]
fn unit_struct() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public sealed record UnitStruct {
        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static UnitStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<UnitStruct>(input);
        }
    }
    ");
}

#[test]
fn newtype_struct() {
    #[derive(Facet)]
    struct NewType(String);

    let actual = emit!(NewType as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class NewType : ObservableObject {
        [JsonPropertyName("value")]
        [ObservableProperty]
        private string _value;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static NewType JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<NewType>(input);
        }
    }
    "#);
}

#[test]
fn tuple_struct() {
    #[derive(Facet)]
    struct TupleStruct(String, i32);

    let actual = emit!(TupleStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class TupleStruct : ObservableObject {
        [JsonPropertyName("field0")]
        [ObservableProperty]
        private string _field0;
        [JsonPropertyName("field1")]
        [ObservableProperty]
        private int _field1;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static TupleStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<TupleStruct>(input);
        }
    }
    "#);
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

    let actual = emit!(StructWithFields as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class StructWithFields : ObservableObject {
        [JsonPropertyName("unit")]
        [ObservableProperty]
        private Unit _unit;
        [JsonPropertyName("bool")]
        [ObservableProperty]
        private bool _bool;
        [JsonPropertyName("i8")]
        [ObservableProperty]
        private sbyte _i8;
        [JsonPropertyName("i16")]
        [ObservableProperty]
        private short _i16;
        [JsonPropertyName("i32")]
        [ObservableProperty]
        private int _i32;
        [JsonPropertyName("i64")]
        [ObservableProperty]
        private long _i64;
        [JsonPropertyName("i128")]
        [ObservableProperty]
        private Int128 _i128;
        [JsonPropertyName("u8")]
        [ObservableProperty]
        private byte _u8;
        [JsonPropertyName("u16")]
        [ObservableProperty]
        private ushort _u16;
        [JsonPropertyName("u32")]
        [ObservableProperty]
        private uint _u32;
        [JsonPropertyName("u64")]
        [ObservableProperty]
        private ulong _u64;
        [JsonPropertyName("u128")]
        [ObservableProperty]
        private UInt128 _u128;
        [JsonPropertyName("f32")]
        [ObservableProperty]
        private float _f32;
        [JsonPropertyName("f64")]
        [ObservableProperty]
        private double _f64;
        [JsonPropertyName("char")]
        [ObservableProperty]
        private char _char;
        [JsonPropertyName("string")]
        [ObservableProperty]
        private string _string;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static StructWithFields JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<StructWithFields>(input);
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

    let actual = emit!(Outer as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class Inner1 : ObservableObject {
        [JsonPropertyName("field1")]
        [ObservableProperty]
        private string _field1;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Inner1 JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Inner1>(input);
        }
    }

    public partial class Inner2 : ObservableObject {
        [JsonPropertyName("value")]
        [ObservableProperty]
        private string _value;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Inner2 JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Inner2>(input);
        }
    }

    public partial class Inner3 : ObservableObject {
        [JsonPropertyName("field0")]
        [ObservableProperty]
        private string _field0;
        [JsonPropertyName("field1")]
        [ObservableProperty]
        private int _field1;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Inner3 JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Inner3>(input);
        }
    }

    public partial class Outer : ObservableObject {
        [JsonPropertyName("one")]
        [ObservableProperty]
        private Inner1 _one;
        [JsonPropertyName("two")]
        [ObservableProperty]
        private Inner2 _two;
        [JsonPropertyName("three")]
        [ObservableProperty]
        private Inner3 _three;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static Outer JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<Outer>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("one")]
        [ObservableProperty]
        private (string, int) _one;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("one")]
        [ObservableProperty]
        private (string, int, ushort) _one;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("one")]
        [ObservableProperty]
        private (string, int, ushort, float) _one;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(EnumWithUnitVariants as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonConverter(typeof(JsonStringEnumConverter))]
    public enum EnumWithUnitVariants {
        Variant1,
        Variant2,
        Variant3
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

    let actual = emit!(MyEnum as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @"

    [JsonConverter(typeof(JsonStringEnumConverter))]
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

    let actual = emit!(MyEnum as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
    [JsonDerivedType(typeof(Variant1), "Variant1")]
    public abstract record MyEnum {
        public sealed record Variant1(string Value) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
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

    let actual = emit!(MyEnum as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
    [JsonDerivedType(typeof(Variant1), "Variant1")]
    [JsonDerivedType(typeof(Variant2), "Variant2")]
    public abstract record MyEnum {
        public sealed record Variant1(string Value) : MyEnum;

        public sealed record Variant2(int Value) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
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

    let actual = emit!(MyEnum as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
    [JsonDerivedType(typeof(Variant1), "Variant1")]
    [JsonDerivedType(typeof(Variant2), "Variant2")]
    public abstract record MyEnum {
        public sealed record Variant1(string Field0, int Field1) : MyEnum;

        public sealed record Variant2(bool Field0, double Field1, byte Field2) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
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

    let actual = emit!(MyEnum as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
    [JsonDerivedType(typeof(Variant1), "Variant1")]
    public abstract record MyEnum {
        public sealed record Variant1(string Field1, int Field2) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
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

    let actual = emit!(MyEnum as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    [JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
    [JsonDerivedType(typeof(Unit), "Unit")]
    [JsonDerivedType(typeof(NewType), "NewType")]
    [JsonDerivedType(typeof(Tuple), "Tuple")]
    [JsonDerivedType(typeof(Struct), "Struct")]
    public abstract record MyEnum {
        public sealed record Unit() : MyEnum;

        public sealed record NewType(string Value) : MyEnum;

        public sealed record Tuple(string Field0, int Field1) : MyEnum;

        public sealed record Struct(bool Field) : MyEnum;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyEnum JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyEnum>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("items")]
        [ObservableProperty]
        private ObservableCollection<string> _items;
        [JsonPropertyName("numbers")]
        [ObservableProperty]
        private ObservableCollection<int> _numbers;
        [JsonPropertyName("nestedItems")]
        [ObservableProperty]
        private ObservableCollection<ObservableCollection<string>> _nestedItems;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("optionalString")]
        [ObservableProperty]
        private string? _optionalString;
        [JsonPropertyName("optionalNumber")]
        [ObservableProperty]
        private int? _optionalNumber;
        [JsonPropertyName("optionalBool")]
        [ObservableProperty]
        private bool? _optionalBool;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("stringToInt")]
        [ObservableProperty]
        private Dictionary<string, int> _stringToInt;
        [JsonPropertyName("intToBool")]
        [ObservableProperty]
        private Dictionary<int, bool> _intToBool;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("optionalList")]
        [ObservableProperty]
        private ObservableCollection<string>? _optionalList;
        [JsonPropertyName("listOfOptionals")]
        [ObservableProperty]
        private ObservableCollection<int?> _listOfOptionals;
        [JsonPropertyName("mapToList")]
        [ObservableProperty]
        private Dictionary<string, ObservableCollection<bool>> _mapToList;
        [JsonPropertyName("optionalMap")]
        [ObservableProperty]
        private Dictionary<string, int>? _optionalMap;
        [JsonPropertyName("complex")]
        [ObservableProperty]
        private ObservableCollection<Dictionary<string, ObservableCollection<bool>>?> _complex;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("fixedArray")]
        [ObservableProperty]
        private int[] _fixedArray;
        [JsonPropertyName("byteArray")]
        [ObservableProperty]
        private byte[] _byteArray;
        [JsonPropertyName("stringArray")]
        [ObservableProperty]
        private string[] _stringArray;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("stringToInt")]
        [ObservableProperty]
        private Dictionary<string, int> _stringToInt;
        [JsonPropertyName("intToBool")]
        [ObservableProperty]
        private Dictionary<int, bool> _intToBool;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("stringSet")]
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [JsonPropertyName("intSet")]
        [ObservableProperty]
        private HashSet<int> _intSet;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("stringSet")]
        [ObservableProperty]
        private HashSet<string> _stringSet;
        [JsonPropertyName("intSet")]
        [ObservableProperty]
        private HashSet<int> _intSet;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("boxedString")]
        [ObservableProperty]
        private string _boxedString;
        [JsonPropertyName("boxedInt")]
        [ObservableProperty]
        private int _boxedInt;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("rcString")]
        [ObservableProperty]
        private string _rcString;
        [JsonPropertyName("rcInt")]
        [ObservableProperty]
        private int _rcInt;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("arcString")]
        [ObservableProperty]
        private string _arcString;
        [JsonPropertyName("arcInt")]
        [ObservableProperty]
        private int _arcInt;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("vecOfSets")]
        [ObservableProperty]
        private ObservableCollection<HashSet<string>> _vecOfSets;
        [JsonPropertyName("optionalBtree")]
        [ObservableProperty]
        private Dictionary<string, int>? _optionalBtree;
        [JsonPropertyName("boxedVec")]
        [ObservableProperty]
        private ObservableCollection<string> _boxedVec;
        [JsonPropertyName("arcOption")]
        [ObservableProperty]
        private string? _arcOption;
        [JsonPropertyName("arrayOfBoxes")]
        [ObservableProperty]
        private int[] _arrayOfBoxes;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("data")]
        [ObservableProperty]
        private byte[] _data;
        [JsonPropertyName("name")]
        [ObservableProperty]
        private string _name;
        [JsonPropertyName("header")]
        [ObservableProperty]
        private byte[] _header;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
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

    let actual = emit!(MyStruct as CSharp with Encoding::Json).unwrap();
    insta::assert_snapshot!(actual, @r#"

    public partial class MyStruct : ObservableObject {
        [JsonPropertyName("data")]
        [ObservableProperty]
        private byte[] _data;
        [JsonPropertyName("name")]
        [ObservableProperty]
        private string _name;
        [JsonPropertyName("header")]
        [ObservableProperty]
        private byte[] _header;
        [JsonPropertyName("optionalBytes")]
        [ObservableProperty]
        private ObservableCollection<byte>? _optionalBytes;

        public string JsonSerialize()
        {
            return JsonSerde.Serialize(this);
        }

        public static MyStruct JsonDeserialize(string input)
        {
            return JsonSerde.Deserialize<MyStruct>(input);
        }
    }
    "#);
}
