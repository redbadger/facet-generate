use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, Utc};

use super::*;
use crate::reflect;

#[test]
fn unit_struct() {
    #[derive(Facet)]
    struct MyUnitStruct;

    insta::assert_yaml_snapshot!(reflect!(MyUnitStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyUnitStruct
    : UNITSTRUCT: []
    ");
}

#[test]
fn unit_struct_with_doc() {
    /// This is a unit struct with a doc comment.
    /// And this is the second line of the doc comment.
    #[derive(Facet)]
    struct MyUnitStruct;

    insta::assert_yaml_snapshot!(reflect!(MyUnitStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyUnitStruct
    : UNITSTRUCT:
        - This is a unit struct with a doc comment.
        - And this is the second line of the doc comment.
    ");
}

#[test]
fn newtype_bool() {
    #[derive(Facet)]
    struct MyNewType(bool);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - BOOL
        - []
    ");
}

#[test]
fn newtype_unit() {
    #[derive(Facet)]
    struct MyNewType(());

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - UNIT
        - []
    ");
}

#[test]
fn newtype_static_str() {
    #[derive(Facet)]
    struct MyNewType(&'static str);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - STR
        - []
    ");
}

#[test]
fn newtype_u8() {
    #[derive(Facet)]
    struct MyNewType(u8);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - U8
        - []
    ");
}

#[test]
fn newtype_u16() {
    #[derive(Facet)]
    struct MyNewType(u16);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - U16
        - []
    ");
}

#[test]
fn newtype_u32() {
    #[derive(Facet)]
    struct MyNewType(u32);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - U32
        - []
    ");
}

#[test]
fn newtype_u64() {
    #[derive(Facet)]
    struct MyNewType(u64);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - U64
        - []
    ");
}

#[test]
fn newtype_u128() {
    #[derive(Facet)]
    struct MyNewType(u128);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - U128
        - []
    ");
}

#[test]
fn newtype_i8() {
    #[derive(Facet)]
    struct MyNewType(i8);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - I8
        - []
    ");
}

#[test]
fn newtype_i16() {
    #[derive(Facet)]
    struct MyNewType(i16);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - I16
        - []
    ");
}

#[test]
fn newtype_i32() {
    #[derive(Facet)]
    struct MyNewType(i32);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - I32
        - []
    ");
}

#[test]
fn newtype_i64() {
    #[derive(Facet)]
    struct MyNewType(i64);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - I64
        - []
    ");
}

#[test]
fn newtype_i128() {
    #[derive(Facet)]
    struct MyNewType(i128);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - I128
        - []
    ");
}

#[test]
fn newtype_f32() {
    #[derive(Facet)]
    struct MyNewType(f32);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - F32
        - []
    ");
}

#[test]
fn newtype_f64() {
    #[derive(Facet)]
    struct MyNewType(f64);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - F64
        - []
    ");
}

#[test]
fn newtype_char() {
    #[derive(Facet)]
    struct MyNewType(char);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - CHAR
        - []
    ");
}

#[test]
fn nested_newtype() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyNewType(Inner);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - TYPENAME:
            namespace: ROOT
            name: Inner
        - []
    ");
}

#[test]
fn newtype_with_list() {
    #[derive(Facet)]
    struct MyNewType(Vec<i32>);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - SEQ: I32
        - []
    ");
}

#[test]
fn newtype_with_list_of_named_type() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyNewType(Vec<Inner>);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - SEQ:
            TYPENAME:
              namespace: ROOT
              name: Inner
        - []
    ");
}

#[test]
fn newtype_with_nested_list() {
    #[derive(Facet)]
    struct MyNewType(Vec<Vec<i32>>);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - SEQ:
            SEQ: I32
        - []
    ");
}

#[test]
fn newtype_with_nested_list_of_named_type() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyNewType(Vec<Vec<Inner>>);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - SEQ:
            SEQ:
              TYPENAME:
                namespace: ROOT
                name: Inner
        - []
    ");
}

#[test]
fn newtype_with_triple_nested_list_of_named_type() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyNewType(Vec<Vec<Vec<Inner>>>);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - SEQ:
            SEQ:
              SEQ:
                TYPENAME:
                  namespace: ROOT
                  name: Inner
        - []
    ");
}

#[test]
fn newtype_with_tuple_array() {
    #[derive(Facet)]
    struct MyNewType([i32; 3]);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - TUPLEARRAY:
            CONTENT: I32
            SIZE: 3
        - []
    ");
}

#[test]
fn tuple_struct() {
    #[derive(Facet)]
    struct MyTupleStruct(u8, i32, bool);

    insta::assert_yaml_snapshot!(reflect!(MyTupleStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyTupleStruct
    : TUPLESTRUCT:
        - - U8
          - I32
          - BOOL
        - []
    ");
}

#[test]
fn tuple_struct_with_unit() {
    #[derive(Facet)]
    struct MyTupleStruct(u8, (), i32);

    insta::assert_yaml_snapshot!(reflect!(MyTupleStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyTupleStruct
    : TUPLESTRUCT:
        - - U8
          - UNIT
          - I32
        - []
    ");
}

#[test]
fn option_of_unit() {
    #[derive(Facet)]
    struct MyStruct {
        a: Option<()>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - OPTION: UNIT
              - []
        - []
    ");
}

#[test]
fn option_of_list() {
    #[derive(Facet)]
    struct MyStruct {
        a: Option<Vec<i32>>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - OPTION:
                  SEQ: I32
              - []
        - []
    ");
}

#[test]
fn option_of_nested_list() {
    #[derive(Facet)]
    struct MyStruct {
        a: Option<Vec<Vec<i32>>>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - OPTION:
                  SEQ:
                    SEQ: I32
              - []
        - []
    ");
}

#[test]
fn option_of_list_of_named_type() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyStruct {
        a: Option<Vec<Inner>>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - OPTION:
                  SEQ:
                    TYPENAME:
                      namespace: ROOT
                      name: Inner
              - []
        - []
    ");
}

#[test]
fn list_of_options() {
    #[derive(Facet)]
    struct MyStruct {
        a: Vec<Option<i32>>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - SEQ:
                  OPTION: I32
              - []
        - []
    ");
}

#[test]
fn list_of_options_of_named_type() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyStruct {
        a: Vec<Option<Inner>>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - SEQ:
                  OPTION:
                    TYPENAME:
                      namespace: ROOT
                      name: Inner
              - []
        - []
    ");
}

#[test]
fn nested_list_with_options() {
    #[derive(Facet)]
    struct MyStruct {
        a: Vec<Vec<Option<i32>>>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - SEQ:
                  SEQ:
                    OPTION: I32
              - []
        - []
    ");
}

#[test]
fn nested_tuple_struct_1() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyTupleStruct(Inner, u8);

    insta::assert_yaml_snapshot!(reflect!(MyTupleStruct).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyTupleStruct
    : TUPLESTRUCT:
        - - TYPENAME:
              namespace: ROOT
              name: Inner
          - U8
        - []
    ");
}

#[test]
fn nested_tuple_struct_2() {
    #[derive(Facet)]
    struct Inner(i32, u8, bool);

    #[derive(Facet)]
    struct MyTupleStruct(i32, Inner, u8);

    insta::assert_yaml_snapshot!(reflect!(MyTupleStruct).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : TUPLESTRUCT:
        - - I32
          - U8
          - BOOL
        - []
    ? namespace: ROOT
      name: MyTupleStruct
    : TUPLESTRUCT:
        - - I32
          - TYPENAME:
              namespace: ROOT
              name: Inner
          - U8
        - []
    ");
}

#[test]
fn struct_with_doc() {
    /// This is a doc comment
    #[derive(Facet)]
    /// and another doc comment
    struct MyStruct {
        a: u8,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - U8
              - []
        - - This is a doc comment
          - and another doc comment
    ");
}

#[test]
fn struct_with_field_doc() {
    #[derive(Facet)]
    struct MyStruct {
        /// This is a doc comment
        a: u8,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - U8
              - - This is a doc comment
        - []
    ");
}

#[test]
fn struct_with_static_str() {
    #[derive(Facet)]
    struct MyStruct {
        a: &'static str,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - STR
              - []
        - []
    ");
}

#[test]
fn struct_with_vec_of_u8() {
    #[derive(Facet)]
    struct MyStruct {
        a: Vec<u8>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - SEQ: U8
              - []
        - []
    ");
}

#[test]
fn struct_with_vec_of_u8_to_bytes() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(bytes)]
        a: Vec<u8>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - BYTES
              - []
        - []
    ");
}

#[test]
fn struct_with_slice_of_u8() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        a: &'a [u8],
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - SEQ: U8
              - []
        - []
    ");
}

#[test]
fn struct_with_slice_of_u8_to_bytes() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        #[facet(bytes)]
        a: &'a [u8],
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - BYTES
              - []
        - []
    ");
}

#[test]
fn struct_with_scalar_fields() {
    #[derive(Facet)]
    struct MyStruct {
        a: u8,
        b: i32,
        c: bool,
        d: (),
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - U8
              - []
          - b:
              - I32
              - []
          - c:
              - BOOL
              - []
          - d:
              - UNIT
              - []
        - []
    ");
}

#[test]
fn struct_with_tuple_field() {
    #[derive(Facet)]
    struct MyStruct {
        a: (u8, i32),
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - TUPLE:
                  - U8
                  - I32
              - []
        - []
    ");
}

#[test]
fn struct_with_option_fields() {
    #[derive(Facet)]
    struct Inner {
        a: Option<bool>,
        b: Option<u8>,
    }

    #[derive(Facet)]
    struct MyStruct {
        a: Option<Inner>,
        b: Option<u8>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : STRUCT:
        - - a:
              - OPTION: BOOL
              - []
          - b:
              - OPTION: U8
              - []
        - []
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - OPTION:
                  TYPENAME:
                    namespace: ROOT
                    name: Inner
              - []
          - b:
              - OPTION: U8
              - []
        - []
    ");
}

#[test]
fn struct_with_fields_of_newtypes_and_tuple_structs() {
    #[derive(Facet)]
    struct Inner1(i32);

    #[derive(Facet)]
    struct Inner2(i8, u32);

    #[derive(Facet)]
    struct MyStruct {
        a: Inner1,
        b: Inner2,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: Inner1
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: Inner2
    : TUPLESTRUCT:
        - - I8
          - U32
        - []
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - TYPENAME:
                  namespace: ROOT
                  name: Inner1
              - []
          - b:
              - TYPENAME:
                  namespace: ROOT
                  name: Inner2
              - []
        - []
    ");
}

#[test]
fn enum_with_unit_variants() {
    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum MyEnum {
        Variant1,
        Variant2,
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Variant1:
              - UNIT
              - []
          1:
            Variant2:
              - UNIT
              - []
        - []
    ");
}

#[test]
fn enum_with_newtype_variants() {
    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum MyEnum {
        Variant1(u8),
        Variant2(i32),
        Variant3(f64),
        Variant4(char),
        Variant5(String),
        Variant6(bool),
        Variant7(()),
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Variant1:
              - NEWTYPE: U8
              - []
          1:
            Variant2:
              - NEWTYPE: I32
              - []
          2:
            Variant3:
              - NEWTYPE: F64
              - []
          3:
            Variant4:
              - NEWTYPE: CHAR
              - []
          4:
            Variant5:
              - NEWTYPE: STR
              - []
          5:
            Variant6:
              - NEWTYPE: BOOL
              - []
          6:
            Variant7:
              - NEWTYPE: UNIT
              - []
        - []
    ");
}

#[test]
fn enum_with_newtype_variants_containing_user_defined_types() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum MyEnum {
        Variant1(Inner),
        Variant2(u8),
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Variant1:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: Inner
              - []
          1:
            Variant2:
              - NEWTYPE: U8
              - []
        - []
    ");
}

#[test]
fn enum_with_tuple_struct_variants() {
    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum MyEnum {
        Variant1(u8, i32, f64, char, String, bool, ()),
        Variant2(i8, u32),
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Variant1:
              - TUPLE:
                  - U8
                  - I32
                  - F64
                  - CHAR
                  - STR
                  - BOOL
                  - UNIT
              - []
          1:
            Variant2:
              - TUPLE:
                  - I8
                  - U32
              - []
        - []
    ");
}

#[test]
fn enum_with_tuple_variants_containing_user_defined_types() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum MyEnum {
        Variant1(Inner, u8),
        Variant2(i8, Inner),
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Variant1:
              - TUPLE:
                  - TYPENAME:
                      namespace: ROOT
                      name: Inner
                  - U8
              - []
          1:
            Variant2:
              - TUPLE:
                  - I8
                  - TYPENAME:
                      namespace: ROOT
                      name: Inner
              - []
        - []
    ");
}

#[test]
fn enum_with_inline_struct_variants() {
    #[derive(Facet)]
    struct Inner {
        a: String,
    }
    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum MyEnum {
        Variant1 { a: Inner, b: u8, c: bool },
        Variant2(Inner),
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : STRUCT:
        - - a:
              - STR
              - []
        - []
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Variant1:
              - STRUCT:
                  - a:
                      - TYPENAME:
                          namespace: ROOT
                          name: Inner
                      - []
                  - b:
                      - U8
                      - []
                  - c:
                      - BOOL
                      - []
              - []
          1:
            Variant2:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: Inner
              - []
        - []
    ");
}

#[test]
fn enum_with_struct_variants_mixed_types() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum MyEnum {
        Variant1 {
            a: Inner,
            b: u8,
            c: f32,
            d: String,
            e: char,
            f: bool,
            g: (),
        },
        Variant2 {
            x: i32,
            y: Inner,
        },
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : NEWTYPESTRUCT:
        - I32
        - []
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Variant1:
              - STRUCT:
                  - a:
                      - TYPENAME:
                          namespace: ROOT
                          name: Inner
                      - []
                  - b:
                      - U8
                      - []
                  - c:
                      - F32
                      - []
                  - d:
                      - STR
                      - []
                  - e:
                      - CHAR
                      - []
                  - f:
                      - BOOL
                      - []
                  - g:
                      - UNIT
                      - []
              - []
          1:
            Variant2:
              - STRUCT:
                  - x:
                      - I32
                      - []
                  - y:
                      - TYPENAME:
                          namespace: ROOT
                          name: Inner
                      - []
              - []
        - []
    ");
}

#[test]
fn enum_with_struct_variant() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    pub enum MyEnum {
        A,
        B(u64),
        C { x: u8 },
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: MyEnum
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
    ");
}

#[test]
fn struct_with_skip_serializing() {
    #[derive(Facet)]
    struct MyStruct {
        a: u8,
        #[facet(skip)]
        b: u8,
        c: u8,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - U8
              - []
          - c:
              - U8
              - []
        - []
    ");
}

#[test]
fn tuple_struct_with_skip_serializing() {
    #[derive(Facet)]
    struct MyStruct(u8, #[facet(skip)] u16, u32);

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : TUPLESTRUCT:
        - - U8
          - U32
        - []
    ");
}

#[test]
fn enum_with_skip_serializing() {
    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum MyEnum {
        Variant1,
        #[facet(skip)]
        Variant2,
        Variant3,
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Variant1:
              - UNIT
              - []
          1:
            Variant3:
              - UNIT
              - []
        - []
    ");
}

#[test]
fn transparent() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyStruct {
        inner: Inner,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - inner:
              - I32
              - []
        - []
    ");
}

#[test]
fn map_of_string_and_bool() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Map(BTreeMap<String, bool>),
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Map:
              - NEWTYPE:
                  MAP:
                    KEY: STR
                    VALUE: BOOL
              - []
        - []
    ");
}

#[test]
fn set_of_string() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Set(BTreeSet<String>),
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Set:
              - NEWTYPE:
                  SET: STR
              - []
        - []
    ");
}

#[test]
fn test_fixed_issues() {
    // Test 1: Set types (previously caused panic due to Def::Set being unhandled)
    #[derive(Facet)]
    struct WithSet {
        set_field: BTreeSet<u32>,
    }

    // Test 2: Nested collections with sets
    #[derive(Facet)]
    #[repr(u8)]
    #[allow(unused)]
    enum EnumWithSet {
        SetVariant(HashSet<String>),
        Regular(u32),
    }

    // Test 3: Complex nested structures that could hit multiple problematic paths
    #[derive(Facet)]
    struct ComplexNested {
        sets: Vec<BTreeSet<String>>,
        optional_set: Option<HashSet<u64>>,
    }

    // This should not panic - all Variable placeholders should be resolved
    let registry = reflect!(WithSet, EnumWithSet, ComplexNested).unwrap();

    // Verify all types were processed
    assert!(!registry.is_empty());

    // The registry should contain our types
    let mut found_types = 0;
    for key in registry.keys() {
        if key.name.contains("WithSet")
            || key.name.contains("EnumWithSet")
            || key.name.contains("ComplexNested")
        {
            found_types += 1;
        }
    }
    assert!(found_types >= 3, "Should have processed all test types");
}

#[test]
fn sequence_and_map_types() {
    #[derive(Facet)]
    struct MyStruct {
        vec: Vec<String>,
        hash_map: HashMap<String, String>,
        hash_set: HashSet<String>,
        btree_map: BTreeMap<String, String>,
        btree_set: BTreeSet<String>,
    }

    insta::assert_yaml_snapshot!(&reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - vec:
              - SEQ: STR
              - []
          - hash_map:
              - MAP:
                  KEY: STR
                  VALUE: STR
              - []
          - hash_set:
              - SET: STR
              - []
          - btree_map:
              - MAP:
                  KEY: STR
                  VALUE: STR
              - []
          - btree_set:
              - SET: STR
              - []
        - []
    ");
}

#[test]
fn map_with_user_defined_types() {
    #[derive(Facet, Ord, PartialOrd, Eq, PartialEq)]
    struct UserId(u32);

    #[derive(Facet)]
    struct UserProfile {
        name: String,
        active: bool,
    }

    #[derive(Facet)]
    struct MyStruct {
        user_map: BTreeMap<UserId, UserProfile>,
        id_to_count: BTreeMap<i32, Vec<String>>,
        nested_map: BTreeMap<String, Option<u64>>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - user_map:
              - MAP:
                  KEY:
                    TYPENAME:
                      namespace: ROOT
                      name: UserId
                  VALUE:
                    TYPENAME:
                      namespace: ROOT
                      name: UserProfile
              - []
          - id_to_count:
              - MAP:
                  KEY: I32
                  VALUE:
                    SEQ: STR
              - []
          - nested_map:
              - MAP:
                  KEY: STR
                  VALUE:
                    OPTION: U64
              - []
        - []
    ? namespace: ROOT
      name: UserId
    : NEWTYPESTRUCT:
        - U32
        - []
    ? namespace: ROOT
      name: UserProfile
    : STRUCT:
        - - name:
              - STR
              - []
          - active:
              - BOOL
              - []
        - []
    ");
}

#[test]
fn complex_map() {
    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum ComplexMap {
        #[allow(clippy::zero_sized_map_values)]
        Map(BTreeMap<([u32; 2], [u8; 4]), ()>),
    }

    insta::assert_yaml_snapshot!(reflect!(ComplexMap).unwrap(), @r"
    ? namespace: ROOT
      name: ComplexMap
    : ENUM:
        - 0:
            Map:
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
        - []
    ");
}

#[test]
fn struct_with_box_of_t() {
    #[derive(Facet)]
    struct UserProfile {
        name: String,
        active: bool,
    }

    #[derive(Facet)]
    struct MyStruct {
        boxed: Box<UserProfile>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - boxed:
              - TYPENAME:
                  namespace: ROOT
                  name: UserProfile
              - []
        - []
    ? namespace: ROOT
      name: UserProfile
    : STRUCT:
        - - name:
              - STR
              - []
          - active:
              - BOOL
              - []
        - []
    ");
}

#[test]
fn struct_with_arc_of_t() {
    #[derive(Facet)]
    struct UserProfile {
        name: String,
        active: bool,
    }

    #[derive(Facet)]
    struct MyStruct {
        boxed: Arc<UserProfile>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - boxed:
              - TYPENAME:
                  namespace: ROOT
                  name: UserProfile
              - []
        - []
    ? namespace: ROOT
      name: UserProfile
    : STRUCT:
        - - name:
              - STR
              - []
          - active:
              - BOOL
              - []
        - []
    ");
}

#[test]
fn own_result_enum() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum HttpResult {
        Ok(HttpResponse),
        Err(HttpError),
    }

    #[derive(Facet)]
    struct HttpResponse {
        status: u16,
        headers: Vec<HttpHeader>,
        #[facet(bytes)]
        body: Vec<u8>,
    }

    #[derive(Facet)]
    struct HttpHeader {
        name: String,
        value: String,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum HttpError {
        #[facet(skip)]
        Http {
            status: u16,
            message: String,
            body: Option<Vec<u8>>,
        },
        #[facet(skip)]
        Json(String),
        Url(String),
        Io(String),
        Timeout,
    }

    insta::assert_yaml_snapshot!(reflect!(HttpResult).unwrap(), @r"
    ? namespace: ROOT
      name: HttpError
    : ENUM:
        - 0:
            Url:
              - NEWTYPE: STR
              - []
          1:
            Io:
              - NEWTYPE: STR
              - []
          2:
            Timeout:
              - UNIT
              - []
        - []
    ? namespace: ROOT
      name: HttpHeader
    : STRUCT:
        - - name:
              - STR
              - []
          - value:
              - STR
              - []
        - []
    ? namespace: ROOT
      name: HttpResponse
    : STRUCT:
        - - status:
              - U16
              - []
          - headers:
              - SEQ:
                  TYPENAME:
                    namespace: ROOT
                    name: HttpHeader
              - []
          - body:
              - BYTES
              - []
        - []
    ? namespace: ROOT
      name: HttpResult
    : ENUM:
        - 0:
            Ok:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: HttpResponse
              - []
          1:
            Err:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: HttpError
              - []
        - []
    ");
}

#[test]
fn struct_rename() {
    #[derive(Facet)]
    #[facet(name = "Effect")]
    struct EffectFfi {
        name: String,
        active: bool,
    }

    insta::assert_yaml_snapshot!(reflect!(EffectFfi).unwrap(), @r"
    ? namespace: ROOT
      name: Effect
    : STRUCT:
        - - name:
              - STR
              - []
          - active:
              - BOOL
              - []
        - []
    ");
}

#[test]
fn enum_rename() {
    #[derive(Facet)]
    #[facet(name = "Effect")]
    #[repr(C)]
    #[allow(unused)]
    enum EffectFfi {
        One,
        Two,
    }

    insta::assert_yaml_snapshot!(reflect!(EffectFfi).unwrap(), @r"
    ? namespace: ROOT
      name: Effect
    : ENUM:
        - 0:
            One:
              - UNIT
              - []
          1:
            Two:
              - UNIT
              - []
        - []
    ");
}

#[test]
fn struct_rename_with_named_type() {
    #[derive(Facet)]
    #[facet(name = "Effect")]
    struct EffectFfi {
        inner: String,
    }

    #[derive(Facet)]
    struct Request {
        id: u32,
        effect: EffectFfi,
    }

    insta::assert_yaml_snapshot!(reflect!(Request).unwrap(), @r"
    ? namespace: ROOT
      name: Effect
    : STRUCT:
        - - inner:
              - STR
              - []
        - []
    ? namespace: ROOT
      name: Request
    : STRUCT:
        - - id:
              - U32
              - []
          - effect:
              - TYPENAME:
                  namespace: ROOT
                  name: Effect
              - []
        - []
    ");
}

#[test]
fn self_referencing_type() {
    #[derive(Facet)]
    struct SimpleList(Option<Box<SimpleList>>);

    insta::assert_debug_snapshot!(reflect!(SimpleList).unwrap(), @r#"
    {
        QualifiedTypeName {
            namespace: Root,
            name: "SimpleList",
        }: NewTypeStruct(
            Option(
                TypeName(
                    QualifiedTypeName {
                        namespace: Root,
                        name: "SimpleList",
                    },
                ),
            ),
            Doc(
                [],
            ),
        ),
    }
    "#);
}

#[test]
fn complex_self_referencing_type() {
    #[derive(Facet)]
    #[allow(clippy::vec_box)]
    struct Node {
        value: i32,
        children: Option<Vec<Box<Node>>>,
    }

    insta::assert_debug_snapshot!(reflect!(Node).unwrap(), @r#"
    {
        QualifiedTypeName {
            namespace: Root,
            name: "Node",
        }: Struct(
            [
                Named {
                    name: "value",
                    doc: Doc(
                        [],
                    ),
                    value: I32,
                },
                Named {
                    name: "children",
                    doc: Doc(
                        [],
                    ),
                    value: Option(
                        Seq(
                            TypeName(
                                QualifiedTypeName {
                                    namespace: Root,
                                    name: "Node",
                                },
                            ),
                        ),
                    ),
                },
            ],
            Doc(
                [],
            ),
        ),
    }
    "#);
}

#[test]
fn tree_struct_with_mutual_recursion() {
    #[derive(Facet)]
    struct Tree<T> {
        value: T,
        children: Vec<Tree<T>>,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Test {
        TreeWithMutualRecursion(Tree<Box<Test>>),
    }

    insta::assert_debug_snapshot!(reflect!(Test).unwrap(), @r#"
    {
        QualifiedTypeName {
            namespace: Root,
            name: "Test",
        }: Enum(
            {
                0: Named {
                    name: "TreeWithMutualRecursion",
                    doc: Doc(
                        [],
                    ),
                    value: NewType(
                        TypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "Tree",
                            },
                        ),
                    ),
                },
            },
            Doc(
                [],
            ),
        ),
        QualifiedTypeName {
            namespace: Root,
            name: "Tree",
        }: Struct(
            [
                Named {
                    name: "value",
                    doc: Doc(
                        [],
                    ),
                    value: TypeName(
                        QualifiedTypeName {
                            namespace: Root,
                            name: "Test",
                        },
                    ),
                },
                Named {
                    name: "children",
                    doc: Doc(
                        [],
                    ),
                    value: Seq(
                        TypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "Tree",
                            },
                        ),
                    ),
                },
            ],
            Doc(
                [],
            ),
        ),
    }
    "#);
}

#[test]
fn tree_enum_with_mutual_recursion() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Tree<T> {
        Value(T),
    }

    #[derive(Facet)]
    struct Test {
        tree_with_mutual_recursion: Tree<Box<Test>>,
    }

    insta::assert_debug_snapshot!(reflect!(Test).unwrap(), @r#"
    {
        QualifiedTypeName {
            namespace: Root,
            name: "Test",
        }: Struct(
            [
                Named {
                    name: "tree_with_mutual_recursion",
                    doc: Doc(
                        [],
                    ),
                    value: TypeName(
                        QualifiedTypeName {
                            namespace: Root,
                            name: "Tree",
                        },
                    ),
                },
            ],
            Doc(
                [],
            ),
        ),
        QualifiedTypeName {
            namespace: Root,
            name: "Tree",
        }: Enum(
            {
                0: Named {
                    name: "Value",
                    doc: Doc(
                        [],
                    ),
                    value: NewType(
                        TypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "Test",
                            },
                        ),
                    ),
                },
            },
            Doc(
                [],
            ),
        ),
    }
    "#);
}

// Reference type tests

#[test]
fn newtype_str_ref() {
    #[derive(Facet)]
    struct MyNewType<'a>(&'a str);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - STR
        - []
    ");
}

#[test]
fn struct_with_str_ref() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        a: &'a str,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - STR
              - []
        - []
    ");
}

#[test]
fn newtype_slice_ref() {
    #[derive(Facet)]
    struct MyNewType<'a>(&'a [u8]);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - SEQ: U8
        - []
    ");
}

#[test]
fn struct_with_slice_ref() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        a: &'a [u8],
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - SEQ: U8
              - []
        - []
    ");
}

#[test]
fn newtype_mut_ref() {
    #[derive(Facet)]
    struct MyNewType<'a>(&'a mut str);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - STR
        - []
    ");
}

#[test]
fn struct_with_mut_ref() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        a: &'a mut [u8],
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - SEQ: U8
              - []
        - []
    ");
}

#[test]
fn newtype_struct_ref() {
    #[derive(Facet)]
    struct Inner {
        value: i32,
    }

    #[derive(Facet)]
    struct MyNewType<'a>(&'a Inner);

    insta::assert_yaml_snapshot!(reflect!(MyNewType).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : STRUCT:
        - - value:
              - I32
              - []
        - []
    ? namespace: ROOT
      name: MyNewType
    : NEWTYPESTRUCT:
        - TYPENAME:
            namespace: ROOT
            name: Inner
        - []
    ");
}

#[test]
fn struct_with_struct_ref() {
    #[derive(Facet)]
    struct Inner {
        value: i32,
    }

    #[derive(Facet)]
    struct MyStruct<'a> {
        a: &'a Inner,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : STRUCT:
        - - value:
              - I32
              - []
        - []
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - TYPENAME:
                  namespace: ROOT
                  name: Inner
              - []
        - []
    ");
}

#[test]
fn enum_with_ref_variants() {
    #[derive(Facet)]
    struct Inner {
        value: i32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused, clippy::enum_variant_names)]
    enum MyEnum<'a> {
        StrRef(&'a str),
        SliceRef(&'a [u8]),
        StructRef(&'a Inner),
    }

    insta::assert_yaml_snapshot!(reflect!(MyEnum).unwrap(), @r"
    ? namespace: ROOT
      name: Inner
    : STRUCT:
        - - value:
              - I32
              - []
        - []
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            StrRef:
              - NEWTYPE: STR
              - []
          1:
            SliceRef:
              - NEWTYPE:
                  SEQ: U8
              - []
          2:
            StructRef:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: Inner
              - []
        - []
    ");
}

#[test]
fn struct_with_vec_ref() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        a: &'a Vec<i32>,
        b: &'a [Vec<String>],
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - SEQ: I32
              - []
          - b:
              - SEQ:
                  SEQ: STR
              - []
        - []
    ");
}

#[test]
fn references_with_options() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        a: Option<&'a str>,
        b: &'a Option<String>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - OPTION: STR
              - []
          - b:
              - OPTION: STR
              - []
        - []
    ");
}

#[test]
fn tuple_struct_with_multiple_refs() {
    #[derive(Facet)]
    struct MyTupleStruct<'a>(&'a str, &'a [u8], &'a i32);

    insta::assert_yaml_snapshot!(reflect!(MyTupleStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyTupleStruct
    : TUPLESTRUCT:
        - - STR
          - SEQ: U8
          - I32
        - []
    ");
}

#[test]
fn struct_with_rc() {
    use std::rc::Rc;
    #[derive(Facet)]
    struct MyStruct {
        a: Rc<String>,
    }

    insta::assert_yaml_snapshot!(reflect!(MyStruct).unwrap(), @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - a:
              - STR
              - []
        - []
    ");
}

#[test]
fn enum_with_a_tuple_variant_that_is_itself_a_tuple() {
    #[derive(Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum MyEnum {
        Variant1((i32, u8)),
    }

    let registry = reflect!(MyEnum).unwrap();
    // TODO: this output is obviously wrong, the `name: (⋯)` is because it's an anonymous tuple struct
    // so what should be name be (if it's a separate type)?
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Variant1:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: (⋯)
              - []
        - []
    ");
}

#[test]
fn enum_with_tuple_variant_of_user_types_in_a_mod() {
    mod api {
        use facet::Facet;

        #[derive(Facet)]
        pub struct Test1;

        #[derive(Facet)]
        pub struct Test2;
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        MyVariant(api::Test1, api::Test2),
    }

    let registry = reflect!(MyEnum).unwrap();
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            MyVariant:
              - TUPLE:
                  - TYPENAME:
                      namespace: ROOT
                      name: Test1
                  - TYPENAME:
                      namespace: ROOT
                      name: Test2
              - []
        - []
    ? namespace: ROOT
      name: Test1
    : UNITSTRUCT: []
    ? namespace: ROOT
      name: Test2
    : UNITSTRUCT: []
    ");
}

#[test]
fn enum_with_tuple_variant_of_lists_of_user_types() {
    use facet::Facet;

    #[derive(Facet)]
    pub struct Test1;

    #[derive(Facet)]
    pub struct Test2;

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        MyVariant(Vec<Test1>, Vec<Test2>),
    }

    let registry = reflect!(MyEnum).unwrap();
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            MyVariant:
              - TUPLE:
                  - SEQ:
                      TYPENAME:
                        namespace: ROOT
                        name: Test1
                  - SEQ:
                      TYPENAME:
                        namespace: ROOT
                        name: Test2
              - []
        - []
    ? namespace: ROOT
      name: Test1
    : UNITSTRUCT: []
    ? namespace: ROOT
      name: Test2
    : UNITSTRUCT: []
    ");
}

#[test]
fn enum_with_tuple_variant_of_option_of_user_types() {
    use facet::Facet;

    #[derive(Facet)]
    pub struct Test1;

    #[derive(Facet)]
    pub struct Test2;

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        MyVariant(Option<Test1>, Option<Test2>),
    }

    let registry = reflect!(MyEnum).unwrap();
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            MyVariant:
              - TUPLE:
                  - OPTION:
                      TYPENAME:
                        namespace: ROOT
                        name: Test1
                  - OPTION:
                      TYPENAME:
                        namespace: ROOT
                        name: Test2
              - []
        - []
    ? namespace: ROOT
      name: Test1
    : UNITSTRUCT: []
    ? namespace: ROOT
      name: Test2
    : UNITSTRUCT: []
    ");
}

#[test]
fn enum_with_tuple_variant_of_map_of_user_types() {
    use facet::Facet;

    #[derive(Facet)]
    pub struct Test1;

    #[derive(Facet)]
    pub struct Test2;

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    #[allow(clippy::zero_sized_map_values)]
    enum MyEnum {
        MyVariant(HashMap<String, Test1>, HashMap<String, Test2>),
    }

    let registry = reflect!(MyEnum).unwrap();
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            MyVariant:
              - TUPLE:
                  - MAP:
                      KEY: STR
                      VALUE:
                        TYPENAME:
                          namespace: ROOT
                          name: Test1
                  - MAP:
                      KEY: STR
                      VALUE:
                        TYPENAME:
                          namespace: ROOT
                          name: Test2
              - []
        - []
    ? namespace: ROOT
      name: Test1
    : UNITSTRUCT: []
    ? namespace: ROOT
      name: Test2
    : UNITSTRUCT: []
    ");
}

#[test]
fn enum_with_tuple_variant_of_set_of_user_types() {
    use facet::Facet;

    #[derive(Facet, PartialEq, Eq, Hash)]
    pub struct Test1;

    #[derive(Facet, PartialEq, Eq, Hash)]
    pub struct Test2;

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        MyVariant(HashSet<Test1>, HashSet<Test2>),
    }

    let registry = reflect!(MyEnum).unwrap();
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            MyVariant:
              - TUPLE:
                  - SET:
                      TYPENAME:
                        namespace: ROOT
                        name: Test1
                  - SET:
                      TYPENAME:
                        namespace: ROOT
                        name: Test2
              - []
        - []
    ? namespace: ROOT
      name: Test1
    : UNITSTRUCT: []
    ? namespace: ROOT
      name: Test2
    : UNITSTRUCT: []
    ");
}

#[test]
fn chrono_date_time() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum MyEnum {
        Date(DateTime<Utc>),
        Date2 { date: DateTime<Utc> },
    }

    #[derive(Facet)]
    struct MyStruct {
        field1: DateTime<Utc>,
        field2: MyEnum,
    }

    let registry = reflect!(MyStruct).unwrap();
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: MyEnum
    : ENUM:
        - 0:
            Date:
              - NEWTYPE: STR
              - []
          1:
            Date2:
              - STRUCT:
                  - date:
                      - STR
                      - []
              - []
        - []
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - field1:
              - STR
              - []
          - field2:
              - TYPENAME:
                  namespace: ROOT
                  name: MyEnum
              - []
        - []
    ");
}

#[test]
fn generics_supported_if_used_once() {
    #[derive(Facet)]
    struct SupportedGenerics<T> {
        field: T,
    }

    #[derive(Facet)]
    struct MyStruct {
        field1: SupportedGenerics<String>,
    }

    let registry = reflect!(MyStruct).unwrap();
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - - field1:
              - TYPENAME:
                  namespace: ROOT
                  name: SupportedGenerics
              - []
        - []
    ? namespace: ROOT
      name: SupportedGenerics
    : STRUCT:
        - - field:
              - STR
              - []
        - []
    ");
}

/// TODO: Eventually we should support generics used more than once
#[test]
fn generics_unsupported_if_used_twice() {
    #[derive(Facet)]
    struct UnsupportedGenerics<T> {
        field: T,
    }

    #[derive(Facet)]
    struct MyStruct {
        field1: UnsupportedGenerics<String>,
        field2: UnsupportedGenerics<u16>,
    }

    let err = reflect!(MyStruct).unwrap_err();

    insta::assert_snapshot!(err.root_cause(), @"failed to add type MyStruct: unsupported generic type: UnsupportedGenerics<u16>, the type may have already been used with different parameters");
}
