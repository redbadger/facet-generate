use facet::Facet;

use crate::reflection::RegistryBuilder;

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

    let registry = RegistryBuilder::new().add_type::<Parent>().build();
    let result = split("Root", &registry);
    insta::assert_yaml_snapshot!(result, @r"
    ? module_name: Root
      serialization: true
      encodings: []
      external_definitions: {}
      external_packages: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
    : ? namespace: ROOT
        name: ChildOne
      : STRUCT:
          - child:
              TYPENAME:
                namespace: ROOT
                name: GrandChild
      ? namespace: ROOT
        name: ChildTwo
      : STRUCT:
          - field: STR
      ? namespace: ROOT
        name: GrandChild
      : STRUCT:
          - field: STR
      ? namespace: ROOT
        name: Parent
      : STRUCT:
          - one:
              TYPENAME:
                namespace: ROOT
                name: ChildOne
          - two:
              TYPENAME:
                namespace: ROOT
                name: ChildTwo
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

    let registry = RegistryBuilder::new().add_type::<Parent>().build();
    let result = split("Root", &registry);
    insta::assert_yaml_snapshot!(result, @r"
    ? module_name: Root
      serialization: true
      encodings: []
      external_definitions:
        one:
          - ChildOne
        two:
          - ChildTwo
      external_packages: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
    : ? namespace: ROOT
        name: Parent
      : STRUCT:
          - one:
              TYPENAME:
                namespace:
                  NAMED: one
                name: ChildOne
          - two:
              TYPENAME:
                namespace:
                  NAMED: two
                name: ChildTwo
    ? module_name: one
      serialization: true
      encodings: []
      external_definitions:
        one:
          - GrandChild
      external_packages: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
    : ? namespace:
          NAMED: one
        name: ChildOne
      : STRUCT:
          - child:
              TYPENAME:
                namespace:
                  NAMED: one
                name: GrandChild
      ? namespace:
          NAMED: one
        name: GrandChild
      : STRUCT:
          - field: STR
    ? module_name: two
      serialization: true
      encodings: []
      external_definitions: {}
      external_packages: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
    : ? namespace:
          NAMED: two
        name: ChildTwo
      : STRUCT:
          - field: STR
    ");
}
