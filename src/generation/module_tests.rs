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
      external_definitions: {}
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

#[test]
fn same_namespace_with_external_dependency_bug_regression() {
    // This test reproduces the specific bug where external dependencies
    // were lost when multiple types in the same namespace were processed
    // and the first type didn't have external dependencies.

    #[derive(Facet)]
    #[facet(namespace = "api")]
    struct GrandChild {
        test: String,
    }

    #[derive(Facet)]
    struct Child {
        api: GrandChild,
    }

    #[derive(Facet)]
    struct Parent {
        event: Child,
    }

    let registries = split("App", &reflect!(Parent));

    // The App module should contain external dependencies to "api" namespace
    // even though Parent itself doesn't directly reference it - Child does
    let app_module = registries
        .keys()
        .find(|m| m.config().module_name() == "App")
        .unwrap();
    let external_deps = &app_module.config().external_definitions;

    // This should NOT be empty - it should include the "api" dependency
    assert!(
        !external_deps.is_empty(),
        "App module should have external dependencies"
    );
    assert!(
        external_deps.contains_key("api"),
        "App module should depend on api namespace"
    );
    assert_eq!(
        external_deps["api"],
        vec!["GrandChild"],
        "App module should reference GrandChild from api"
    );

    insta::assert_yaml_snapshot!(registries, @r"
    ? module_name: App
      encoding: None
      external_definitions:
        api:
          - GrandChild
      external_packages: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
      features: []
    : ? namespace: ROOT
        name: Child
      : STRUCT:
          - - api:
                - TYPENAME:
                    namespace:
                      NAMED: api
                    name: GrandChild
                - []
          - []
      ? namespace: ROOT
        name: Parent
      : STRUCT:
          - - event:
                - TYPENAME:
                    namespace: ROOT
                    name: Child
                - []
          - []
    ? module_name: api
      encoding: None
      external_definitions: {}
      external_packages: {}
      comments: {}
      custom_code: {}
      c_style_enums: false
      package_manifest: true
      features: []
    : ? namespace:
          NAMED: api
        name: GrandChild
      : STRUCT:
          - - test:
                - STR
                - []
          - []
    ");
}
