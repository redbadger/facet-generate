//! these tests will be updated once the Swift emitter is converted
//! to use the new Emitter<Language> trait
#![allow(clippy::too_many_lines)]

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use crate as fg;
use facet::Facet;

use super::*;
use crate::{emit, emit_two_modules, generation::swift::CodeGenerator};

#[test]
fn unit_struct_1() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit!(UnitStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct UnitStruct {
        public init() {
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

    let actual = emit!(UnitStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct UnitStruct {
        public init() {
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

    let actual = emit!(NewType as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct NewType {
        public var value: String

        public init(value: String) {
            self.value = value
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

    let actual = emit!(TupleStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct TupleStruct {
        public var field0: String
        public var field1: Int32

        public init(field0: String, field1: Int32) {
            self.field0 = field0
            self.field1 = field1
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

    let actual = emit!(StructWithFields as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line 1
    /// line 2
    public struct StructWithFields {
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

    let actual = emit!(Outer as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct Inner1 {
        public var field1: String

        public init(field1: String) {
            self.field1 = field1
        }
    }

    public struct Inner2 {
        public var value: String

        public init(value: String) {
            self.value = value
        }
    }

    public struct Inner3 {
        public var field0: String
        public var field1: Int32

        public init(field0: String, field1: Int32) {
            self.field0 = field0
            self.field1 = field1
        }
    }

    public struct Outer {
        public var one: Inner1
        public var two: Inner2
        public var three: Inner3

        public init(one: Inner1, two: Inner2, three: Inner3) {
            self.one = one
            self.two = two
            self.three = three
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var one: (String, Int32)

        public init(one: (String, Int32)) {
            self.one = one
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var one: (String, Int32, UInt16)

        public init(one: (String, Int32, UInt16)) {
            self.one = one
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var one: (String, Int32, UInt16, Float)

        public init(one: (String, Int32, UInt16, Float)) {
            self.one = one
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
        Variant2,
        /// variant three
        Variant3,
    }

    let actual = emit!(EnumWithUnitVariants as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    /// line one
    /// line two
    indirect public enum EnumWithUnitVariants {
        /// variant one
        case variant1
        case variant2
        /// variant three
        case variant3
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

    let actual = emit!(MyEnum as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum {
        case variant1
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

    let actual = emit!(MyEnum as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum {
        case variant1(String)
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

    let actual = emit!(MyEnum as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum {
        case variant1(String)
        case variant2(Int32)
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

    let actual = emit!(MyEnum as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum {
        case variant1(String, Int32)
        case variant2(Bool, Double, UInt8)
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

    let actual = emit!(MyEnum as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum {
        case variant1(field1: String, field2: Int32)
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

    let actual = emit!(MyEnum as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum {
        case unit
        case newType(String)
        case tuple(String, Int32)
        case struct(field: Bool)
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var items: [String]
        public var numbers: [Int32]
        public var nestedItems: [[String]]

        public init(items: [String], numbers: [Int32], nestedItems: [[String]]) {
            self.items = items
            self.numbers = numbers
            self.nestedItems = nestedItems
        }
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var optionalString: String?
        public var optionalNumber: Int32?
        public var optionalBool: Bool?

        public init(optionalString: String?, optionalNumber: Int32?, optionalBool: Bool?) {
            self.optionalString = optionalString
            self.optionalNumber = optionalNumber
            self.optionalBool = optionalBool
        }
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var stringToInt: [String: Int32]
        public var intToBool: [Int32: Bool]

        public init(stringToInt: [String: Int32], intToBool: [Int32: Bool]) {
            self.stringToInt = stringToInt
            self.intToBool = intToBool
        }
    }
    ");
}

#[test]
fn struct_with_nested_generics() {
    #[derive(Facet)]
    struct MyStruct {
        optional_list: Option<Vec<String>>,
        list_of_options: Vec<Option<i32>>,
        map_to_list: HashMap<String, Vec<bool>>,
        optional_map: Option<HashMap<String, i32>>,
        complex: Vec<Option<HashMap<String, Vec<bool>>>>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var optionalList: [String]?
        public var listOfOptions: [Int32?]
        public var mapToList: [String: [Bool]]
        public var optionalMap: [String: Int32]?
        public var complex: [[String: [Bool]]?]

        public init(optionalList: [String]?, listOfOptions: [Int32?], mapToList: [String: [Bool]], optionalMap: [String: Int32]?, complex: [[String: [Bool]]?]) {
            self.optionalList = optionalList
            self.listOfOptions = listOfOptions
            self.mapToList = mapToList
            self.optionalMap = optionalMap
            self.complex = complex
        }
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var fixedArray: [Int32]
        public var byteArray: [UInt8]
        public var stringArray: [String]

        public init(fixedArray: [Int32], byteArray: [UInt8], stringArray: [String]) {
            self.fixedArray = fixedArray
            self.byteArray = byteArray
            self.stringArray = stringArray
        }
    }
    ");
}

#[test]
fn struct_with_map_fields() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var stringToInt: [String: Int32]
        public var intToBool: [Int32: Bool]

        public init(stringToInt: [String: Int32], intToBool: [Int32: Bool]) {
            self.stringToInt = stringToInt
            self.intToBool = intToBool
        }
    }
    ");
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var stringSet: Set<String>
        public var intSet: Set<Int32>

        public init(stringSet: Set<String>, intSet: Set<Int32>) {
            self.stringSet = stringSet
            self.intSet = intSet
        }
    }
    ");
}

#[test]
fn struct_with_set_fields() {
    // NOTE: BTreeSet<T> now maps to Set<T> in Kotlin with the new Format::Set variant.
    // This preserves the uniqueness constraint and provides better type safety.
    #[derive(Facet)]
    struct MyStruct {
        string_set: BTreeSet<String>,
        int_set: BTreeSet<i32>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var stringSet: Set<String>
        public var intSet: Set<Int32>

        public init(stringSet: Set<String>, intSet: Set<Int32>) {
            self.stringSet = stringSet
            self.intSet = intSet
        }
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var boxedString: String
        public var boxedInt: Int32

        public init(boxedString: String, boxedInt: Int32) {
            self.boxedString = boxedString
            self.boxedInt = boxedInt
        }
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var rcString: String
        public var rcInt: Int32

        public init(rcString: String, rcInt: Int32) {
            self.rcString = rcString
            self.rcInt = rcInt
        }
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var arcString: String
        public var arcInt: Int32

        public init(arcString: String, arcInt: Int32) {
            self.arcString = arcString
            self.arcInt = arcInt
        }
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
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
        #[facet(fg::bytes)]
        header: Vec<u8>,
    }

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
        public var data: [UInt8]
        public var name: String
        public var header: [UInt8]

        public init(data: [UInt8], name: String, header: [UInt8]) {
            self.data = data
            self.name = name
            self.header = header
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

    let actual = emit!(MyStruct as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct {
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
    }
    ");
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

    let actual = emit!(Parent as Swift with Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct Parent {
        public var child: Test.Child

        public init(child: Test.Child) {
            self.child = child
        }
    }

    public struct Child {
        public var test: String

        public init(test: String) {
            self.test = test
        }
    }
    ");
}

#[test]
fn type_in_root_and_named_namespace() {
    #[derive(Facet)]
    struct Child {
        value: String,
    }

    mod other {
        use crate as fg;
        use facet::Facet;

        #[derive(Facet)]
        #[facet(fg::namespace = "other")]
        pub struct Child {
            value: i32,
        }
    }

    #[derive(Facet)]
    struct Parent {
        child: Child,
        other_child: other::Child,
    }

    let (other, root) = emit_two_modules!(CodeGenerator, Parent, "root");
    insta::assert_snapshot!(other, @"

    public struct Child {
        public var value: Int32

        public init(value: Int32) {
            self.value = value
        }
    }
    ");

    insta::assert_snapshot!(root, @"
    import Other

    public struct Child {
        public var value: String

        public init(value: String) {
            self.value = value
        }
    }

    public struct Parent {
        public var child: Child
        public var otherChild: Other.Child

        public init(child: Child, otherChild: Other.Child) {
            self.child = child
            self.otherChild = otherChild
        }
    }
    ");
}
