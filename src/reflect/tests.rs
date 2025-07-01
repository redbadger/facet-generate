use std::sync::Arc;

use super::*;

#[test]
fn unit_struct() {
    #[derive(Facet)]
    struct MyUnitStruct;

    let registry = reflect::<MyUnitStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @"MyUnitStruct: UNITSTRUCT");
}

#[test]
fn newtype_bool() {
    #[derive(Facet)]
    struct MyNewType(bool);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: BOOL
        ");
}

#[test]
fn newtype_unit() {
    #[derive(Facet)]
    struct MyNewType(());

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: UNIT
        ");
}

#[test]
fn newtype_u8() {
    #[derive(Facet)]
    struct MyNewType(u8);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U8
        ");
}

#[test]
fn newtype_u16() {
    #[derive(Facet)]
    struct MyNewType(u16);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U16
        ");
}

#[test]
fn newtype_u32() {
    #[derive(Facet)]
    struct MyNewType(u32);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U32
        ");
}

#[test]
fn newtype_u64() {
    #[derive(Facet)]
    struct MyNewType(u64);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U64
        ");
}

#[test]
fn newtype_u128() {
    #[derive(Facet)]
    struct MyNewType(u128);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U128
        ");
}

#[test]
fn newtype_i8() {
    #[derive(Facet)]
    struct MyNewType(i8);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I8
        ");
}

#[test]
fn newtype_i16() {
    #[derive(Facet)]
    struct MyNewType(i16);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I16
        ");
}

#[test]
fn newtype_i32() {
    #[derive(Facet)]
    struct MyNewType(i32);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I32
        ");
}

#[test]
fn newtype_i64() {
    #[derive(Facet)]
    struct MyNewType(i64);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I64
        ");
}

#[test]
fn newtype_i128() {
    #[derive(Facet)]
    struct MyNewType(i128);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I128
        ");
}

#[test]
fn newtype_f32() {
    #[derive(Facet)]
    struct MyNewType(f32);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: F32
        ");
}

#[test]
fn newtype_f64() {
    #[derive(Facet)]
    struct MyNewType(f64);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: F64
        ");
}

#[test]
fn newtype_char() {
    #[derive(Facet)]
    struct MyNewType(char);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: CHAR
        ");
}

#[test]
fn nested_newtype() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyNewType(Inner);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyNewType:
      NEWTYPESTRUCT:
        QUALIFIEDTYPENAME:
          namespace: ROOT
          name: Inner
    ");
}

#[test]
fn newtype_with_list() {
    #[derive(Facet)]
    struct MyNewType(Vec<i32>);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT:
            SEQ: I32
        ");
}

#[test]
fn newtype_with_list_of_named_type() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyNewType(Vec<Inner>);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyNewType:
      NEWTYPESTRUCT:
        SEQ:
          QUALIFIEDTYPENAME:
            namespace: ROOT
            name: Inner
    ");
}

#[test]
fn newtype_with_nested_list() {
    #[derive(Facet)]
    struct MyNewType(Vec<Vec<i32>>);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT:
            SEQ:
              SEQ: I32
        ");
}

#[test]
fn newtype_with_nested_list_of_named_type() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyNewType(Vec<Vec<Inner>>);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyNewType:
      NEWTYPESTRUCT:
        SEQ:
          SEQ:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: Inner
    ");
}

#[test]
fn newtype_with_triple_nested_list_of_named_type() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyNewType(Vec<Vec<Vec<Inner>>>);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyNewType:
      NEWTYPESTRUCT:
        SEQ:
          SEQ:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: Inner
    ");
}

#[test]
fn newtype_with_tuple_array() {
    #[derive(Facet)]
    struct MyNewType([i32; 3]);

    let registry = reflect::<MyNewType>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT:
            TUPLEARRAY:
              CONTENT: I32
              SIZE: 3
        ");
}

#[test]
fn tuple_struct() {
    #[derive(Facet)]
    struct MyTupleStruct(u8, i32, bool);

    let registry = reflect::<MyTupleStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyTupleStruct:
          TUPLESTRUCT:
            - U8
            - I32
            - BOOL
        ");
}

#[test]
fn tuple_struct_with_unit() {
    #[derive(Facet)]
    struct MyTupleStruct(u8, (), i32);

    let registry = reflect::<MyTupleStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyTupleStruct:
          TUPLESTRUCT:
            - U8
            - UNIT
            - I32
        ");
}

#[test]
fn option_of_unit() {
    #[derive(Facet)]
    struct MyStruct {
        a: Option<()>,
    }

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyStruct:
          STRUCT:
            - a:
                OPTION: UNIT
        ");
}

#[test]
fn option_of_list() {
    #[derive(Facet)]
    struct MyStruct {
        a: Option<Vec<i32>>,
    }

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyStruct:
          STRUCT:
            - a:
                OPTION:
                  SEQ: I32
        ");
}

#[test]
fn option_of_nested_list() {
    #[derive(Facet)]
    struct MyStruct {
        a: Option<Vec<Vec<i32>>>,
    }

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyStruct:
          STRUCT:
            - a:
                OPTION:
                  SEQ:
                    SEQ: I32
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

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyStruct:
      STRUCT:
        - a:
            OPTION:
              SEQ:
                QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: Inner
    ");
}

#[test]
fn list_of_options() {
    #[derive(Facet)]
    struct MyStruct {
        a: Vec<Option<i32>>,
    }

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyStruct:
          STRUCT:
            - a:
                SEQ:
                  OPTION: I32
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

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyStruct:
      STRUCT:
        - a:
            SEQ:
              OPTION:
                QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: Inner
    ");
}

#[test]
fn nested_list_with_options() {
    #[derive(Facet)]
    struct MyStruct {
        a: Vec<Vec<Option<i32>>>,
    }

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyStruct:
          STRUCT:
            - a:
                SEQ:
                  SEQ:
                    OPTION: I32
        ");
}

#[test]
fn nested_tuple_struct_1() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyTupleStruct(Inner, u8);

    let registry = reflect::<MyTupleStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyTupleStruct:
      TUPLESTRUCT:
        - QUALIFIEDTYPENAME:
            namespace: ROOT
            name: Inner
        - U8
    ");
}

#[test]
fn nested_tuple_struct_2() {
    #[derive(Facet)]
    struct Inner(i32, u8, bool);

    #[derive(Facet)]
    struct MyTupleStruct(i32, Inner, u8);

    let registry = reflect::<MyTupleStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      TUPLESTRUCT:
        - I32
        - U8
        - BOOL
    MyTupleStruct:
      TUPLESTRUCT:
        - I32
        - QUALIFIEDTYPENAME:
            namespace: ROOT
            name: Inner
        - U8
    ");
}

#[test]
fn struct_with_vec_of_u8_to_bytes() {
    #[derive(Facet)]
    struct MyStruct {
        #[facet(bytes)]
        a: Vec<u8>,
    }

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyStruct:
      STRUCT:
        - a: BYTES
    ");
}

#[test]
fn struct_with_slice_of_u8_to_bytes() {
    #[derive(Facet)]
    struct MyStruct<'a> {
        #[facet(bytes)]
        a: &'a [u8],
    }

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyStruct:
      STRUCT:
        - a: BYTES
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

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyStruct:
          STRUCT:
            - a: U8
            - b: I32
            - c: BOOL
            - d: UNIT
        ");
}

#[test]
fn struct_with_tuple_field() {
    #[derive(Facet)]
    struct MyStruct {
        a: (u8, i32),
    }

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyStruct:
      STRUCT:
        - a:
            TUPLE:
              - U8
              - I32
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

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      STRUCT:
        - a:
            OPTION: BOOL
        - b:
            OPTION: U8
    MyStruct:
      STRUCT:
        - a:
            OPTION:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: Inner
        - b:
            OPTION: U8
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

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner1:
      NEWTYPESTRUCT: I32
    Inner2:
      TUPLESTRUCT:
        - I8
        - U32
    MyStruct:
      STRUCT:
        - a:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: Inner1
        - b:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: Inner2
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyEnum:
          ENUM:
            0:
              Variant1: UNIT
            1:
              Variant2: UNIT
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyEnum:
      ENUM:
        0:
          Variant1:
            NEWTYPE: U8
        1:
          Variant2:
            NEWTYPE: I32
        2:
          Variant3:
            NEWTYPE: F64
        3:
          Variant4:
            NEWTYPE: CHAR
        4:
          Variant5:
            NEWTYPE: STR
        5:
          Variant6:
            NEWTYPE: BOOL
        6:
          Variant7:
            NEWTYPE: UNIT
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyEnum:
      ENUM:
        0:
          Variant1:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: Inner
        1:
          Variant2:
            NEWTYPE: U8
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyEnum:
      ENUM:
        0:
          Variant1:
            TUPLE:
              - U8
              - I32
              - F64
              - CHAR
              - STR
              - BOOL
              - UNIT
        1:
          Variant2:
            TUPLE:
              - I8
              - U32
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyEnum:
      ENUM:
        0:
          Variant1:
            TUPLE:
              - QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: Inner
              - U8
        1:
          Variant2:
            TUPLE:
              - I8
              - QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: Inner
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      STRUCT:
        - a: STR
    MyEnum:
      ENUM:
        0:
          Variant1:
            STRUCT:
              - a:
                  QUALIFIEDTYPENAME:
                    namespace: ROOT
                    name: Inner
              - b: U8
              - c: BOOL
        1:
          Variant2:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: Inner
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Inner:
      NEWTYPESTRUCT: I32
    MyEnum:
      ENUM:
        0:
          Variant1:
            STRUCT:
              - a:
                  QUALIFIEDTYPENAME:
                    namespace: ROOT
                    name: Inner
              - b: U8
              - c: F32
              - d: STR
              - e: CHAR
              - f: BOOL
              - g: UNIT
        1:
          Variant2:
            STRUCT:
              - x: I32
              - y:
                  QUALIFIEDTYPENAME:
                    namespace: ROOT
                    name: Inner
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyEnum:
      ENUM:
        0:
          A: UNIT
        1:
          B:
            NEWTYPE: U64
        2:
          C:
            STRUCT:
              - x: U8
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyEnum:
      ENUM:
        0:
          Variant1: UNIT
        1:
          Variant3: UNIT
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

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyStruct:
      STRUCT:
        - inner: I32
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

    let registry = reflect::<MyEnum>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyEnum:
          ENUM:
            0:
              Map:
                NEWTYPE:
                  MAP:
                    KEY: STR
                    VALUE: BOOL
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

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyStruct:
      STRUCT:
        - user_map:
            MAP:
              KEY:
                QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: UserId
              VALUE:
                QUALIFIEDTYPENAME:
                  namespace: ROOT
                  name: UserProfile
        - id_to_count:
            MAP:
              KEY: I32
              VALUE:
                SEQ: STR
        - nested_map:
            MAP:
              KEY: STR
              VALUE:
                OPTION: U64
    UserId:
      NEWTYPESTRUCT: U32
    UserProfile:
      STRUCT:
        - name: STR
        - active: BOOL
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

    let registry = reflect::<ComplexMap>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    ComplexMap:
      ENUM:
        0:
          Map:
            NEWTYPE:
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

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyStruct:
      STRUCT:
        - boxed:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: UserProfile
    UserProfile:
      STRUCT:
        - name: STR
        - active: BOOL
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

    let registry = reflect::<MyStruct>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    MyStruct:
      STRUCT:
        - boxed:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: UserProfile
    UserProfile:
      STRUCT:
        - name: STR
        - active: BOOL
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

    #[derive(facet::Facet, PartialEq, Eq, Clone, Debug)]
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

    let registry = reflect::<HttpResult>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    HttpError:
      ENUM:
        0:
          Url:
            NEWTYPE: STR
        1:
          Io:
            NEWTYPE: STR
        2:
          Timeout: UNIT
    HttpHeader:
      STRUCT:
        - name: STR
        - value: STR
    HttpResponse:
      STRUCT:
        - status: U16
        - headers:
            SEQ:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: HttpHeader
        - body: BYTES
    HttpResult:
      ENUM:
        0:
          Ok:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: HttpResponse
        1:
          Err:
            NEWTYPE:
              QUALIFIEDTYPENAME:
                namespace: ROOT
                name: HttpError
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

    let registry = reflect::<EffectFfi>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Effect:
      STRUCT:
        - name: STR
        - active: BOOL
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

    let registry = reflect::<EffectFfi>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Effect:
      ENUM:
        0:
          One: UNIT
        1:
          Two: UNIT
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

    let registry = reflect::<Request>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Effect:
      STRUCT:
        - inner: STR
    Request:
      STRUCT:
        - id: U32
        - effect:
            QUALIFIEDTYPENAME:
              namespace: ROOT
              name: Effect
    ");
}

#[test]
fn self_referencing_type() {
    #[derive(Facet)]
    struct SimpleList(Option<Box<SimpleList>>);

    let registry = reflect::<SimpleList>();

    insta::assert_debug_snapshot!(registry.containers, @r#"
    {
        "SimpleList": NewTypeStruct(
            Option(
                QualifiedTypeName(
                    QualifiedTypeName {
                        namespace: Root,
                        name: "SimpleList",
                    },
                ),
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

    let registry = reflect::<Node>();

    insta::assert_debug_snapshot!(registry.containers, @r#"
    {
        "Node": Struct(
            [
                Named {
                    name: "value",
                    value: I32,
                },
                Named {
                    name: "children",
                    value: Option(
                        Seq(
                            QualifiedTypeName(
                                QualifiedTypeName {
                                    namespace: Root,
                                    name: "Node",
                                },
                            ),
                        ),
                    ),
                },
            ],
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

    let registry = reflect::<Test>();

    insta::assert_debug_snapshot!(registry.containers, @r#"
    {
        "Test": Enum(
            {
                0: Named {
                    name: "TreeWithMutualRecursion",
                    value: NewType(
                        QualifiedTypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "Tree",
                            },
                        ),
                    ),
                },
            },
        ),
        "Tree": Struct(
            [
                Named {
                    name: "value",
                    value: QualifiedTypeName(
                        QualifiedTypeName {
                            namespace: Root,
                            name: "Test",
                        },
                    ),
                },
                Named {
                    name: "children",
                    value: Seq(
                        QualifiedTypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "Tree",
                            },
                        ),
                    ),
                },
            ],
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

    let registry = reflect::<Test>();

    insta::assert_debug_snapshot!(registry.containers, @r#"
    {
        "Test": Struct(
            [
                Named {
                    name: "tree_with_mutual_recursion",
                    value: QualifiedTypeName(
                        QualifiedTypeName {
                            namespace: Root,
                            name: "Tree",
                        },
                    ),
                },
            ],
        ),
        "Tree": Enum(
            {
                0: Named {
                    name: "Value",
                    value: NewType(
                        QualifiedTypeName(
                            QualifiedTypeName {
                                namespace: Root,
                                name: "Test",
                            },
                        ),
                    ),
                },
            },
        ),
    }
    "#);
}
