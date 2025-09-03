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

    let registries = split("Root", &reflect!(Parent));
    insta::assert_yaml_snapshot!(registries, @r"
    ? module_name: Root
      encoding: None
      external_definitions: {}
      external_packages: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
      features: []
    : ? namespace: ROOT
        name: ChildOne
      : STRUCT:
          - - child:
                - TYPENAME:
                    namespace: ROOT
                    name: GrandChild
                - []
          - []
      ? namespace: ROOT
        name: ChildTwo
      : STRUCT:
          - - field:
                - STR
                - []
          - []
      ? namespace: ROOT
        name: GrandChild
      : STRUCT:
          - - field:
                - STR
                - []
          - []
      ? namespace: ROOT
        name: Parent
      : STRUCT:
          - - one:
                - TYPENAME:
                    namespace: ROOT
                    name: ChildOne
                - []
            - two:
                - TYPENAME:
                    namespace: ROOT
                    name: ChildTwo
                - []
          - []
    ");
}

#[test]
#[allow(clippy::too_many_lines)]
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

    let registries = split("Root", &reflect!(Parent));
    insta::assert_yaml_snapshot!(registries, @r"
    ? module_name: Root
      encoding: None
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
      features: []
    : ? namespace: ROOT
        name: Parent
      : STRUCT:
          - - one:
                - TYPENAME:
                    namespace:
                      NAMED: one
                    name: ChildOne
                - []
            - two:
                - TYPENAME:
                    namespace:
                      NAMED: two
                    name: ChildTwo
                - []
          - []
    ? module_name: one
      encoding: None
      external_definitions:
        one:
          - GrandChild
      external_packages: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
      features: []
    : ? namespace:
          NAMED: one
        name: ChildOne
      : STRUCT:
          - - child:
                - TYPENAME:
                    namespace:
                      NAMED: one
                    name: GrandChild
                - []
          - []
      ? namespace:
          NAMED: one
        name: GrandChild
      : STRUCT:
          - - field:
                - STR
                - []
          - []
    ? module_name: two
      encoding: None
      external_definitions: {}
      external_packages: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
      features: []
    : ? namespace:
          NAMED: two
        name: ChildTwo
      : STRUCT:
          - - field:
                - STR
                - []
          - []
    ");
}
