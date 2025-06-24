use crate::reflect;

#[test]
fn nested_namespaced_structs() {
    mod one {
        #[derive(facet::Facet)]
        pub struct GrandChild {
            field: String,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "one")]
        pub struct Child {
            child: GrandChild,
        }
    }
    mod two {
        #[derive(facet::Facet)]
        #[facet(namespace = "two")]
        pub struct Child {
            field: String,
        }
    }

    #[derive(facet::Facet)]
    struct Parent {
        one: one::Child,
        two: two::Child,
    }

    let registry = reflect::<Parent>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
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
    ");
}

#[test]
fn nested_namespaced_enums() {
    mod one {
        #![allow(unused)]
        #[derive(facet::Facet)]
        #[repr(C)]
        pub enum GrandChild {
            None,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "one")]
        #[repr(C)]
        pub enum Child {
            Data(GrandChild),
        }
    }
    mod two {
        #![allow(unused)]
        #[derive(facet::Facet)]
        #[repr(C)]
        #[facet(namespace = "two")]
        pub enum Child {
            Data(String),
        }
    }

    #[derive(facet::Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Parent {
        One(one::Child),
        Two(two::Child),
    }

    let registry = reflect::<Parent>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Parent:
      ENUM:
        0:
          One:
            NEWTYPE:
              TYPENAME: one.Child
        1:
          Two:
            NEWTYPE:
              TYPENAME: two.Child
    one.Child:
      ENUM:
        0:
          Data:
            NEWTYPE:
              TYPENAME: one.GrandChild
    one.GrandChild:
      ENUM:
        0:
          None: UNIT
    two.Child:
      ENUM:
        0:
          Data:
            NEWTYPE: STR
    ");
}

#[test]
fn nested_namespaced_renamed_structs() {
    mod one {
        #[derive(facet::Facet)]
        #[facet(name = "GrandKid")]
        pub struct GrandChild {
            field: String,
        }

        #[derive(facet::Facet)]
        #[facet(namespace = "one")]
        #[facet(name = "Kid")]
        pub struct Child {
            child: GrandChild,
        }
    }
    mod two {
        #[derive(facet::Facet)]
        #[facet(namespace = "two")]
        #[facet(name = "Kid")]
        pub struct Child {
            field: String,
        }
    }

    #[derive(facet::Facet)]
    #[facet(name = "Pop")]
    struct Parent {
        one: one::Child,
        two: two::Child,
    }

    let registry = reflect::<Parent>();
    insta::assert_yaml_snapshot!(registry.containers, @r"
    Pop:
      STRUCT:
        - one:
            TYPENAME: one.Kid
        - two:
            TYPENAME: two.Kid
    one.GrandKid:
      STRUCT:
        - field: STR
    one.Kid:
      STRUCT:
        - child:
            TYPENAME: one.GrandKid
    two.Kid:
      STRUCT:
        - field: STR
    ");
}
