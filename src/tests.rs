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

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: BOOL
        ");
}

#[test]
fn newtype_u8() {
    #[derive(Facet)]
    struct MyNewType(u8);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U8
        ");
}

#[test]
fn newtype_u16() {
    #[derive(Facet)]
    struct MyNewType(u16);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U16
        ");
}

#[test]
fn newtype_u32() {
    #[derive(Facet)]
    struct MyNewType(u32);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U32
        ");
}

#[test]
fn newtype_u64() {
    #[derive(Facet)]
    struct MyNewType(u64);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U64
        ");
}

#[test]
fn newtype_u128() {
    #[derive(Facet)]
    struct MyNewType(u128);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: U128
        ");
}

#[test]
fn newtype_i8() {
    #[derive(Facet)]
    struct MyNewType(i8);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I8
        ");
}

#[test]
fn newtype_i16() {
    #[derive(Facet)]
    struct MyNewType(i16);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I16
        ");
}

#[test]
fn newtype_i32() {
    #[derive(Facet)]
    struct MyNewType(i32);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I32
        ");
}

#[test]
fn newtype_i64() {
    #[derive(Facet)]
    struct MyNewType(i64);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I64
        ");
}

#[test]
fn newtype_i128() {
    #[derive(Facet)]
    struct MyNewType(i128);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT: I128
        ");
}

#[test]
fn nested_newtype() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyNewType(Inner);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        Inner:
          NEWTYPESTRUCT: I32
        MyNewType:
          NEWTYPESTRUCT:
            TYPENAME: Inner
        ");
}

#[test]
fn newtype_with_list() {
    #[derive(Facet)]
    struct MyNewType(Vec<i32>);

    let registry = dbg!(reflect::<MyNewType>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyNewType:
          NEWTYPESTRUCT:
            SEQ: I32
        ");
}

#[test]
fn newtype_with_tuple_array() {
    #[derive(Facet)]
    struct MyNewType([i32; 3]);

    let registry = dbg!(reflect::<MyNewType>());
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

    let registry = dbg!(reflect::<MyTupleStruct>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyTupleStruct:
          TUPLESTRUCT:
            - U8
            - I32
            - BOOL
        ");
}

#[test]
fn nested_tuple_struct_1() {
    #[derive(Facet)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyTupleStruct(Inner, u8);

    let registry = dbg!(reflect::<MyTupleStruct>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        Inner:
          NEWTYPESTRUCT: I32
        MyTupleStruct:
          TUPLESTRUCT:
            - TYPENAME: Inner
            - U8
        ");
}

#[test]
fn nested_tuple_struct_2() {
    #[derive(Facet)]
    struct Inner(i32, u8);

    #[derive(Facet)]
    struct MyTupleStruct(i32, Inner, u8);

    let registry = dbg!(reflect::<MyTupleStruct>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        Inner:
          TUPLESTRUCT:
            - I32
            - U8
        MyTupleStruct:
          TUPLESTRUCT:
            - I32
            - TYPENAME: Inner
            - U8
        ");
}

#[test]
fn struct_with_scalar_fields() {
    #[derive(Facet)]
    struct MyStruct {
        a: u8,
        b: i32,
        c: bool,
    }

    let registry = dbg!(reflect::<MyStruct>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyStruct:
          STRUCT:
            - a: U8
            - b: I32
            - c: BOOL
        ");
}

#[test]
fn struct_with_tuple_field() {
    #[derive(Facet)]
    struct MyStruct {
        a: (u8, i32),
    }

    let registry = dbg!(reflect::<MyStruct>());
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

    let registry = dbg!(reflect::<MyStruct>());
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
              TYPENAME: Inner
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

    let registry = dbg!(reflect::<MyStruct>());
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
            TYPENAME: Inner1
        - b:
            TYPENAME: Inner2
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

    let registry = dbg!(reflect::<MyEnum>());
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
        Variant3(String),
        Variant4(bool),
    }

    let registry = dbg!(reflect::<MyEnum>());
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
            NEWTYPE: STR
        3:
          Variant4:
            NEWTYPE: BOOL
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

    let registry = dbg!(reflect::<MyEnum>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        Inner:
          NEWTYPESTRUCT: I32
        MyEnum:
          ENUM:
            0:
              Variant1:
                NEWTYPE:
                  TYPENAME: Inner
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
        Variant1(u8, i32),
        Variant2(i8, u32),
    }

    let registry = dbg!(reflect::<MyEnum>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        MyEnum:
          ENUM:
            0:
              Variant1:
                TUPLE:
                  - U8
                  - I32
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

    let registry = dbg!(reflect::<MyEnum>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        Inner:
          NEWTYPESTRUCT: I32
        MyEnum:
          ENUM:
            0:
              Variant1:
                TUPLE:
                  - TYPENAME: Inner
                  - U8
            1:
              Variant2:
                TUPLE:
                  - I8
                  - TYPENAME: Inner
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

    let registry = dbg!(reflect::<MyEnum>());
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
                      TYPENAME: Inner
                  - b: U8
                  - c: BOOL
            1:
              Variant2:
                NEWTYPE:
                  TYPENAME: Inner
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
        Variant1 { a: Inner, b: u8, c: String, d: bool },
        Variant2 { x: i32, y: Inner },
    }

    let registry = dbg!(reflect::<MyEnum>());
    insta::assert_yaml_snapshot!(registry.containers, @r"
        Inner:
          NEWTYPESTRUCT: I32
        MyEnum:
          ENUM:
            0:
              Variant1:
                STRUCT:
                  - a:
                      TYPENAME: Inner
                  - b: U8
                  - c: STR
                  - d: BOOL
            1:
              Variant2:
                STRUCT:
                  - x: I32
                  - y:
                      TYPENAME: Inner
        ");
}
