#![cfg(any(
    feature = "java",
    feature = "kotlin",
    feature = "swift",
    feature = "typescript"
))]

use crate::common::{
    Runtime, SerdeData, get_alternate_sample_value_with_container_depth, get_registry,
    get_sample_value_with_container_depth, get_sample_value_with_long_sequence, get_sample_values,
    get_simple_registry,
};

pub mod common;

#[test]
fn test_get_sample_values() {
    // assert_eq!(get_sample_values(false, true).len(), 18);
    assert_eq!(get_sample_values(false, true).len(), 17);
}

#[test]
fn test_get_simple_registry() {
    let registry = get_simple_registry();
    insta::assert_yaml_snapshot!(&registry, @r"
    ? namespace: ROOT
      name: Choice
    : ENUM:
        - 0:
            A:
              - UNIT
              - []
          1:
            B:
              - NEWTYPE: U64
              - []
          2:
            C:
              - STRUCT:
                  - x:
                      - U8
                      - []
              - []
        - []
    ? namespace: ROOT
      name: Test
    : STRUCT:
        - - a:
              - SEQ: U32
              - []
          - b:
              - TUPLE:
                  - I64
                  - U64
              - []
          - c:
              - TYPENAME:
                  namespace: ROOT
                  name: Choice
              - []
        - []
    ");
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_get_registry() {
    let registry = get_registry();
    insta::assert_yaml_snapshot!(&registry, @r"
    ? namespace: ROOT
      name: CStyleEnum
    : ENUM:
        - 0:
            A:
              - UNIT
              - []
          1:
            B:
              - UNIT
              - []
          2:
            C:
              - UNIT
              - []
          3:
            D:
              - UNIT
              - []
          4:
            E:
              - UNIT
              - []
        - []
    ? namespace: ROOT
      name: List
    : ENUM:
        - 0:
            Empty:
              - UNIT
              - []
          1:
            Node:
              - TUPLE:
                  - TYPENAME:
                      namespace: ROOT
                      name: SerdeData
                  - TYPENAME:
                      namespace: ROOT
                      name: List
              - []
        - []
    ? namespace: ROOT
      name: NewTypeStruct
    : NEWTYPESTRUCT:
        - U64
        - []
    ? namespace: ROOT
      name: OtherTypes
    : STRUCT:
        - - f_string:
              - STR
              - []
          - f_bytes:
              - BYTES
              - []
          - f_option:
              - OPTION:
                  TYPENAME:
                    namespace: ROOT
                    name: Struct
              - []
          - f_unit:
              - UNIT
              - []
          - f_seq:
              - SEQ:
                  TYPENAME:
                    namespace: ROOT
                    name: Struct
              - []
          - f_opt_seq:
              - OPTION:
                  SEQ: I32
              - []
          - f_tuple:
              - TUPLE:
                  - U8
                  - U16
              - []
          - f_stringmap:
              - MAP:
                  KEY: STR
                  VALUE: U32
              - []
          - f_intset:
              - MAP:
                  KEY: U64
                  VALUE: UNIT
              - []
          - f_nested_seq:
              - SEQ:
                  SEQ:
                    TYPENAME:
                      namespace: ROOT
                      name: Struct
              - []
        - []
    ? namespace: ROOT
      name: PrimitiveTypes
    : STRUCT:
        - - f_bool:
              - BOOL
              - []
          - f_u8:
              - U8
              - []
          - f_u16:
              - U16
              - []
          - f_u32:
              - U32
              - []
          - f_u64:
              - U64
              - []
          - f_u128:
              - U128
              - []
          - f_i8:
              - I8
              - []
          - f_i16:
              - I16
              - []
          - f_i32:
              - I32
              - []
          - f_i64:
              - I64
              - []
          - f_i128:
              - I128
              - []
          - f_f32:
              - OPTION: F32
              - []
          - f_f64:
              - OPTION: F64
              - []
          - f_char:
              - OPTION: CHAR
              - []
        - []
    ? namespace: ROOT
      name: SerdeData
    : ENUM:
        - 0:
            PrimitiveTypes:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: PrimitiveTypes
              - []
          1:
            OtherTypes:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: OtherTypes
              - []
          2:
            UnitVariant:
              - UNIT
              - []
          3:
            NewTypeVariant:
              - NEWTYPE: STR
              - []
          4:
            TupleVariant:
              - TUPLE:
                  - U32
                  - U64
              - []
          5:
            StructVariant:
              - STRUCT:
                  - f0:
                      - TYPENAME:
                          namespace: ROOT
                          name: UnitStruct
                      - []
                  - f1:
                      - TYPENAME:
                          namespace: ROOT
                          name: NewTypeStruct
                      - []
                  - f2:
                      - TYPENAME:
                          namespace: ROOT
                          name: TupleStruct
                      - []
                  - f3:
                      - TYPENAME:
                          namespace: ROOT
                          name: Struct
                      - []
              - []
          6:
            ListWithMutualRecursion:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: List
              - []
          7:
            TreeWithMutualRecursion:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: Tree
              - []
          8:
            TupleArray:
              - NEWTYPE:
                  TUPLEARRAY:
                    CONTENT: U32
                    SIZE: 3
              - []
          9:
            UnitVector:
              - NEWTYPE:
                  SEQ: UNIT
              - []
          10:
            SimpleList:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: SimpleList
              - []
          11:
            CStyleEnum:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: CStyleEnum
              - []
          12:
            ComplexMap:
              - NEWTYPE:
                  MAP:
                    KEY:
                      TUPLE:
                        - TUPLEARRAY:
                            CONTENT: U32
                            SIZE: 2
                        - TUPLEARRAY:
                            CONTENT: U8
                            SIZE: 4
                    VALUE: UNIT
              - []
          13:
            EmptyStructVariant:
              - UNIT
              - []
        - []
    ? namespace: ROOT
      name: SimpleList
    : NEWTYPESTRUCT:
        - OPTION:
            TYPENAME:
              namespace: ROOT
              name: SimpleList
        - []
    ? namespace: ROOT
      name: Struct
    : STRUCT:
        - - x:
              - U32
              - []
          - y:
              - U64
              - []
        - []
    ? namespace: ROOT
      name: Tree
    : STRUCT:
        - - value:
              - TYPENAME:
                  namespace: ROOT
                  name: SerdeData
              - []
          - children:
              - SEQ:
                  TYPENAME:
                    namespace: ROOT
                    name: Tree
              - []
        - []
    ? namespace: ROOT
      name: TupleStruct
    : TUPLESTRUCT:
        - - U32
          - U64
        - []
    ? namespace: ROOT
      name: UnitStruct
    : UNITSTRUCT: []
    ");
}

#[test]
fn test_bincode_get_sample_with_long_sequence() {
    test_get_sample_with_long_sequence(Runtime::Bincode);
}

#[test]
fn test_bcs_get_sample_with_long_sequence() {
    test_get_sample_with_long_sequence(Runtime::Bcs);
}

// Make sure the direct computation of the serialization of these test values
// agrees with the usual serialization.
fn test_get_sample_with_long_sequence(runtime: Runtime) {
    let value = get_sample_value_with_long_sequence(0);
    assert_eq!(
        runtime.serialize(&value),
        runtime.get_sample_with_long_sequence(0)
    );

    let value = get_sample_value_with_long_sequence(20);
    assert_eq!(
        runtime.serialize(&value),
        runtime.get_sample_with_long_sequence(20)
    );

    let value = get_sample_value_with_long_sequence(200);
    assert_eq!(
        runtime.serialize(&value),
        runtime.get_sample_with_long_sequence(200)
    );
}

#[test]
fn test_bincode_samples_with_container_depth() {
    test_get_sample_with_container_depth(Runtime::Bincode);
    test_get_alternate_sample_with_container_depth(Runtime::Bincode);
}

#[test]
fn test_bcs_samples_with_container_depth() {
    test_get_sample_with_container_depth(Runtime::Bcs);
    test_get_alternate_sample_with_container_depth(Runtime::Bcs);
}

// Make sure the direct computation of the serialization of these test values
// agrees with the usual serialization.
fn test_get_sample_with_container_depth(runtime: Runtime) {
    let value = get_sample_value_with_container_depth(2).unwrap();
    assert_eq!(
        runtime.serialize(&value),
        runtime.get_sample_with_container_depth(2).unwrap()
    );

    let value = get_sample_value_with_container_depth(20).unwrap();
    assert_eq!(
        runtime.serialize(&value),
        runtime.get_sample_with_container_depth(20).unwrap()
    );

    let value = get_sample_value_with_container_depth(200).unwrap();
    assert_eq!(
        runtime.serialize(&value),
        runtime.get_sample_with_container_depth(200).unwrap()
    );
}

// Make sure the direct computation of the serialization of these test values
// agrees with the usual serialization.
fn test_get_alternate_sample_with_container_depth(runtime: Runtime) {
    let value = get_alternate_sample_value_with_container_depth(2).unwrap();
    assert_eq!(
        runtime.serialize(&value),
        runtime
            .get_alternate_sample_with_container_depth(2)
            .unwrap()
    );

    let value = get_alternate_sample_value_with_container_depth(20).unwrap();
    assert_eq!(
        runtime.serialize(&value),
        runtime
            .get_alternate_sample_with_container_depth(20)
            .unwrap()
    );

    let value = get_alternate_sample_value_with_container_depth(200).unwrap();
    assert_eq!(
        runtime.serialize(&value),
        runtime
            .get_alternate_sample_with_container_depth(200)
            .unwrap()
    );
}

#[test]
fn test_bincode_get_positive_samples() {
    // assert_eq!(test_get_positive_samples(Runtime::Bincode), 18);
    assert_eq!(test_get_positive_samples(Runtime::Bincode), 17);
}

#[test]
// This test requires --release because of deserialization of long (unit) vectors.
#[cfg(not(debug_assertions))]
fn test_bcs_get_positive_samples() {
    assert_eq!(test_get_positive_samples(Runtime::Bcs), 82);
}

// Make sure all the "positive" samples successfully deserialize with the reference Rust
// implementation.
fn test_get_positive_samples(runtime: Runtime) -> usize {
    let samples = runtime.get_positive_samples();
    let length = samples.len();
    for sample in samples {
        assert!(runtime.deserialize::<SerdeData>(&sample).is_some());
    }
    length
}

#[test]
fn test_bincode_get_negative_samples() {
    assert_eq!(test_get_negative_samples(Runtime::Bincode), 0);
}

#[test]
// This test requires --release because of deserialization of long (unit) vectors.
#[cfg(not(debug_assertions))]
fn test_bcs_get_negative_samples() {
    assert_eq!(test_get_negative_samples(Runtime::Bcs), 59);
}

// Make sure all the "negative" samples fail to deserialize with the reference Rust
// implementation.
fn test_get_negative_samples(runtime: Runtime) -> usize {
    let samples = runtime.get_negative_samples();
    let length = samples.len();
    for sample in samples {
        assert!(runtime.deserialize::<SerdeData>(&sample).is_none());
    }
    length
}

#[test]
fn test_bcs_serialize_with_noise_and_deserialize() {
    let value = "\u{10348}.".to_string();
    let samples = Runtime::Bcs.serialize_with_noise_and_deserialize(&value);
    // 1 for original encoding
    // 1 for each byte in the serialization (value.len() + 1)
    // 1 for added incorrect 5-byte UTF8-like codepoint
    assert_eq!(samples.len(), value.len() + 3);
}
