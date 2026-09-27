//! Every reference to a type names the type as it is registered, whatever wraps the reference
//! (#160, #167), and `build` rejects a reference that names no registered type.

use std::collections::{BTreeMap, HashMap};

use facet::Facet;

use crate as fg;
use crate::{
    Registry,
    error::Error,
    reflect,
    reflection::{
        RegistryBuilder,
        format::{
            ContainerFormat, Doc, Format, FormatHolder, Named, QualifiedTypeName, VariantFormat,
        },
    },
};

/// Every reference in `registry` to a type called `name`, as `(referring container, location,
/// the name referred to)`.
fn references_to(registry: &Registry, name: &str) -> Vec<(String, String, QualifiedTypeName)> {
    let mut found = vec![];
    let mut collect = |container: &QualifiedTypeName, location: String, format: &Format| {
        format
            .visit(&mut |format| {
                if let Format::TypeName(qualified) = format
                    && qualified.name == name
                {
                    found.push((container.to_string(), location.clone(), qualified.clone()));
                }
                Ok(())
            })
            .unwrap();
    };
    for (container_name, container) in registry {
        match container {
            ContainerFormat::UnitStruct(_) => {}
            ContainerFormat::NewTypeStruct(format, _) => {
                collect(container_name, "0".to_string(), format);
            }
            ContainerFormat::TupleStruct(formats, _) => {
                for (index, format) in formats.iter().enumerate() {
                    collect(container_name, index.to_string(), format);
                }
            }
            ContainerFormat::Struct(fields, _) => {
                for field in fields {
                    collect(container_name, field.name.clone(), &field.value);
                }
            }
            ContainerFormat::Enum(variants, _, _) => {
                for variant in variants.values() {
                    match &variant.value {
                        VariantFormat::Variable(_) | VariantFormat::Unit => {}
                        VariantFormat::NewType(format) => {
                            collect(container_name, variant.name.clone(), format);
                        }
                        VariantFormat::Tuple(formats) => {
                            for (index, format) in formats.iter().enumerate() {
                                collect(
                                    container_name,
                                    format!("{}.{index}", variant.name),
                                    format,
                                );
                            }
                        }
                        VariantFormat::Struct(fields) => {
                            for field in fields {
                                let location = format!("{}.{}", variant.name, field.name);
                                collect(container_name, location, &field.value);
                            }
                        }
                    }
                }
            }
        }
    }
    found
}

/// Asserts that `Leaf` is registered once, as `expected`, and that every reference to it, from
/// every position in [`holders!`], names it as `expected`. `leaf_in_root` is whether the ROOT
/// `App` holds a `Leaf` too.
fn assert_every_reference_names(
    registry: &Registry,
    expected: &QualifiedTypeName,
    leaf_in_root: bool,
) {
    let registered: Vec<_> = registry.keys().filter(|key| key.name == "Leaf").collect();
    assert_eq!(
        registered,
        [expected],
        "`Leaf` registered as {registered:?}"
    );

    let references = references_to(registry, "Leaf");
    let wrong: Vec<_> = references
        .iter()
        .filter(|(_, _, name)| name != expected)
        .collect();
    assert!(
        wrong.is_empty(),
        "references not naming {expected}: {wrong:#?}"
    );

    let mut locations: Vec<_> = references
        .iter()
        .map(|(container, location, _)| format!("{container}.{location}"))
        .collect();
    locations.sort();
    let mut expected_locations = vec![
        "detail::Holder.array",
        "detail::Holder.direct",
        "detail::Holder.map_key",
        "detail::Holder.map_value",
        "detail::Holder.option",
        "detail::Holder.option_seq",
        "detail::Holder.seq",
        "detail::Holder.seq_option",
        "detail::Holder.set",
        "detail::Holder.transparent",
        "detail::Holder.transparent_seq",
        "detail::Holder.tuple",
        "detail::Pair.0",
        "detail::Pair.1",
        "detail::Payload.NewType",
        "detail::Payload.NewTypeSeq",
        "detail::Payload.Struct.map",
        "detail::Payload.Struct.option",
        "detail::Payload.Tuple.0",
        "detail::Payload.Tuple.1",
        "detail::Wrapped.0",
    ];
    if leaf_in_root {
        expected_locations.insert(0, "ROOT::App.leaf_in_root");
    }
    assert_eq!(locations, expected_locations);
}

/// Types in namespace `detail` referring to `Leaf` from every kind of position, each wrapping it
/// differently, and a ROOT `App` holding them. `$leaf` is `Leaf`'s own namespace attribute, if
/// any, and `$leaf_in_root` names a field of `App` holding a `Leaf` directly. (An unannotated
/// `Leaf` held by `App` would be registered in ROOT as well as in `detail`.)
macro_rules! holders {
    ($(#[$leaf:meta])* ; $($leaf_in_root:ident)?) => {
        #[derive(Facet, PartialEq, Eq, PartialOrd, Ord)]
        $(#[$leaf])*
        pub struct Leaf {
            pub id: u32,
        }

        #[derive(Facet)]
        #[facet(transparent)]
        pub struct LeafId(Leaf);

        #[derive(Facet)]
        #[facet(fg::namespace = "detail")]
        pub struct Holder {
            pub direct: Leaf,
            pub option: Option<Leaf>,
            pub seq: Vec<Leaf>,
            pub option_seq: Option<Vec<Leaf>>,
            pub seq_option: Vec<Option<Leaf>>,
            pub set: std::collections::BTreeSet<Leaf>,
            pub map_key: BTreeMap<Leaf, u8>,
            pub map_value: BTreeMap<u8, Leaf>,
            pub tuple: (Leaf, u8),
            pub array: [Leaf; 2],
            pub transparent: LeafId,
            pub transparent_seq: Vec<LeafId>,
        }

        #[derive(Facet)]
        #[repr(C)]
        #[facet(fg::namespace = "detail")]
        #[allow(dead_code)]
        pub enum Payload {
            NewType(Option<Leaf>),
            NewTypeSeq(Vec<Leaf>),
            Tuple(Option<Leaf>, Vec<(u8, Leaf)>),
            Struct {
                option: Option<Leaf>,
                map: BTreeMap<u8, Vec<Leaf>>,
            },
        }

        #[derive(Facet)]
        #[facet(fg::namespace = "detail")]
        pub struct Pair(Option<Leaf>, Vec<Leaf>);

        #[derive(Facet)]
        #[facet(fg::namespace = "detail")]
        pub struct Wrapped(Vec<Option<Leaf>>);

        #[derive(Facet)]
        pub struct App {
            pub holder: Holder,
            pub payload: Payload,
            pub pair: Pair,
            pub wrapped: Wrapped,
            $(pub $leaf_in_root: Leaf,)?
        }
    };
}

#[test]
fn an_unannotated_type_is_referred_to_in_the_namespace_it_inherits() {
    holders!(;);

    let registry = reflect!(App).unwrap();
    assert_every_reference_names(
        &registry,
        &QualifiedTypeName::namespaced("detail".to_string(), "Leaf".to_string()),
        false,
    );
}

#[test]
fn a_type_pinned_to_root_is_referred_to_in_root() {
    holders!(#[facet(fg::namespace)]; leaf_in_root);

    let registry = reflect!(App).unwrap();
    assert_every_reference_names(
        &registry,
        &QualifiedTypeName::root("Leaf".to_string()),
        true,
    );
}

#[test]
fn an_explicitly_namespaced_type_is_referred_to_in_its_namespace() {
    holders!(#[facet(fg::namespace = "other")]; leaf_in_root);

    let registry = reflect!(App).unwrap();
    assert_every_reference_names(
        &registry,
        &QualifiedTypeName::namespaced("other".to_string(), "Leaf".to_string()),
        true,
    );
}

/// #167: a type with no namespace attribute, reached from a namespaced one, is referred to in
/// that namespace from inside `Option` as well as from inside `Vec`.
#[test]
fn an_inherited_namespace_is_kept_inside_an_option() {
    #[derive(Facet)]
    pub struct Neighbour {
        pub id: u32,
    }

    #[derive(Facet)]
    pub struct Neighbourhood {
        pub manager: Option<Neighbour>,
        pub reports: Vec<Neighbour>,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "detail")]
    pub struct ViewModel {
        pub neighbourhood: Neighbourhood,
    }

    #[derive(Facet)]
    pub struct App {
        pub detail: ViewModel,
    }

    insta::assert_yaml_snapshot!(reflect!(App).unwrap(), @"
    ? namespace: ROOT
      name: App
    : STRUCT:
        - - detail:
              - TYPENAME:
                  namespace:
                    NAMED: detail
                  name: ViewModel
              - []
        - []
    ? namespace:
        NAMED: detail
      name: Neighbour
    : STRUCT:
        - - id:
              - U32
              - []
        - []
    ? namespace:
        NAMED: detail
      name: Neighbourhood
    : STRUCT:
        - - manager:
              - OPTION:
                  TYPENAME:
                    namespace:
                      NAMED: detail
                    name: Neighbour
              - []
          - reports:
              - SEQ:
                  TYPENAME:
                    namespace:
                      NAMED: detail
                    name: Neighbour
              - []
        - []
    ? namespace:
        NAMED: detail
      name: ViewModel
    : STRUCT:
        - - neighbourhood:
              - TYPENAME:
                  namespace:
                    NAMED: detail
                  name: Neighbourhood
              - []
        - []
    ");
}

/// #160: a type pinned to ROOT keeps its pin when a namespaced type holds it inside a generic.
#[test]
fn a_root_pin_is_kept_inside_a_generic() {
    #[derive(Facet)]
    #[facet(fg::namespace)]
    pub struct Shared {
        pub id: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    pub struct Entry {
        pub one: Shared,
        pub many: Vec<Option<Shared>>,
    }

    insta::assert_yaml_snapshot!(reflect!(Entry).unwrap(), @"
    ? namespace: ROOT
      name: Shared
    : STRUCT:
        - - id:
              - U32
              - []
        - []
    ? namespace:
        NAMED: kv
      name: Entry
    : STRUCT:
        - - one:
              - TYPENAME:
                  namespace: ROOT
                  name: Shared
              - []
          - many:
              - SEQ:
                  OPTION:
                    TYPENAME:
                      namespace: ROOT
                      name: Shared
              - []
        - []
    ");
}

/// An enum held by a newtype or tuple struct is the struct's field, rather than the enum's own
/// payloads leaking into the struct.
#[test]
fn an_enum_in_a_tuple_struct_is_its_field() {
    #[derive(Facet)]
    pub struct Leaf {
        pub id: u32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Choice {
        One(Leaf),
        Many(Vec<Leaf>),
    }

    #[derive(Facet)]
    pub struct Wrapper(Choice);

    #[derive(Facet)]
    pub struct Pair(u8, Choice);

    #[derive(Facet)]
    pub struct App {
        pub wrapper: Wrapper,
        pub pair: Pair,
    }

    let registry = reflect!(App).unwrap();
    let choice = Format::TypeName(QualifiedTypeName::root("Choice".to_string()));
    assert_eq!(
        registry[&QualifiedTypeName::root("Wrapper".to_string())],
        ContainerFormat::NewTypeStruct(Box::new(choice.clone()), Doc::default()),
    );
    assert_eq!(
        registry[&QualifiedTypeName::root("Pair".to_string())],
        ContainerFormat::TupleStruct(vec![Format::U8, choice], Doc::default()),
    );
}

#[test]
fn a_reference_to_an_unregistered_type_is_rejected() {
    let mut builder = RegistryBuilder::new();
    builder.registry.insert(
        QualifiedTypeName::namespaced("detail".to_string(), "Neighbourhood".to_string()),
        ContainerFormat::Struct(
            vec![Named {
                name: "manager".to_string(),
                doc: Doc::default(),
                value: Format::Option(Box::new(Format::TypeName(QualifiedTypeName::root(
                    "Neighbour".to_string(),
                )))),
            }],
            Doc::default(),
        ),
    );

    let err = builder.build().unwrap_err();
    assert_eq!(
        err,
        Error::DanglingTypeReference {
            name: "Neighbourhood".to_string(),
            namespace: "detail".to_string(),
            location: "manager".to_string(),
            missing_name: "Neighbour".to_string(),
            missing_namespace: "ROOT".to_string(),
        }
    );
    insta::assert_snapshot!(err, @r#"`manager` in "Neighbourhood" in namespace "detail" refers to "Neighbour" in namespace "ROOT", which is not a registered type. This is a bug in facet_generate's reflection; please report it"#);
}

#[test]
fn a_dangling_reference_in_an_enum_payload_is_located_by_its_variant() {
    let mut builder = RegistryBuilder::new();
    builder.registry.insert(
        QualifiedTypeName::root("Event".to_string()),
        ContainerFormat::Enum(
            BTreeMap::from([(
                0,
                Named {
                    name: "Moved".to_string(),
                    doc: Doc::default(),
                    value: VariantFormat::Struct(vec![Named {
                        name: "to".to_string(),
                        doc: Doc::default(),
                        value: Format::Seq(Box::new(Format::TypeName(QualifiedTypeName::root(
                            "Place".to_string(),
                        )))),
                    }]),
                },
            )]),
            crate::reflection::format::EnumTagging::External,
            Doc::default(),
        ),
    );

    let err = builder.build().unwrap_err();
    assert!(
        matches!(&err, Error::DanglingTypeReference { location, .. } if location == "Moved.to"),
        "{err:?}"
    );
}

/// facet gives `Range` `Def::Scalar`, although it is a user struct, so it is referred to as the
/// container it is registered as, rather than as a scalar (crux's `notes` example).
#[test]
fn a_range_in_an_enum_newtype_variant_is_a_container() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Event {
        Selection(std::ops::Range<usize>),
    }

    insta::assert_yaml_snapshot!(reflect!(Event).unwrap(), @"
    ? namespace: ROOT
      name: Event
    : ENUM:
        - 0:
            Selection:
              - NEWTYPE:
                  TYPENAME:
                    namespace: ROOT
                    name: Range
              - []
        - EXTERNAL
        - []
    ? namespace: ROOT
      name: Range
    : STRUCT:
        - - start:
              - U64
              - []
          - end:
              - U64
              - []
        - []
    ");
}

/// The std types that facet exposes as user structs with `Def::Scalar` are containers from every
/// position a reference can be in, not only the direct ones.
#[test]
fn std_user_structs_with_a_scalar_def_are_containers_everywhere() {
    macro_rules! positions {
        ($($name:ident: $t:ty => $container:literal),* $(,)?) => {$(
            mod $name {
                #![allow(dead_code)]
                use super::*;

                #[derive(Facet)]
                pub struct Holder {
                    pub direct: $t,
                    pub option: Option<$t>,
                    pub seq: Vec<Option<$t>>,
                    pub map: BTreeMap<String, $t>,
                    pub tuple: ($t, u8),
                }

                #[derive(Facet)]
                pub struct NewType(pub $t);

                #[derive(Facet)]
                pub struct Pair(pub u8, pub Vec<$t>);

                #[derive(Facet)]
                #[repr(C)]
                pub enum Event {
                    NewType($t),
                    Option(Option<$t>),
                    Tuple(Vec<$t>, u8),
                    Struct { value: Option<$t> },
                }

                #[derive(Facet)]
                pub struct App {
                    pub holder: Holder,
                    pub new_type: NewType,
                    pub pair: Pair,
                    pub event: Event,
                }
            }

            {
                use $name::App;
                let registry =
                    reflect!(App).unwrap_or_else(|err| panic!("{}: {err}", stringify!($t)));
                let references = references_to(&registry, $container);
                assert_eq!(references.len(), 11, "{}: {references:#?}", stringify!($t));
                assert!(
                    registry.contains_key(&QualifiedTypeName::root($container.to_string())),
                    "{} is not registered",
                    $container,
                );
            }
        )*};
    }

    positions!(
        range: std::ops::Range<u32> => "Range",
        phantom: std::marker::PhantomData<u8> => "PhantomData",
        infallible: std::convert::Infallible => "Infallible",
    );
}

/// The error from reflecting `T`, which must be an [`Error::UnsupportedFieldType`], as
/// `[container, field, field type, unsupported type]`.
fn unsupported_field<'a, T: Facet<'a>>() -> [String; 4] {
    match RegistryBuilder::new().add_type::<T>() {
        Err(Error::UnsupportedFieldType {
            container,
            field,
            field_type,
            unsupported,
        }) => [container, field, field_type, unsupported],
        Err(err) => panic!("unexpected error: {err:?}"),
        Ok(_) => panic!("reflection unexpectedly succeeded"),
    }
}

fn strings<const N: usize>(parts: [&str; N]) -> [String; N] {
    parts.map(ToString::to_string)
}

/// A type reflection doesn't support, such as `Result`, is an error rather than being left out
/// (#235): here, as an element of a newtype, a tuple struct or a tuple variant.
#[test]
fn an_unsupported_type_in_a_tuple_position_is_an_error() {
    #[derive(Facet)]
    pub struct Wrapper(pub std::time::Duration);

    #[derive(Facet)]
    pub struct Pair(pub u8, pub Result<u32, String>);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Event {
        Tuple(u8, Result<u32, String>),
    }

    assert_eq!(
        unsupported_field::<Wrapper>(),
        strings(["Wrapper", "0", "Duration", "Duration"]),
    );
    assert_eq!(
        unsupported_field::<Pair>(),
        strings(["Pair", "1", "Result<u32, String>", "Result<u32, String>"]),
    );
    assert_eq!(
        unsupported_field::<Event>(),
        strings([
            "Event::Tuple",
            "1",
            "Result<u32, String>",
            "Result<u32, String>"
        ]),
    );
}

/// A payload of an unsupported scalar, optional or not, is an error rather than making a unit
/// variant.
#[test]
fn an_unsupported_scalar_payload_is_an_error() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Wait {
        Wait(Option<std::time::Duration>),
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Pause {
        Pause(std::time::Duration),
    }

    assert_eq!(
        unsupported_field::<Wait>(),
        strings(["Wait::Wait", "0", "Option<Duration>", "Duration"]),
    );
    assert_eq!(
        unsupported_field::<Pause>(),
        strings(["Pause::Pause", "0", "Duration", "Duration"]),
    );
}

/// An array in a tuple position is one element, not the array followed by its element type.
#[test]
fn an_array_in_a_tuple_position_is_one_element() {
    #[derive(Facet)]
    pub struct Leaf {
        pub id: u32,
    }

    #[derive(Facet)]
    pub struct Pair(pub u8, pub [Leaf; 2]);

    let registry = reflect!(Pair).unwrap();
    let leaves = Format::TupleArray {
        content: Box::new(Format::TypeName(QualifiedTypeName::root(
            "Leaf".to_string(),
        ))),
        size: 2,
    };
    assert_eq!(
        registry[&QualifiedTypeName::root("Pair".to_string())],
        ContainerFormat::TupleStruct(vec![Format::U8, leaves], Doc::default()),
    );
}

/// A field whose type contains an unsupported type such as `Result`, however it is wrapped, is an
/// error, in a struct and in a struct variant, naming the container, the field, its type and the
/// unsupported type within it.
#[test]
fn an_unsupported_type_anywhere_in_a_field_is_an_error() {
    macro_rules! field_of_type {
        ($($name:ident: $ty:ty => $field_type:literal, $unsupported:literal;)*) => {$(
            #[derive(Facet)]
            pub struct $name {
                pub kept: u8,
                pub field: $ty,
            }
            assert_eq!(
                unsupported_field::<$name>(),
                strings([stringify!($name), "field", $field_type, $unsupported]),
            );
        )*};
    }

    field_of_type!(
        Seq: Vec<Result<u8, String>> => "Vec<Result<u8, String>>", "Result<u8, String>";
        Opt: Option<Result<u8, String>> => "Option<Result<u8, String>>", "Result<u8, String>";
        Tuple: (u8, Result<u8, String>) => "(u8, Result<u8, String>)", "Result<u8, String>";
        Map: HashMap<String, std::time::Duration> => "HashMap<String, Duration>", "Duration";
        Url: url::Url => "Url", "Url";
        Date: chrono::NaiveDate => "NaiveDate", "NaiveDate";
        Offset: chrono::DateTime<chrono::FixedOffset> => "DateTime<FixedOffset>", "DateTime<FixedOffset>";
        Path: std::path::PathBuf => "PathBuf", "PathBuf";
    );

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Event {
        Struct {
            kept: u8,
            seq: Vec<Result<u8, String>>,
        },
    }

    assert_eq!(
        unsupported_field::<Event>(),
        strings([
            "Event::Struct",
            "seq",
            "Vec<Result<u8, String>>",
            "Result<u8, String>"
        ]),
    );
}

/// The error for an unsupported field says what to do about it.
#[test]
fn an_unsupported_field_error_message() {
    #[derive(Facet)]
    pub struct Timeout {
        pub after: Option<std::time::Duration>,
    }

    let err = RegistryBuilder::new().add_type::<Timeout>().unwrap_err();
    assert_eq!(
        err.to_string(),
        "field `after` of `Timeout` has type `Option<Duration>`, which can't be generated because \
         `Duration` is not supported. Mark the field `#[facet(skip)]` or `#[facet(opaque)]` to \
         leave it out, or change its type",
    );
}

/// A field of an unsupported type is still left out, without an error, if it is marked
/// `#[facet(skip)]` or `#[facet(opaque)]`, in a struct and in every kind of variant. A
/// `PhantomData` field is not unsupported: it is a unit struct.
#[test]
fn a_skipped_or_opaque_field_of_an_unsupported_type_is_left_out() {
    #[derive(Facet)]
    pub struct Holder {
        pub kept: u8,
        #[facet(skip)]
        pub skipped: std::time::Duration,
        #[facet(opaque)]
        pub opaque: Result<u8, String>,
        pub phantom: std::marker::PhantomData<u8>,
    }

    #[derive(Facet)]
    pub struct Pair(pub u8, #[facet(opaque)] pub std::time::Duration);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Event {
        Newtype(#[facet(opaque)] std::time::Duration),
        Tuple(u8, #[facet(opaque)] Result<u8, String>),
        Struct {
            kept: u8,
            #[facet(skip)]
            skipped: std::time::Duration,
            #[facet(opaque)]
            opaque: url::Url,
        },
    }

    let registry = reflect!(Holder, Pair, Event).unwrap();
    let ContainerFormat::Struct(fields, _) =
        &registry[&QualifiedTypeName::root("Holder".to_string())]
    else {
        panic!("not a struct");
    };
    let names: Vec<_> = fields.iter().map(|field| field.name.as_str()).collect();
    assert_eq!(names, ["kept", "phantom"]);
    assert_eq!(
        registry[&QualifiedTypeName::root("Pair".to_string())],
        ContainerFormat::TupleStruct(vec![Format::U8], Doc::default()),
    );

    let ContainerFormat::Enum(variants, _, _) =
        &registry[&QualifiedTypeName::root("Event".to_string())]
    else {
        panic!("not an enum");
    };
    assert_eq!(variants[&0].value, VariantFormat::Unit);
    assert_eq!(variants[&1].value, VariantFormat::Tuple(vec![Format::U8]));
    let VariantFormat::Struct(fields) = &variants[&2].value else {
        panic!("not a struct variant");
    };
    let names: Vec<_> = fields.iter().map(|field| field.name.as_str()).collect();
    assert_eq!(names, ["kept"]);
}

/// An untagged enum is rejected (#234), whether it is added itself or reached through a field,
/// rather than generated as externally tagged.
#[test]
fn an_untagged_enum_is_an_error() {
    #[derive(Facet)]
    #[facet(untagged)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Value {
        Number(u32),
        Text(String),
    }

    #[derive(Facet)]
    pub struct Holder {
        pub value: Vec<Value>,
    }

    let err = RegistryBuilder::new().add_type::<Value>().unwrap_err();
    assert_eq!(err, Error::UntaggedEnum("Value".to_string()));
    assert_eq!(
        err.to_string(),
        r#"enum `Value` is `#[facet(untagged)]`, which is not supported. Use an externally, internally (`#[facet(tag = "...")]`) or adjacently (`#[facet(tag = "...", content = "...")]`) tagged representation"#,
    );

    let err = RegistryBuilder::new().add_type::<Holder>().unwrap_err();
    assert_eq!(err, Error::UntaggedEnum("Value".to_string()));
}

/// Any other error naming a field's type fails as itself, not as an unsupported field.
#[test]
fn an_error_naming_a_field_type_is_not_an_unsupported_field() {
    #[derive(Facet)]
    #[facet(fg::namespace = "one")]
    pub struct Holder {
        pub leaves: Vec<Leaf>,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "two")]
    pub struct Other {
        pub leaf: Leaf,
    }

    #[derive(Facet)]
    pub struct Leaf {
        pub id: u32,
    }

    let err = RegistryBuilder::new()
        .add_type::<Holder>()
        .unwrap()
        .add_type::<Other>()
        .unwrap_err();
    assert!(
        matches!(err, Error::AmbiguousNamespaceInheritance { .. }),
        "{err:?}"
    );
}

/// A chain of transparent wrappers is referred to as the type it finally wraps, wherever the
/// reference is.
#[test]
fn a_chain_of_transparent_wrappers_is_the_type_it_wraps() {
    #[derive(Facet)]
    #[facet(transparent)]
    pub struct Inner(u32);

    #[derive(Facet)]
    #[facet(transparent)]
    pub struct Outer(Inner);

    #[derive(Facet)]
    pub struct Leaf {
        pub id: u32,
    }

    #[derive(Facet)]
    #[facet(transparent)]
    pub struct LeafInner(Leaf);

    #[derive(Facet)]
    #[facet(transparent)]
    pub struct LeafOuter(LeafInner);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Event {
        Scalar(Outer),
        Leaf(LeafOuter),
    }

    #[derive(Facet)]
    pub struct Holder {
        pub direct: Outer,
        pub seq: Vec<Outer>,
        pub leaves: Vec<LeafOuter>,
    }

    let registry = reflect!(Event, Holder).unwrap();
    let leaf = || Format::TypeName(QualifiedTypeName::root("Leaf".to_string()));

    let ContainerFormat::Enum(variants, _, _) =
        &registry[&QualifiedTypeName::root("Event".to_string())]
    else {
        panic!("not an enum");
    };
    assert_eq!(
        variants[&0].value,
        VariantFormat::NewType(Box::new(Format::U32))
    );
    assert_eq!(variants[&1].value, VariantFormat::NewType(Box::new(leaf())));

    let ContainerFormat::Struct(fields, _) =
        &registry[&QualifiedTypeName::root("Holder".to_string())]
    else {
        panic!("not a struct");
    };
    let formats: Vec<_> = fields.iter().map(|field| field.value.clone()).collect();
    assert_eq!(
        formats,
        [
            Format::U32,
            Format::Seq(Box::new(Format::U32)),
            Format::Seq(Box::new(leaf())),
        ]
    );
}

/// A renamed type moved into a namespace by a field's `fg::namespace` is registered, and referred
/// to, under its new name, from a struct field and from a struct variant's field.
#[test]
fn a_renamed_type_in_a_field_namespace_keeps_its_name() {
    #[derive(Facet)]
    #[facet(rename = "Renamed")]
    pub struct Original {
        pub id: u32,
    }

    #[derive(Facet)]
    pub struct Holder {
        #[facet(fg::namespace = "x")]
        pub over: Original,
    }

    #[derive(Facet)]
    #[facet(rename = "RenamedToo")]
    pub struct OriginalToo {
        pub id: u32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Event {
        Moved {
            #[facet(fg::namespace = "x")]
            over: OriginalToo,
        },
    }

    let registry = reflect!(Holder, Event).unwrap();
    let renamed = QualifiedTypeName::namespaced("x".to_string(), "Renamed".to_string());
    let renamed_too = QualifiedTypeName::namespaced("x".to_string(), "RenamedToo".to_string());
    assert!(registry.contains_key(&renamed), "{:#?}", registry.keys());
    assert!(
        registry.contains_key(&renamed_too),
        "{:#?}",
        registry.keys()
    );
    assert!(
        !registry
            .keys()
            .any(|key| key.name == "Original" || key.name == "OriginalToo"),
        "{:#?}",
        registry.keys()
    );
    assert_eq!(
        references_to(&registry, "Renamed"),
        [(
            "ROOT::Holder".to_string(),
            "over".to_string(),
            renamed.clone()
        )]
    );
    assert_eq!(
        references_to(&registry, "RenamedToo"),
        [(
            "ROOT::Event".to_string(),
            "Moved.over".to_string(),
            renamed_too.clone()
        )]
    );
}

/// A transparent wrapper's own namespace is the context for the type it wraps in every position,
/// as it is for a direct field, so that type is registered once, in the wrapper's namespace.
#[test]
fn a_transparent_wrappers_namespace_applies_in_every_position() {
    #[derive(Facet)]
    pub struct Leaf {
        pub id: u32,
    }

    #[derive(Facet)]
    #[facet(transparent, fg::namespace = "wrappers")]
    pub struct Wrapper(Leaf);

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    pub enum Payload {
        NewType(Wrapper),
        Tuple(Wrapper, u8),
        Struct { wrapper: Wrapper },
    }

    #[derive(Facet)]
    pub struct RootHolder {
        pub direct: Wrapper,
        pub seq: Vec<Wrapper>,
        pub option: Option<Wrapper>,
        pub map: HashMap<String, Wrapper>,
        pub tuple: (Wrapper, u8),
        pub payload: Payload,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "named")]
    pub struct NamedHolder {
        pub direct: Wrapper,
        pub seq: Vec<Wrapper>,
        pub option: Option<Wrapper>,
    }

    let registry = reflect!(RootHolder, NamedHolder).unwrap();
    let expected = QualifiedTypeName::namespaced("wrappers".to_string(), "Leaf".to_string());
    let registered: Vec<_> = registry.keys().filter(|key| key.name == "Leaf").collect();
    assert_eq!(registered, [&expected]);

    let references = references_to(&registry, "Leaf");
    let wrong: Vec<_> = references
        .iter()
        .filter(|(_, _, name)| name != &expected)
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
    assert_eq!(references.len(), 11, "{references:#?}");

    // `format_of` names it the same way.
    let builder = RegistryBuilder::new().add_type::<RootHolder>().unwrap();
    assert_eq!(
        builder.format_of::<Vec<Wrapper>>().unwrap(),
        Format::Seq(Box::new(Format::TypeName(expected)))
    );
}
