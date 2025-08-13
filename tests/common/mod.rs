#![allow(clippy::must_use_candidate, clippy::missing_panics_doc)]
use facet::Facet;
use facet_generate::{Registry, generation::Encoding, reflect};
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Simple data formats used to create and test values in each language.
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
    reflect!(Test)
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
    // The following types are not supported by our bincode and BCS runtimes, therefore
    // we don't populate them for testing.
    f_f32: Option<f32>,
    f_f64: Option<f64>,
    f_char: Option<char>,
}

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
#[allow(clippy::struct_field_names)]
pub struct OtherTypes {
    f_string: String,
    #[facet(bytes)]
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

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnitStruct;

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewTypeStruct(u64);

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct TupleStruct(u32, u64);

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
    reflect!(SerdeData)
}

/// Manually generate sample values.
/// Avoid maps with more than one element when `has_canonical_maps` is false so that
/// we can test re-serialization.
#[allow(clippy::too_many_lines)]
pub fn get_sample_values(has_canonical_maps: bool, has_floats: bool) -> Vec<SerdeData> {
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
        f_f32: if has_floats { Some(0.4) } else { None },
        f_f64: if has_floats { Some(35.21) } else { None },
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
        f_f32: if has_floats { Some(-4111.0) } else { None },
        f_f64: if has_floats { Some(-0.0021) } else { None },
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
        f_stringmap: if has_canonical_maps {
            btreemap! {"foo".to_string() => 1, "bar".to_string() => 2}
        } else {
            btreemap! {"foo".to_string() => 1}
        },
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
        f_intset: if has_canonical_maps {
            btreemap! {1 => (), 5 => (), 16 => (), 64 => (), 257 => (), 1024 => ()}
        } else {
            btreemap! {64 => ()}
        },
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
        f_intset: if has_canonical_maps {
            std::iter::repeat_n((), 10)
                .enumerate()
                .map(|(i, ())| (i as u64, ()))
                .collect()
        } else {
            #[allow(clippy::zero_sized_map_values)]
            BTreeMap::new()
        },
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

/// Structure used to factorize code in runtime tests.
#[derive(Copy, Clone)]
pub enum Runtime {
    Bcs,
    Bincode,
}

impl std::convert::From<Runtime> for Encoding {
    fn from(runtime: Runtime) -> Self {
        match runtime {
            Runtime::Bcs => Encoding::Bcs,
            Runtime::Bincode => Encoding::Bincode,
        }
    }
}

impl Runtime {
    pub fn name(self) -> &'static str {
        <Self as std::convert::Into<Encoding>>::into(self).name()
    }

    pub fn rust_package(self) -> &'static str {
        match self {
            Self::Bcs => "bcs = \"0.1.1\"",
            Self::Bincode => "bincode = \"1.3\"",
        }
    }

    pub fn serialize<T>(self, value: &T) -> Vec<u8>
    where
        T: serde::Serialize,
    {
        match self {
            Self::Bcs => bcs::to_bytes(value).unwrap(),
            Self::Bincode => bincode::serialize(value).unwrap(),
        }
    }

    pub fn deserialize<T>(self, bytes: &[u8]) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        match self {
            Self::Bcs => bcs::from_bytes(bytes).ok(),
            Self::Bincode => bincode::deserialize(bytes).ok(),
        }
    }

    /// Serialize a value then add noise to the serialized bits repeatedly. Additionally return
    /// `true` if the deserialization of each modified bitstring should succeed.
    pub fn serialize_with_noise_and_deserialize<T>(self, value: &T) -> Vec<(Vec<u8>, bool)>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let mut results = Vec::new();
        let s = self.serialize(value);
        results.push((s.clone(), true));

        if let Runtime::Bincode = self {
            // Unfortunately, the current Rust implementation of bincode does not take fuzzing of
            // `Vec<()>` values well at all.
            return results;
        }

        // For each byte position < 9 in the serialization of `value`:
        for i in 0..std::cmp::min(s.len(), 9) {
            // Flip the highest bit
            {
                let mut s2 = s.clone();
                s2[i] ^= 0x80;
                let is_valid = self.deserialize::<T>(&s2).is_some();
                results.push((s2, is_valid));
            }

            // See if we can turn an (apparent) 4-byte UTF-8 codepoint into an invalid
            // 5-byte codepoint.
            if (i + 4 < s.len())
                && (s[i] ^ 0xf0 < 0x08)
                && (s[i + 1] ^ 0x80 < 0x40)
                && (s[i + 2] ^ 0x80 < 0x40)
                && (s[i + 3] ^ 0x80 < 0x40)
                && (s[i + 4] < 0x40)
            {
                let mut s2 = s.clone();
                s2[i] ^= 0x08;
                s2[i + 4] ^= 0x80;
                let is_valid = self.deserialize::<T>(&s2).is_some();
                results.push((s2, is_valid));
            }
        }
        results
    }

    pub fn quote_serialize(self) -> &'static str {
        match self {
            Self::Bcs => "bcs::to_bytes",
            Self::Bincode => "bincode::serialize",
        }
    }

    pub fn quote_deserialize(self) -> &'static str {
        match self {
            Self::Bcs => "bcs::from_bytes",
            Self::Bincode => "bincode::deserialize",
        }
    }

    /// Whether the encoding enforces ordering of map keys.
    /// Note that both encodings are canonical on other data structures.
    pub fn has_canonical_maps(self) -> bool {
        match self {
            Self::Bcs => true,
            Self::Bincode => false,
        }
    }

    /// Whether the encoding supports float32 and float64.
    pub fn has_floats(self) -> bool {
        match self {
            Self::Bcs => false,
            Self::Bincode => true,
        }
    }

    pub fn maximum_length(self) -> Option<usize> {
        match self {
            Self::Bcs => Some(bcs::MAX_SEQUENCE_LENGTH),
            Self::Bincode => None,
        }
    }

    pub fn maximum_container_depth(self) -> Option<usize> {
        match self {
            Self::Bcs => Some(bcs::MAX_CONTAINER_DEPTH),
            Self::Bincode => None,
        }
    }

    pub fn get_positive_samples_quick(self) -> Vec<Vec<u8>> {
        let values = get_sample_values(self.has_canonical_maps(), self.has_floats());
        let mut positive_samples = Vec::new();
        for value in values {
            for (sample, result) in self.serialize_with_noise_and_deserialize(&value) {
                if result {
                    positive_samples.push(sample);
                }
            }
        }
        if let Some(depth) = self.maximum_container_depth() {
            positive_samples.push(
                self.get_sample_with_container_depth(depth)
                    .expect("depth should be large enough"),
            );
            positive_samples.push(
                self.get_alternate_sample_with_container_depth(depth)
                    .expect("depth should be large enough"),
            );
        }
        positive_samples
    }

    pub fn get_positive_samples(self) -> Vec<Vec<u8>> {
        let mut positive_samples = self.get_positive_samples_quick();
        if let Some(length) = self.maximum_length() {
            positive_samples.push(self.get_sample_with_long_sequence(length));
        }
        positive_samples
    }

    pub fn get_negative_samples(self) -> Vec<Vec<u8>> {
        let values = get_sample_values(self.has_canonical_maps(), self.has_floats());
        let mut negative_samples = Vec::new();
        for value in values {
            for (sample, result) in self.serialize_with_noise_and_deserialize(&value) {
                if !result {
                    negative_samples.push(sample);
                }
            }
        }
        if let Some(length) = self.maximum_length() {
            negative_samples.push(self.get_sample_with_long_sequence(length + 1));
        }
        if let Some(depth) = self.maximum_container_depth() {
            negative_samples.push(self.get_sample_with_container_depth(depth + 1).unwrap());
            negative_samples.push(
                self.get_alternate_sample_with_container_depth(depth + 1)
                    .unwrap(),
            );
        }
        if let Self::Bcs = self {
            negative_samples.push(vec![0x09, 0x00, 0x00]);
            negative_samples.push(vec![0x09, 0x80, 0x00]);
            negative_samples.push(vec![0x09, 0xff, 0xff, 0xff, 0xff, 0x10]);
            negative_samples.push(vec![0x09, 0xff, 0xff, 0xff, 0xff, 0x08]);
        }
        negative_samples
    }

    // Used to test limits on "container depth".
    // Here we construct the serialized bytes directly to allow examples outside the limit.
    pub fn get_sample_with_container_depth(self, depth: usize) -> Option<Vec<u8>> {
        if depth < 2 {
            return None;
        }
        let mut e = self.serialize::<List<SerdeData>>(&List::Empty);

        let f0 = self.serialize(&List::Node(
            Box::new(SerdeData::UnitVariant),
            Box::new(List::Empty),
        ));
        let f = f0[..f0.len() - e.len()].to_vec();

        let h0 = self.serialize(&SerdeData::ListWithMutualRecursion(List::Empty));
        let mut result = h0[..h0.len() - e.len()].to_vec();

        for _ in 2..depth {
            result.append(&mut f.clone());
        }
        result.append(&mut e);
        Some(result)
    }

    // Used to test limits on "container depth".
    // Here we construct the serialized bytes directly to allow examples outside the limit.
    pub fn get_alternate_sample_with_container_depth(self, depth: usize) -> Option<Vec<u8>> {
        if depth < 2 {
            return None;
        }
        let mut e = self.serialize::<SimpleList>(&SimpleList(None));

        let f0 = self.serialize(&SimpleList(Some(Box::new(SimpleList(None)))));
        let f = f0[..f0.len() - e.len()].to_vec();

        let h0 = self.serialize(&SerdeData::SimpleList(SimpleList(None)));
        let mut result = h0[..h0.len() - e.len()].to_vec();

        for _ in 2..depth {
            result.append(&mut f.clone());
        }
        result.append(&mut e);
        Some(result)
    }

    // Used to test limits on sequence lengths and container depth.
    // Here we construct the serialized bytes directly to allow examples outside the limit.
    pub fn get_sample_with_long_sequence(self, length: usize) -> Vec<u8> {
        let e = self.serialize::<Vec<()>>(&Vec::new());
        let f0 = self.serialize(&SerdeData::UnitVector(Vec::new()));
        let mut result = f0[..f0.len() - e.len()].to_vec();
        match self {
            Runtime::Bincode => result.append(&mut self.serialize(&(length as u64))),
            Runtime::Bcs => {
                // ULEB-128 encoding of the length.
                let mut value = length;
                while value >= 0x80 {
                    #[allow(clippy::cast_possible_truncation)]
                    let byte = (value & 0x7f) as u8;
                    result.push(byte | 0x80);
                    value >>= 7;
                }
                #[allow(clippy::cast_possible_truncation)]
                result.push(value as u8);
            }
        }
        result
    }
}
