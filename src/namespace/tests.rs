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
    let result = split(&registry);
    insta::assert_yaml_snapshot!(result, @r"
    Root:
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
    ");
}

#[test]
fn root_namespace_with_two_child_namespaces() {
    let registry = serde_yaml::from_str(
        r"
    Parent:
      STRUCT:
        - one:
            TYPENAME: one.Child
        - two:
            TYPENAME: two.Child
    one.Child:
      STRUCT:
        - child:
            TYPENAME: one.GrandChild
    one.GrandChild:
      STRUCT:
        - field: STR
    two.Child:
      STRUCT:
        - field: STR
    ",
    )
    .unwrap();
    let result = split(&registry);
    insta::assert_yaml_snapshot!(result, @r"
    Root:
      Parent:
        STRUCT:
          - one:
              TYPENAME: one.Child
          - two:
              TYPENAME: two.Child
    ? Child: one
    : Child:
        STRUCT:
          - child:
              TYPENAME: one.GrandChild
      GrandChild:
        STRUCT:
          - field: STR
    ? Child: two
    : Child:
        STRUCT:
          - field: STR
    ");
}
