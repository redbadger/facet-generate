//! these tests will be updated once the Swift emitter is converted
//! to use the new Emitter<Language> trait
#![allow(clippy::too_many_lines)]

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use facet::Facet;

use crate::{
    Registry, emit_swift,
    generation::{
        Encoding,
        module::{self, Module},
        swift::CodeGenerator,
    },
    reflect,
};

#[test]
fn unit_struct_1() {
    /// line 1
    #[derive(Facet)]
    /// line 2
    struct UnitStruct;

    let actual = emit_swift!(UnitStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct UnitStruct: Hashable {

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

    let actual = emit_swift!(UnitStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct UnitStruct: Hashable {

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

    let actual = emit_swift!(NewType as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct NewType: Hashable {
        @Indirect public var value: String

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

    let actual = emit_swift!(TupleStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct TupleStruct: Hashable {
        @Indirect public var field0: String
        @Indirect public var field1: Int32

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

    let actual = emit_swift!(StructWithFields as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct StructWithFields: Hashable {
        @Indirect public var unit: Unit
        @Indirect public var bool: Bool
        @Indirect public var i8: Int8
        @Indirect public var i16: Int16
        @Indirect public var i32: Int32
        @Indirect public var i64: Int64
        @Indirect public var i128: Int128
        @Indirect public var u8: UInt8
        @Indirect public var u16: UInt16
        @Indirect public var u32: UInt32
        @Indirect public var u64: UInt64
        @Indirect public var u128: UInt128
        @Indirect public var f32: Float
        @Indirect public var f64: Double
        @Indirect public var char: Character
        @Indirect public var string: String

        public init(unit: Unit, bool: Bool, i8: Int8, i16: Int16, i32: Int32, i64: Int64, i128: Int128, u8: UInt8, u16: UInt16, u32: UInt32, u64: UInt64, u128: UInt128, f32: Float, f64: Double, char: Character, string: String) {
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

    let actual = emit_swift!(Outer as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct Inner1: Hashable {
        @Indirect public var field1: String

        public init(field1: String) {
            self.field1 = field1
        }
    }

    public struct Inner2: Hashable {
        @Indirect public var value: String

        public init(value: String) {
            self.value = value
        }
    }

    public struct Inner3: Hashable {
        @Indirect public var field0: String
        @Indirect public var field1: Int32

        public init(field0: String, field1: Int32) {
            self.field0 = field0
            self.field1 = field1
        }
    }

    public struct Outer: Hashable {
        @Indirect public var one: Inner1
        @Indirect public var two: Inner2
        @Indirect public var three: Inner3

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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var one: Tuple2<String, Int32>

        public init(one: Tuple2<String, Int32>) {
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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var one: Tuple3<String, Int32, UInt16>

        public init(one: Tuple3<String, Int32, UInt16>) {
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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var one: Tuple4<String, Int32, UInt16, Float>

        public init(one: Tuple4<String, Int32, UInt16, Float>) {
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
        /// variant two
        Variant2,
        /// variant three
        Variant3,
    }

    let actual = emit_swift!(EnumWithUnitVariants as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum EnumWithUnitVariants: Hashable {
        case variant1
        case variant2
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

    let actual = emit_swift!(MyEnum as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum: Hashable {
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

    let actual = emit_swift!(MyEnum as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum: Hashable {
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

    let actual = emit_swift!(MyEnum as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum: Hashable {
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

    let actual = emit_swift!(MyEnum as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum: Hashable {
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

    let actual = emit_swift!(MyEnum as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum: Hashable {
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

    let actual = emit_swift!(MyEnum as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    indirect public enum MyEnum: Hashable {
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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var items: [String]
        @Indirect public var numbers: [Int32]
        @Indirect public var nestedItems: [[String]]

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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var optionalString: String?
        @Indirect public var optionalNumber: Int32?
        @Indirect public var optionalBool: Bool?

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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var stringToInt: [String: Int32]
        @Indirect public var intToBool: [Int32: Bool]

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
        list_of_optionals: Vec<Option<i32>>,
        map_to_list: HashMap<String, Vec<bool>>,
        optional_map: Option<HashMap<String, i32>>,
        complex: Vec<Option<HashMap<String, Vec<bool>>>>,
    }

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var fixedArray: [Int32]
        @Indirect public var byteArray: [UInt8]
        @Indirect public var stringArray: [String]

        public init(fixedArray: [Int32], byteArray: [UInt8], stringArray: [String]) {
            self.fixedArray = fixedArray
            self.byteArray = byteArray
            self.stringArray = stringArray
        }
    }
    ");
}

#[test]
fn struct_with_btreemap_field() {
    #[derive(Facet)]
    struct MyStruct {
        string_to_int: BTreeMap<String, i32>,
        int_to_bool: BTreeMap<i32, bool>,
    }

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var stringToInt: [String: Int32]
        @Indirect public var intToBool: [Int32: Bool]

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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var stringSet: [String]
        @Indirect public var intSet: [Int32]

        public init(stringSet: [String], intSet: [Int32]) {
            self.stringSet = stringSet
            self.intSet = intSet
        }
    }
    ");
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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var stringSet: [String]
        @Indirect public var intSet: [Int32]

        public init(stringSet: [String], intSet: [Int32]) {
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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var boxedString: String
        @Indirect public var boxedInt: Int32

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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var rcString: String
        @Indirect public var rcInt: Int32

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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var arcString: String
        @Indirect public var arcInt: Int32

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

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

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
        #[facet(bytes)]
        header: Vec<u8>,
    }

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct MyStruct: Hashable {
        @Indirect public var data: [UInt8]
        @Indirect public var name: String
        @Indirect public var header: [UInt8]

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
        #[facet(bytes)]
        data: &'a [u8],
        name: String,
        #[facet(bytes)]
        header: Vec<u8>,
        optional_bytes: Option<Vec<u8>>,
    }

    let actual = emit_swift!(MyStruct as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

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
    }
    ");
}

#[test]
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

    let actual = emit_swift!(Parent as Encoding::None).unwrap();
    insta::assert_snapshot!(actual, @"

    public struct Parent: Hashable {
        @Indirect public var child: Child

        public init(child: Child) {
            self.child = child
        }
    }

    public struct Child: Hashable {
        @Indirect public var test: String

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
        use facet::Facet;

        #[derive(Facet)]
        #[facet(namespace = "other")]
        pub struct Child {
            value: i32,
        }
    }

    #[derive(Facet)]
    struct Parent {
        child: Child,
        other_child: other::Child,
    }

    let registry = reflect!(Parent).unwrap();
    let mut modules: Vec<_> = module::split("root", &registry).into_iter().collect();
    modules.sort_by(|a, b| a.0.config().module_name.cmp(&b.0.config().module_name));

    let modules: [(Module, Registry); 2] = modules.try_into().expect("Two modules expected");
    let [(other_module, other_registry), (root_module, root_registry)] = modules;

    let actual = emit_swift(&other_module, &other_registry);
    insta::assert_snapshot!(actual, @"
    import Serde

    public struct Child: Hashable {
        @Indirect public var value: Int32

        public init(value: Int32) {
            self.value = value
        }
    }
    ");

    let actual = emit_swift(&root_module, &root_registry);
    insta::assert_snapshot!(actual, @"
    import Other
    import Serde

    public struct Child: Hashable {
        @Indirect public var value: String

        public init(value: String) {
            self.value = value
        }
    }

    public struct Parent: Hashable {
        @Indirect public var child: Child
        @Indirect public var otherChild: Other.Child

        public init(child: Child, otherChild: Other.Child) {
            self.child = child
            self.otherChild = otherChild
        }
    }
    ");
}

fn emit_swift(module: &Module, registry: &Registry) -> String {
    let mut out = Vec::new();
    let generator = CodeGenerator::new(module.config());
    generator.output(&mut out, registry).unwrap();
    String::from_utf8(out).unwrap()
}
