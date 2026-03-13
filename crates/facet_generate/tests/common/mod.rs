#![allow(clippy::must_use_candidate, clippy::missing_panics_doc)]

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
    ListWithMutualRecursion(List<Box<SerdeData>>),
    TreeWithMutualRecursion(Tree<Box<SerdeData>>),
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
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
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
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnitStruct;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewTypeStruct(u64);

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct TupleStruct(u32, u64);

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct Struct {
    x: u32,
    y: u64,
}

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
#[repr(C)]
pub enum List<T> {
    Empty,
    Node(T, Box<List<T>>),
}

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tree<T> {
    value: T,
    children: Vec<Tree<T>>,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimpleList(pub Option<Box<SimpleList>>);

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
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
