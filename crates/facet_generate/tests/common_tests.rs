#![cfg(any(
    feature = "csharp",
    feature = "kotlin",
    feature = "swift",
    feature = "typescript"
))]

use crate::common::{
    get_alternate_sample_value_with_container_depth, get_alternate_sample_with_container_depth,
    get_positive_samples, get_registry, get_sample_value_with_container_depth,
    get_sample_value_with_long_sequence, get_sample_values, get_sample_with_container_depth,
    get_sample_with_long_sequence, get_simple_registry,
};

pub mod common;

#[test]
fn test_get_sample_values() {
    // assert_eq!(get_sample_values(false, true).len(), 18);
    assert_eq!(get_sample_values().len(), 17);
}

#[test]
fn test_get_simple_registry() {
    let registry = get_simple_registry();
    insta::assert_yaml_snapshot!(&registry, @"
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
    insta::assert_yaml_snapshot!(&registry, @"
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
    let value = get_sample_value_with_long_sequence(0);
    assert_eq!(
        bincode::serialize(&value).unwrap(),
        get_sample_with_long_sequence(0)
    );

    let value = get_sample_value_with_long_sequence(20);
    assert_eq!(
        bincode::serialize(&value).unwrap(),
        get_sample_with_long_sequence(20)
    );

    let value = get_sample_value_with_long_sequence(200);
    assert_eq!(
        bincode::serialize(&value).unwrap(),
        get_sample_with_long_sequence(200)
    );
}

#[test]
fn test_bincode_samples_with_container_depth() {
    let value = get_sample_value_with_container_depth(2).unwrap();
    assert_eq!(
        bincode::serialize(&value).unwrap(),
        get_sample_with_container_depth(2).unwrap()
    );

    let value = get_sample_value_with_container_depth(20).unwrap();
    assert_eq!(
        bincode::serialize(&value).unwrap(),
        get_sample_with_container_depth(20).unwrap()
    );

    let value = get_sample_value_with_container_depth(200).unwrap();
    assert_eq!(
        bincode::serialize(&value).unwrap(),
        get_sample_with_container_depth(200).unwrap()
    );

    let value = get_alternate_sample_value_with_container_depth(2).unwrap();
    assert_eq!(
        bincode::serialize(&value).unwrap(),
        get_alternate_sample_with_container_depth(2).unwrap()
    );

    let value = get_alternate_sample_value_with_container_depth(20).unwrap();
    assert_eq!(
        bincode::serialize(&value).unwrap(),
        get_alternate_sample_with_container_depth(20).unwrap()
    );

    let value = get_alternate_sample_value_with_container_depth(200).unwrap();
    assert_eq!(
        bincode::serialize(&value).unwrap(),
        get_alternate_sample_with_container_depth(200).unwrap()
    );
}

#[test]
fn test_bincode_get_positive_samples() {
    let samples = get_positive_samples();
    // assert_eq!(samples.len(), 18);
    assert_eq!(samples.len(), 17);
    for sample in samples {
        assert!(bincode::deserialize::<common::SerdeData>(&sample).is_ok());
    }
}
