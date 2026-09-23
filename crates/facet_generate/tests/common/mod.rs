#![allow(
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::unsafe_derive_deserialize
)]

use std::collections::BTreeMap;

use facet::Facet;
use maplit::btreemap;
use serde::{Deserialize, Serialize};

use facet_generate as fg;
use facet_generate::{Registry, reflect};

// Simple data formats used to create and test values in each language.
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Serialize, Deserialize)]
pub struct Test {
    pub a: Vec<u32>,
    pub b: (i64, u64),
    pub c: Choice,
}

#[derive(Facet, Serialize, Deserialize)]
#[repr(C)]
pub enum Choice {
    A,
    B(u64),
    C { x: u8 },
}

pub fn get_simple_registry() -> Registry {
    reflect!(Test).unwrap()
}

// More complex data format used to test re-serialization and basic fuzzing.
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub enum SerdeData {
    PrimitiveTypes(PrimitiveTypes),
    OtherTypes(OtherTypes),
    UnitVariant,
    NewTypeVariant(String),
    TupleVariant(u32, u64),
    StructVariant {
        f0: UnitStruct,
        f1: NewTypeStruct,
        f2: TupleStruct,
        f3: Struct,
    },
    ListWithMutualRecursion(List<Box<Self>>),
    TreeWithMutualRecursion(Tree<Box<Self>>),
    TupleArray([u32; 3]),
    UnitVector(Vec<()>),
    SimpleList(SimpleList),
    CStyleEnum(CStyleEnum),
    #[allow(clippy::zero_sized_map_values)]
    ComplexMap(BTreeMap<([u32; 2], [u8; 4]), ()>),
    // TODO: Facet has a problem with empty tuple variants
    // EmptyTupleVariant(),
    EmptyStructVariant {},
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
#[allow(clippy::struct_field_names)]
pub struct PrimitiveTypes {
    f_bool: bool,
    f_u8: u8,
    f_u16: u16,
    f_u32: u32,
    f_u64: u64,
    f_u128: u128,
    f_i8: i8,
    f_i16: i16,
    f_i32: i32,
    f_i64: i64,
    f_i128: i128,
    // The following types are not supported by our bincode runtime, therefore
    // we don't populate them for testing.
    f_f32: Option<f32>,
    f_f64: Option<f64>,
    f_char: Option<char>,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct OtherTypes {
    f_string: String,
    #[facet(fg::bytes)]
    f_bytes: Vec<u8>,
    f_option: Option<Struct>,
    f_unit: (),
    f_seq: Vec<Struct>,
    f_opt_seq: Option<Vec<i32>>,
    f_tuple: (u8, u16),
    f_stringmap: BTreeMap<String, u32>,
    #[allow(clippy::zero_sized_map_values)]
    f_intset: BTreeMap<u64, ()>, // Avoiding BTreeSet because Serde treats them as sequences.
    f_nested_seq: Vec<Vec<Struct>>,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnitStruct;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewTypeStruct(u64);

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TupleStruct(u32, u64);

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Struct {
    x: u32,
    y: u64,
}

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub enum List<T> {
    Empty,
    Node(T, Box<Self>),
}

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tree<T> {
    value: T,
    children: Vec<Self>,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimpleList(pub Option<Box<Self>>);

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(C)]
pub enum CStyleEnum {
    A,
    B,
    C,
    D,
    E = 10,
}

/// The registry corresponding to the test data structures above.
pub fn get_registry() -> Registry {
    reflect!(SerdeData).unwrap()
}

/// Manually generate sample values.
#[allow(clippy::too_many_lines)]
pub fn get_sample_values() -> Vec<SerdeData> {
    let v0 = SerdeData::PrimitiveTypes(PrimitiveTypes {
        f_bool: false,
        f_u8: 6,
        f_u16: 5,
        f_u32: 4,
        f_u64: 3,
        f_u128: 2,
        f_i8: 1,
        f_i16: 0,
        f_i32: -1,
        f_i64: -2,
        f_i128: -3,
        f_f32: Some(0.4),
        f_f64: Some(35.21),
        f_char: None,
    });

    let v1 = SerdeData::PrimitiveTypes(PrimitiveTypes {
        f_bool: true,
        f_u8: u8::MAX,
        f_u16: u16::MAX,
        f_u32: u32::MAX,
        f_u64: u64::MAX,
        f_u128: u128::MAX,
        f_i8: i8::MIN,
        f_i16: i16::MIN,
        f_i32: i32::MIN,
        f_i64: i64::MIN,
        f_i128: i128::MIN,
        f_f32: Some(-4111.0),
        f_f64: Some(-0.0021),
        f_char: None,
    });

    let v2 = SerdeData::OtherTypes(OtherTypes {
        f_string: "test".to_string(),
        f_bytes: b"bytes".to_vec(),
        f_option: Some(Struct { x: 2, y: 3 }),
        f_unit: (),
        f_seq: vec![Struct { x: 1, y: 3 }],
        f_opt_seq: Some(vec![1]),
        f_tuple: (4, 5),
        f_stringmap: btreemap! {"foo".to_string() => 1},
        #[allow(clippy::zero_sized_map_values)]
        f_intset: BTreeMap::new(),
        f_nested_seq: vec![
            vec![Struct { x: 4, y: 5 }, Struct { x: 6, y: 7 }],
            vec![Struct { x: 8, y: 9 }],
        ],
    });

    let v2bis = SerdeData::OtherTypes(OtherTypes {
        f_string: String::new(),
        f_bytes: b"".to_vec(),
        f_option: None,
        f_unit: (),
        f_seq: Vec::new(),
        f_opt_seq: None,
        f_tuple: (4, 5),
        f_stringmap: BTreeMap::new(),
        f_intset: btreemap! {64 => ()},
        f_nested_seq: vec![],
    });

    let v2ter = SerdeData::OtherTypes(OtherTypes {
        f_string: String::new(),
        f_bytes: vec![1u8; 129],
        f_option: None,
        f_unit: (),
        f_seq: Vec::new(),
        f_opt_seq: None,
        f_tuple: (4, 5),
        f_stringmap: BTreeMap::new(),
        #[allow(clippy::zero_sized_map_values)]
        f_intset: BTreeMap::new(),
        f_nested_seq: vec![],
    });

    let v3 = SerdeData::UnitVariant;

    let v4 =
        SerdeData::NewTypeVariant("test.\u{10348}.\u{00a2}\u{0939}\u{20ac}\u{d55c}..".to_string());

    let v5 = SerdeData::TupleVariant(3, 6);

    let v6 = SerdeData::StructVariant {
        f0: UnitStruct,
        f1: NewTypeStruct(1),
        f2: TupleStruct(2, 3),
        f3: Struct { x: 4, y: 5 },
    };

    let v7 = SerdeData::ListWithMutualRecursion(List::Empty);

    let v8 = SerdeData::TreeWithMutualRecursion(Tree {
        value: Box::new(SerdeData::PrimitiveTypes(PrimitiveTypes {
            f_bool: false,
            f_u8: 0,
            f_u16: 1,
            f_u32: 2,
            f_u64: 3,
            f_u128: 4,
            f_i8: 5,
            f_i16: 6,
            f_i32: 7,
            f_i64: 8,
            f_i128: 9,
            f_f32: None,
            f_f64: None,
            f_char: None,
        })),
        children: vec![Tree {
            value: Box::new(SerdeData::PrimitiveTypes(PrimitiveTypes {
                f_bool: false,
                f_u8: 0,
                f_u16: 0,
                f_u32: 0,
                f_u64: 0,
                f_u128: 0,
                f_i8: 0,
                f_i16: 0,
                f_i32: 0,
                f_i64: 0,
                f_i128: 0,
                f_f32: None,
                f_f64: None,
                f_char: None,
            })),
            children: vec![],
        }],
    });

    let v9 = SerdeData::TupleArray([0, 2, 3]);

    let v10 = SerdeData::UnitVector(vec![(); 1000]);

    let v11 = SerdeData::SimpleList(SimpleList(Some(Box::new(SimpleList(None)))));

    let v12 = SerdeData::CStyleEnum(CStyleEnum::C);

    let v13 = SerdeData::ComplexMap(btreemap! { ([1,2], [3,4,5,6]) => ()});

    // let v14 = SerdeData::EmptyTupleVariant();
    let v15 = SerdeData::EmptyStructVariant {};

    vec![
        v0, v1, v2, v2bis, v2ter, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, //v14,
        v15,
    ]
}

#[cfg(test)]
// Used to test limits on "container depth".
#[must_use]
pub fn get_sample_value_with_container_depth(depth: usize) -> Option<SerdeData> {
    if depth < 2 {
        return None;
    }
    let mut list = List::Empty;
    for _ in 2..depth {
        list = List::Node(Box::new(SerdeData::UnitVariant), Box::new(list));
    }
    Some(SerdeData::ListWithMutualRecursion(list))
}

#[cfg(test)]
// Used to test limits on "container depth".
#[must_use]
pub fn get_alternate_sample_value_with_container_depth(depth: usize) -> Option<SerdeData> {
    if depth < 2 {
        return None;
    }
    let mut list = SimpleList(None);
    for _ in 2..depth {
        list = SimpleList(Some(Box::new(list)));
    }
    Some(SerdeData::SimpleList(list))
}

#[cfg(test)]
// Used to test limits on sequence lengths and container depth.
#[must_use]
pub fn get_sample_value_with_long_sequence(length: usize) -> SerdeData {
    SerdeData::UnitVector(vec![(); length])
}

/// Serialize each sample value with bincode, returning the serialized bytes.
pub fn get_positive_samples() -> Vec<Vec<u8>> {
    get_sample_values()
        .iter()
        .map(|v| bincode::serialize(v).unwrap())
        .collect()
}

/// Construct serialized bytes for a list with the given container depth.
/// Bytes are constructed directly to allow testing depths outside normal limits.
pub fn get_sample_with_container_depth(depth: usize) -> Option<Vec<u8>> {
    if depth < 2 {
        return None;
    }
    let mut e = bincode::serialize::<List<SerdeData>>(&List::Empty).unwrap();

    let f0 = bincode::serialize(&List::Node(
        Box::new(SerdeData::UnitVariant),
        Box::new(List::Empty),
    ))
    .unwrap();
    let f = f0[..f0.len() - e.len()].to_vec();

    let h0 = bincode::serialize(&SerdeData::ListWithMutualRecursion(List::Empty)).unwrap();
    let mut result = h0[..h0.len() - e.len()].to_vec();

    for _ in 2..depth {
        result.append(&mut f.clone());
    }
    result.append(&mut e);
    Some(result)
}

/// Construct serialized bytes for a `SimpleList` with the given container depth.
/// Bytes are constructed directly to allow testing depths outside normal limits.
pub fn get_alternate_sample_with_container_depth(depth: usize) -> Option<Vec<u8>> {
    if depth < 2 {
        return None;
    }
    let mut e = bincode::serialize::<SimpleList>(&SimpleList(None)).unwrap();

    let f0 = bincode::serialize(&SimpleList(Some(Box::new(SimpleList(None))))).unwrap();
    let f = f0[..f0.len() - e.len()].to_vec();

    let h0 = bincode::serialize(&SerdeData::SimpleList(SimpleList(None))).unwrap();
    let mut result = h0[..h0.len() - e.len()].to_vec();

    for _ in 2..depth {
        result.append(&mut f.clone());
    }
    result.append(&mut e);
    Some(result)
}

/// Construct serialized bytes for a `UnitVector` with the given length.
/// Bytes are constructed directly to allow testing lengths outside normal limits.
pub fn get_sample_with_long_sequence(length: usize) -> Vec<u8> {
    let e = bincode::serialize::<Vec<()>>(&Vec::new()).unwrap();
    let f0 = bincode::serialize(&SerdeData::UnitVector(Vec::new())).unwrap();
    let mut result = f0[..f0.len() - e.len()].to_vec();
    result.append(&mut bincode::serialize(&(length as u64)).unwrap());
    result
}

// ---------------------------------------------------------------------------
// Swift-compatible test fixture
//
// `SerdeData` includes `ComplexMap(BTreeMap<([u32; 2], [u8; 4]), ()>)` whose
// key is a native tuple — valid in Rust but not representable as a Swift
// `Dictionary` key (native tuples do not conform to `Hashable`).
//
// `SwiftSerdeData` covers the same surface area without that variant.
// `SwiftSimpleList` is a recursive struct; it compiles in Swift because the
// emitter generates `@Indirect` for the recursive field.
// ---------------------------------------------------------------------------

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
#[repr(C)]
#[allow(dead_code)]
pub enum SwiftSerdeData {
    PrimitiveTypes(SwiftPrimitiveTypes),
    OtherTypes(SwiftOtherTypes),
    UnitVariant,
    NewTypeVariant(String),
    TupleVariant(u32, u64),
    StructVariant {
        f0: SwiftUnitStruct,
        f1: SwiftNewTypeStruct,
        f2: SwiftTupleStruct,
        f3: SwiftStruct,
    },
    ListWithMutualRecursion(SwiftList<Box<Self>>),
    TreeWithMutualRecursion(Tree<Box<Self>>),
    TupleArray([u32; 3]),
    UnitVector(Vec<()>),
    SimpleList(SwiftSimpleList),
    EmptyStructVariant {},
}

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
#[allow(clippy::struct_field_names)]
pub struct SwiftPrimitiveTypes {
    pub f_bool: bool,
    pub f_u8: u8,
    pub f_u16: u16,
    pub f_u32: u32,
    pub f_u64: u64,
    pub f_u128: u128,
    pub f_i8: i8,
    pub f_i16: i16,
    pub f_i32: i32,
    pub f_i64: i64,
    pub f_i128: i128,
    pub f_f32: Option<f32>,
    pub f_f64: Option<f64>,
    pub f_char: Option<char>,
}

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct SwiftOtherTypes {
    pub f_string: String,
    #[facet(fg::bytes)]
    pub f_bytes: Vec<u8>,
    pub f_option: Option<SwiftStruct>,
    pub f_unit: (),
    pub f_seq: Vec<SwiftStruct>,
    pub f_opt_seq: Option<Vec<i32>>,
    pub f_tuple: (u8, u16),
    pub f_stringmap: BTreeMap<String, u32>,
    pub f_nested_seq: Vec<Vec<SwiftStruct>>,
}

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftUnitStruct;

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftNewTypeStruct(pub u64);

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftTupleStruct(pub u32, pub u64);

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftStruct {
    pub x: u32,
    pub y: u64,
}

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(C)]
#[allow(dead_code)]
pub enum SwiftList<T> {
    Empty,
    Node(T, Box<Self>),
}

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftSimpleList(pub Option<Box<Self>>);

// ---------------------------------------------------------------------------
// UUID round-trip fixtures
// ---------------------------------------------------------------------------

/// Two well-known RFC 4122 UUIDs used as pinned test vectors.
/// Using `uuid!` gives a compile-time constant so the bytes are deterministic.
pub const UUID_ID: uuid::Uuid = uuid::uuid!("550e8400-e29b-41d4-a716-446655440000");
pub const UUID_PARENT_ID: uuid::Uuid = uuid::uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8");

/// A struct with a required and an optional UUID field.
#[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UuidData {
    pub id: uuid::Uuid,
    pub parent_id: Option<uuid::Uuid>,
}

/// Returns a registry containing only [`UuidData`].
pub fn get_uuid_registry() -> Registry {
    reflect!(UuidData).unwrap()
}

/// Serialize the canonical UUID test value with bincode.
/// The resulting bytes are used as the reference wire encoding in all
/// language round-trip tests.
pub fn get_uuid_reference_bytes() -> Vec<u8> {
    bincode::serialize(&UuidData {
        id: UUID_ID,
        parent_id: Some(UUID_PARENT_ID),
    })
    .unwrap()
}

/// Registry used for Swift compilation and runtime tests.
///
/// Excludes `ComplexMap(BTreeMap<([u32; 2], [u8; 4]), ()>)` because native
/// Swift tuples are not `Hashable` and cannot be used as `Dictionary` keys.
/// Includes `SwiftSimpleList` which the Swift emitter handles with selective
/// `@Indirect` on the recursive field.
pub fn get_swift_registry() -> Registry {
    reflect!(SwiftSerdeData).unwrap()
}

/// Sample values for [`SwiftSerdeData`], serialised with bincode.
#[allow(clippy::too_many_lines)]
pub fn get_swift_positive_samples() -> Vec<Vec<u8>> {
    let values: Vec<SwiftSerdeData> = vec![
        SwiftSerdeData::UnitVariant,
        SwiftSerdeData::NewTypeVariant("test.\u{10348}".to_string()),
        SwiftSerdeData::TupleVariant(3, 6),
        SwiftSerdeData::PrimitiveTypes(SwiftPrimitiveTypes {
            f_bool: false,
            f_u8: 6,
            f_u16: 5,
            f_u32: 4,
            f_u64: 3,
            f_u128: 2,
            f_i8: 1,
            f_i16: 0,
            f_i32: -1,
            f_i64: -2,
            f_i128: -3,
            f_f32: Some(0.4),
            f_f64: Some(35.21),
            f_char: None,
        }),
        SwiftSerdeData::OtherTypes(SwiftOtherTypes {
            f_string: "test".to_string(),
            f_bytes: b"bytes".to_vec(),
            f_option: Some(SwiftStruct { x: 2, y: 3 }),
            f_unit: (),
            f_seq: vec![SwiftStruct { x: 1, y: 3 }],
            f_opt_seq: Some(vec![1]),
            f_tuple: (4, 5),
            f_stringmap: {
                let mut m = BTreeMap::new();
                m.insert("foo".to_string(), 1u32);
                m
            },
            f_nested_seq: vec![vec![SwiftStruct { x: 4, y: 5 }]],
        }),
        SwiftSerdeData::OtherTypes(SwiftOtherTypes {
            f_string: String::new(),
            f_bytes: b"".to_vec(),
            f_option: None,
            f_unit: (),
            f_seq: Vec::new(),
            f_opt_seq: None,
            f_tuple: (0, 0),
            f_stringmap: BTreeMap::new(),
            f_nested_seq: vec![],
        }),
        SwiftSerdeData::StructVariant {
            f0: SwiftUnitStruct,
            f1: SwiftNewTypeStruct(1),
            f2: SwiftTupleStruct(2, 3),
            f3: SwiftStruct { x: 4, y: 5 },
        },
        SwiftSerdeData::TupleArray([0, 2, 3]),
        SwiftSerdeData::UnitVector(vec![(); 3]),
        SwiftSerdeData::SimpleList(SwiftSimpleList(Some(Box::new(SwiftSimpleList(None))))),
        SwiftSerdeData::EmptyStructVariant {},
    ];
    values
        .iter()
        .map(|v| bincode::serialize(v).unwrap())
        .collect()
}

// ---------------------------------------------------------------------------
// Keyword fixture — shared by the per-language compilation tests.
//
// Every field and variant name here collides with a keyword in at least one
// target language, except `import` and `type`, which are soft or contextual
// keywords everywhere and must come through untouched.
// ---------------------------------------------------------------------------

#[derive(Facet)]
#[allow(clippy::struct_excessive_bools)]
pub struct KeywordFields {
    pub r#default: String,
    pub r#in: i32,
    pub class: bool,
    pub object: String,
    pub r#static: bool,
    pub r#let: String,
    pub when: i32,
    pub is: bool,
    pub fun: String,
    pub operator: String,
    pub import: String,
    pub r#type: String,
    pub function: Option<String>,
    /// A tuple field: the Swift plugin derives `whereField0` / `whereField1`
    /// locals from this name, which must stay unescaped.
    pub r#where: (i32, String),
}

#[derive(Facet)]
pub struct KeywordTuple(pub String, pub i32);

#[derive(Facet)]
pub struct KeywordNewType(pub String);

#[derive(Facet)]
#[repr(C)]
#[allow(dead_code)]
pub enum KeywordEnum {
    Default,
    Case,
    Switch(String),
    Where { r#in: i32, r#default: String },
}

/// Registry of the keyword fixture types, used by the per-language
/// compilation tests.
pub fn get_keyword_registry() -> Registry {
    reflect!(KeywordFields, KeywordTuple, KeywordNewType, KeywordEnum).unwrap()
}

// ---------------------------------------------------------------------------
// Builtin-shadowing fixture — shared by the per-language compilation tests.
//
// `Set` is declared as a top-level struct, so every `Set<T>` the generated
// module writes must be qualified (`kotlin.collections.Set`, `Swift.Set`, …)
// while the declaration itself keeps its name.
// ---------------------------------------------------------------------------

#[derive(Facet)]
pub struct Get {
    pub key: String,
}

#[derive(Facet)]
pub struct Set {
    pub key: String,
    #[facet(fg::bytes)]
    pub value: Vec<u8>,
}

#[derive(Facet)]
pub struct Delete {
    pub key: String,
}

#[derive(Facet)]
pub struct Exists {
    pub key: String,
}

#[derive(Facet)]
pub struct ListKeys {
    pub prefix: String,
    pub cursor: u64,
}

#[derive(Facet)]
pub struct Keys {
    // Named `items` rather than `keys`: a C# property may not share its name
    // with its enclosing type (CS0542).
    pub items: Vec<String>,
    pub next_cursor: u64,
}

// `Value::Bytes` becomes a nested class in Kotlin and C#, beside a module that
// imports `Bytes` for `Set.value`: the nested class only shadows the import
// inside `Value`, where nothing uses it.
#[derive(Facet)]
#[repr(C)]
#[allow(dead_code)]
pub enum Value {
    None,
    Bytes(Vec<u8>),
}

#[derive(Facet)]
#[repr(C)]
#[allow(dead_code)]
pub enum ValueResult {
    Ok(Value),
    Err(String),
}

#[derive(Facet)]
#[repr(C)]
#[allow(dead_code)]
pub enum BoolResult {
    Ok(bool),
    Err(String),
}

#[derive(Facet)]
#[repr(C)]
#[allow(dead_code)]
pub enum KeysResult {
    Ok(Keys),
    Err(String),
}

#[derive(Facet)]
pub struct Store {
    pub tags: std::collections::HashSet<String>,
    pub entries: BTreeMap<String, String>,
    pub blob: Vec<u8>,
    pub pair: (i32, String),
}

/// Registry of the builtin-shadowing fixture types, used by the per-language
/// compilation tests.
pub fn get_shadowing_registry() -> Registry {
    reflect!(
        Get,
        Set,
        Delete,
        Exists,
        ListKeys,
        Keys,
        Value,
        ValueResult,
        BoolResult,
        KeysResult,
        Store
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Cross-namespace fixtures — shared by the per-language compilation tests.
//
// Types that reference types in another namespace, where the generated code
// depends on what kind of type the referenced one is (an enum is serialized
// differently from a struct) or on which names it brings into scope.
//
// A namespaced type referencing a ROOT one (`across_namespaces::to_root`) is
// compiled in every language. Swift compiles it without the ROOT type holding
// the namespaced one (`to_root::get_namespace_registry`): its targets cannot
// depend on each other, so it rejects `to_root::get_registry`.
// ---------------------------------------------------------------------------

pub mod across_namespaces {
    use std::collections::BTreeSet;

    use facet::Facet;
    use facet_generate as fg;
    use facet_generate::{Registry, reflect};

    pub mod kit {
        use std::collections::BTreeSet;

        use facet::Facet;
        use facet_generate as fg;

        #[derive(Facet)]
        #[repr(C)]
        #[facet(fg::namespace = "kit")]
        #[allow(dead_code)]
        pub enum Presence {
            Online,
            Offline,
        }

        #[derive(Facet)]
        #[repr(C)]
        #[facet(fg::namespace = "kit")]
        #[allow(dead_code)]
        pub enum Shape {
            Circle(f64),
            Empty,
        }

        #[derive(Facet)]
        #[facet(fg::namespace = "kit")]
        pub struct Badge {
            pub presence: Presence,
            pub shape: Shape,
        }

        /// Shadows `Swift.Set` in every module that imports `Kit`.
        #[derive(Facet)]
        #[facet(fg::namespace = "kit")]
        pub struct Set {
            pub value: u32,
        }

        #[derive(Facet)]
        #[facet(fg::namespace = "kit")]
        pub struct Tray {
            pub nothing: (),
            pub ids: BTreeSet<u32>,
        }
    }

    /// Shares its name with the `kit` enum, and is still a class.
    #[derive(Facet)]
    pub struct Presence {
        pub since: u64,
    }

    /// Holds the ROOT `Presence`, beside [`Card`] holding the `kit` one.
    ///
    /// Not a field of `Card`: C# would resolve `Presence.Deserialize` inside
    /// `Card` to its `Presence` property (#159).
    #[derive(Facet)]
    pub struct Sighting {
        pub last_seen: Presence,
    }

    /// Shadows the C# runtime's `Unit` in the namespaces nested in the root
    /// one.
    #[derive(Facet)]
    pub struct Unit {
        pub value: u32,
    }

    /// A ROOT type holding enums from `kit`, beside a `kit` type holding them.
    #[derive(Facet)]
    pub struct Card {
        pub presence: kit::Presence,
        pub shape: kit::Shape,
        pub shapes: Vec<Option<kit::Shape>>,
        pub badge: kit::Badge,
    }

    /// Builtin names declared in the other module: `kit::Set` for Swift, and
    /// the ROOT `Unit` for C# (which `kit::Tray` has to spell out).
    ///
    /// `kit::Tray` is not a field: its `()` makes it not `Hashable` in Swift,
    /// and Swift decides a ROOT type's conformance as if every type from
    /// another module were `Hashable` (#156).
    #[derive(Facet)]
    pub struct Shelf {
        pub set: kit::Set,
        pub ids: BTreeSet<u32>,
        pub unit: Unit,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[facet(fg::namespace = "b")]
    #[allow(dead_code)]
    pub enum Status {
        Up,
        Down,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[facet(fg::namespace = "b")]
    #[allow(dead_code)]
    pub enum Signal {
        Level(u8),
        Silent,
    }

    /// A type in namespace `a` holding enums from namespace `b`.
    #[derive(Facet)]
    #[facet(fg::namespace = "a")]
    pub struct Row {
        pub status: Status,
        pub signal: Signal,
    }

    /// References from ROOT into `kit`, and from `kit` into `kit`.
    pub fn get_registry() -> Registry {
        use kit::Tray;
        reflect!(Card, Sighting, Shelf, Tray).unwrap()
    }

    /// Gives the root module a type, which a Swift package needs for its root
    /// target to have sources.
    #[derive(Facet)]
    pub struct Table {
        pub row: Row,
    }

    /// References from namespace `a` into namespace `b`.
    pub fn get_sibling_registry() -> Registry {
        reflect!(Table).unwrap()
    }

    /// A type in namespace `kv` holding types pinned to ROOT, held in turn by
    /// a ROOT type, so the root module and `kv` reference each other.
    pub mod to_root {
        use facet::Facet;
        use facet_generate as fg;
        use facet_generate::{Registry, reflect};
        use serde::{Deserialize, Serialize};

        pub mod kv {
            use std::collections::BTreeMap;

            use facet::Facet;
            use facet_generate as fg;
            use serde::{Deserialize, Serialize};

            /// Shares its name with the ROOT enum, and is still a class.
            #[derive(Facet, Serialize, Deserialize)]
            #[facet(fg::namespace = "kv")]
            pub struct Presence {
                pub since: u64,
            }

            #[derive(Facet, Serialize, Deserialize)]
            #[facet(fg::namespace = "kv")]
            pub struct Entry {
                pub shared: super::Shared,
                pub level: super::Level,
                pub outcome: super::Outcome,
                /// Not called `presence`: C# would resolve the local
                /// `Presence.Deserialize` inside `Entry` to that property (#159).
                pub status: super::Presence,
                pub local: Presence,
            }

            /// ROOT types nested in generics, which keep their pin to ROOT.
            #[derive(Facet)]
            #[facet(fg::namespace = "kv")]
            pub struct Batch {
                pub many: Vec<Option<super::Shared>>,
                pub levels: BTreeMap<String, super::Level>,
                pub outcome: Option<super::Outcome>,
                pub statuses: Option<Vec<super::Presence>>,
            }
        }

        #[derive(Facet, Serialize, Deserialize)]
        #[facet(fg::namespace)]
        pub struct Shared {
            pub id: u32,
        }

        #[derive(Facet, Serialize, Deserialize)]
        #[repr(C)]
        #[facet(fg::namespace)]
        #[allow(dead_code)]
        pub enum Level {
            Low,
            High,
        }

        #[derive(Facet, Serialize, Deserialize)]
        #[repr(C)]
        #[facet(fg::namespace)]
        #[allow(dead_code)]
        pub enum Outcome {
            Score(u32),
            Missing,
        }

        /// Shares its name with the `kv` struct.
        #[derive(Facet, Serialize, Deserialize)]
        #[repr(C)]
        #[facet(fg::namespace)]
        #[allow(dead_code)]
        pub enum Presence {
            Online,
            Offline,
        }

        #[derive(Facet, Serialize, Deserialize)]
        pub struct App {
            pub entry: kv::Entry,
            pub shared: Shared,
        }

        /// References from ROOT into `kv`, and from `kv` back into ROOT.
        pub fn get_registry() -> Registry {
            use kv::Batch;
            reflect!(App, Batch).unwrap()
        }

        /// References from `kv` into ROOT only.
        pub fn get_namespace_registry() -> Registry {
            use kv::{Batch, Entry};
            reflect!(Entry, Batch).unwrap()
        }
    }

    /// Types with no namespace attribute, reached only from a type in
    /// `detail`, so they are generated in `detail` too, and every reference
    /// to them, however it is wrapped, is to `detail` (#167). A ROOT type holds
    /// the `detail` one, so a reference from `detail` to ROOT would make each
    /// module depend on the other.
    pub mod inherited {
        use std::collections::BTreeMap;

        use facet::Facet;
        use facet_generate as fg;
        use facet_generate::{Registry, reflect};

        #[derive(Facet)]
        pub struct Neighbour {
            pub id: u32,
        }

        #[derive(Facet)]
        #[repr(C)]
        #[allow(dead_code)]
        pub enum Visit {
            Planned(Option<Neighbour>),
            Done,
        }

        #[derive(Facet)]
        pub struct Neighbourhood {
            pub manager: Option<Neighbour>,
            pub reports: Vec<Neighbour>,
            pub by_name: BTreeMap<String, Neighbour>,
            pub deputies: Option<Vec<Neighbour>>,
            pub visits: Vec<Visit>,
        }

        #[derive(Facet)]
        #[facet(fg::namespace = "detail")]
        pub struct ViewModel {
            pub neighbourhood: Neighbourhood,
        }

        #[derive(Facet)]
        pub struct App {
            pub detail: ViewModel,
        }

        pub fn get_registry() -> Registry {
            reflect!(App).unwrap()
        }
    }
}
