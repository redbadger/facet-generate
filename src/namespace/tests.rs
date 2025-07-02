use facet::Facet;

use crate::reflect;

use super::*;

#[test]
fn single_namespace() {
    #[derive(Facet)]
    struct ChildOne {
        child: GrandChild,
    }

    #[derive(Facet)]
    struct ChildTwo {
        field: String,
    }

    #[derive(Facet)]
    struct GrandChild {
        field: String,
    }

    #[derive(Facet)]
    struct Parent {
        one: ChildOne,
        two: ChildTwo,
    }

    let registry = reflect::<Parent>();
    let result = split("Root", registry).unwrap();
    insta::assert_yaml_snapshot!(result, @r"
    ? module_name: Root
      serialization: true
      encodings: []
      external_definitions: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
    : ChildOne:
        STRUCT:
          - child:
              TYPENAME: GrandChild
      ChildTwo:
        STRUCT:
          - field: STR
      GrandChild:
        STRUCT:
          - field: STR
      Parent:
        STRUCT:
          - one:
              TYPENAME: ChildOne
          - two:
              TYPENAME: ChildTwo
    ");
}

#[test]
fn root_namespace_with_two_child_namespaces() {
    #[derive(Facet)]
    #[facet(namespace = "one")]
    struct ChildOne {
        child: GrandChild,
    }

    #[derive(Facet)]
    #[facet(namespace = "two")]
    struct ChildTwo {
        field: String,
    }

    #[derive(Facet)]
    #[facet(namespace = "one")]
    struct GrandChild {
        field: String,
    }

    #[derive(Facet)]
    struct Parent {
        one: ChildOne,
        two: ChildTwo,
    }

    let registry = reflect::<Parent>();
    let result = split("Root", registry).unwrap();
    insta::assert_yaml_snapshot!(result, @r"
    ? module_name: Root
      serialization: true
      encodings: []
      external_definitions:
        one:
          - ChildOne
        two:
          - ChildTwo
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
    : Parent:
        STRUCT:
          - one:
              TYPENAME: ChildOne
          - two:
              TYPENAME: ChildTwo
    ? module_name: one
      serialization: true
      encodings: []
      external_definitions:
        one:
          - GrandChild
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
    : ChildOne:
        STRUCT:
          - child:
              TYPENAME: GrandChild
      GrandChild:
        STRUCT:
          - field: STR
    ? module_name: two
      serialization: true
      encodings: []
      external_definitions: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
    : ChildTwo:
        STRUCT:
          - field: STR
    ");
}
