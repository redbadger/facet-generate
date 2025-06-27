use super::*;

#[test]
fn single_namespace() {
    let registry = serde_yaml::from_str(
        r"
    ChildOne:
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
    ",
    )
    .unwrap();
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
    let registry = serde_yaml::from_str(
        r"
    Parent:
      STRUCT:
        - one:
            TYPENAME: one.ChildOne
        - two:
            TYPENAME: two.ChildTwo
    one.ChildOne:
      STRUCT:
        - child:
            TYPENAME: one.GrandChild
    one.GrandChild:
      STRUCT:
        - field: STR
    two.ChildTwo:
      STRUCT:
        - field: STR
    ",
    )
    .unwrap();
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
