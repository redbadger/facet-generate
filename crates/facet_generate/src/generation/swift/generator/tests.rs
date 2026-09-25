//! Unit tests for [`SwiftCodeGenerator`] — import generation and qualified-name
//! resolution.
//!
//! Tests build small [`Registry`] values by hand (rather than via the
//! `reflect!` macro) so that module and external-package configurations can
//! be controlled precisely.
//!
//! # Coverage
//!
//! | Area | What is tested |
//! |------|----------------|
//! | Serde imports | `BincodePlugin` triggers `import Serde`; no plugin does not |
//! | External definitions | External namespaces appear as `import` statements |
//! | Plugin config | Plugins propagate through to generated output |
//! | Feature helpers | Complex types (e.g. `Seq`) trigger trait helper emission when a plugin is active |

use std::collections::BTreeMap;

use std::sync::Arc;

use crate::{
    generation::{CodeGeneratorConfig, bincode::BincodePlugin, plugin::EmitterPlugin},
    reflection::format::{
        ContainerFormat, Doc, EnumTagging, Format, Named, Namespace, QualifiedTypeName,
        VariantFormat,
    },
};

use super::*;
use crate::generation::{
    ExternalPackages,
    swift::{conformance::Conformance, emitter::Swift},
};

fn generate(
    config: &CodeGeneratorConfig,
    plugins: Vec<Arc<dyn EmitterPlugin<Swift>>>,
    registry: &Registry,
) -> String {
    let generator = SwiftCodeGenerator::new(config).with_plugins(plugins);
    let mut output = Vec::new();
    generator.output(&mut output, registry).unwrap();
    String::from_utf8(output).unwrap()
}

#[test]
fn test_no_encoding_does_not_import_serde() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "id".to_string(),
        doc: Doc::new(),
        value: Format::U32,
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, vec![], &registry);

    assert!(
        !output.contains("import Serde"),
        "Should not import Serde when encoding is None: {output}"
    );
}

#[test]
fn test_bincode_encoding_has_serde_import() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "name".to_string(),
        doc: Doc::new(),
        value: Format::Str,
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("import Serde"),
        "Bincode encoding should import Serde: {output}"
    );
}

#[test]
fn test_preamble_includes_external_definition_imports() {
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("another_target".to_string(), vec!["Child".to_string()]);

    let config = CodeGeneratorConfig::new("MyPackage".to_string())
        .with_external_definitions(external_definitions);

    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::UnitStruct(Doc::new()),
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("import AnotherTarget"),
        "Should import UpperCamelCase external definitions: {output}"
    );
    assert!(
        output.contains("import Serde"),
        "Should always import Serde when encoding is set: {output}"
    );
}

#[test]
fn test_trait_helpers_emitted_for_complex_types() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "items".to_string(),
        doc: Doc::new(),
        value: Format::Seq(Box::new(Format::Str)),
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("func serializeArray<T, S: Serializer>"),
        "Should emit generic array serialization helper: {output}"
    );
    assert!(
        output.contains("func deserializeArray<T, D: Deserializer>"),
        "Should emit generic array deserialization helper: {output}"
    );
}

#[test]
fn test_no_trait_helpers_without_encoding() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "items".to_string(),
        doc: Doc::new(),
        value: Format::Seq(Box::new(Format::Str)),
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, vec![], &registry);

    assert!(
        !output.contains("func serializeArray"),
        "No encoding means no trait helpers: {output}"
    );
    assert!(
        !output.contains("func deserializeArray"),
        "No encoding means no trait helpers: {output}"
    );
}

#[test]
fn test_map_with_hashable_k_v_implement_hashable_and_equatable() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "items".to_string(),
        doc: Doc::new(),
        value: Format::Map {
            key: Box::new(Format::Str),
            value: Box::new(Format::Str),
        },
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("public struct MyStruct: Hashable, Equatable {"),
        "Struct is not hashable and equatable:\n{output}"
    );
}

#[test]
fn test_struct_with_hashable_scalar_implements_hashable_and_equatable() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "items".to_string(),
        doc: Doc::new(),
        value: Format::U8,
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("public struct MyStruct: Hashable, Equatable {"),
        "Struct is not hashable and equatable:\n{output}"
    );
}

#[test]
fn test_struct_with_hashable_map_implements_hashable_and_equatable() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "items".to_string(),
        doc: Doc::new(),
        value: Format::Map {
            key: Box::new(Format::Str),
            value: Box::new(Format::Str),
        },
    }];
    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("public struct MyStruct: Hashable, Equatable {"),
        "Struct is not hashable and equatable:\n{output}"
    );
}

#[test]
fn test_enum_with_hashable_variants_implements_hashable_and_equatable() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();
    let mut variants = BTreeMap::new();

    let str_variant = Named {
        name: "StrVariant".to_string(),
        doc: Doc::new(),
        value: VariantFormat::NewType(Box::new(Format::Str)),
    };

    variants.insert(0, str_variant);

    let map_variant = Named {
        name: "MapVariant".to_string(),
        doc: Doc::new(),
        value: VariantFormat::NewType(Box::new(Format::Map {
            key: Box::new(Format::Str),
            value: Box::new(Format::Str),
        })),
    };
    variants.insert(1, map_variant);

    registry.insert(
        QualifiedTypeName::root("MyEnum".to_string()),
        ContainerFormat::Enum(variants, EnumTagging::External, Doc::new()),
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("indirect public enum MyEnum: Hashable, Equatable {"),
        "MyEnum is not hashable:\n{output}"
    );
}

#[test]
fn test_with_enum_and_struct_variant_implements_hashable_and_equatable() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();

    let fields = vec![
        Named {
            name: "str_item".to_string(),
            doc: Doc::new(),
            value: Format::Str,
        },
        Named {
            name: "str_map".to_string(),
            doc: Doc::new(),
            value: Format::Map {
                key: Box::new(Format::Str),
                value: Box::new(Format::Str),
            },
        },
        Named {
            name: "str_option".to_string(),
            doc: Doc::new(),
            value: Format::Option(Box::new(Format::Str)),
        },
    ];

    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let mut variants = BTreeMap::new();

    let str_variant = Named {
        name: "StrVariant".to_string(),
        doc: Doc::new(),
        value: VariantFormat::NewType(Box::new(Format::Str)),
    };

    variants.insert(0, str_variant);

    let struct_variant = Named {
        name: "StructVariant".to_string(),
        doc: Doc::new(),
        value: VariantFormat::NewType(Box::new(Format::TypeName(QualifiedTypeName {
            namespace: Namespace::Root,
            name: "MyStruct".to_string(),
        }))),
    };
    variants.insert(1, struct_variant);

    registry.insert(
        QualifiedTypeName::root("MyEnum".to_string()),
        ContainerFormat::Enum(variants, EnumTagging::External, Doc::new()),
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("indirect public enum MyEnum: Hashable, Equatable {"),
        "MyEnum is not hashable:\n{output}"
    );
}

#[test]
fn test_type_cycle_is_hashable_and_equatable() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let mut registry = Registry::new();

    let fields = vec![Named {
        name: "str_item".to_string(),
        doc: Doc::new(),
        value: Format::Str,
    }];

    registry.insert(
        QualifiedTypeName::root("MyStruct".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );

    let mut variants = BTreeMap::new();

    let struct_variant = Named {
        name: "StructVariant".to_string(),
        doc: Doc::new(),
        value: VariantFormat::NewType(Box::new(Format::TypeName(QualifiedTypeName {
            namespace: Namespace::Root,
            name: "MyStruct".to_string(),
        }))),
    };

    let self_enum_variant = Named {
        name: "EnumVariant".to_string(),
        doc: Doc::new(),
        value: VariantFormat::NewType(Box::new(Format::TypeName(QualifiedTypeName {
            namespace: Namespace::Root,
            name: "MyEnum".to_string(),
        }))),
    };
    variants.insert(0, struct_variant);
    variants.insert(1, self_enum_variant);

    registry.insert(
        QualifiedTypeName::root("MyEnum".to_string()),
        ContainerFormat::Enum(variants, EnumTagging::External, Doc::new()),
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("indirect public enum MyEnum: Hashable, Equatable {"),
        "MyEnum is not hashable:\n{output}"
    );
}

#[test]
fn test_struct_declaration_easy_order() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());
    let fields = vec![
        Named {
            name: "items".to_string(),
            doc: Doc::new(),
            value: Format::Map {
                key: Box::new(Format::Str),
                value: Box::new(Format::Str),
            },
        },
        // MyStruct2, it will be added to the registry before MyStruct1
        Named {
            name: "struct_2".to_string(),
            doc: Doc::new(),
            value: Format::TypeName(QualifiedTypeName {
                namespace: Namespace::Root,
                name: "MyStruct2".to_string(),
            }),
        },
    ];
    let struct1 = ContainerFormat::Struct(fields.clone(), Doc::new());
    let struct2 = ContainerFormat::Struct(fields, Doc::new());

    let mut registry = Registry::new();

    registry.insert(QualifiedTypeName::root("MyStruct2".to_string()), struct2);
    registry.insert(QualifiedTypeName::root("MyStruct1".to_string()), struct1);

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("public struct MyStruct1: Hashable, Equatable {"),
        "MyStruct1 is not hashable and equatable:\n{output}"
    );
    assert!(
        output.contains("public struct MyStruct2: Hashable, Equatable {"),
        "MyStruct2 is not hashable and equatable:\n{output}"
    );
}

#[test]
fn test_struct_declaration_inv_order() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());
    let fields = vec![
        Named {
            name: "items".to_string(),
            doc: Doc::new(),
            value: Format::Map {
                key: Box::new(Format::Str),
                value: Box::new(Format::Str),
            },
        },
        // MyStruct2, will be added to the registry after MyStruct1
        Named {
            name: "struct_2".to_string(),
            doc: Doc::new(),
            value: Format::TypeName(QualifiedTypeName {
                namespace: Namespace::Root,
                name: "MyStruct2".to_string(),
            }),
        },
    ];
    let struct1 = ContainerFormat::Struct(fields.clone(), Doc::new());
    let struct2 = ContainerFormat::Struct(fields, Doc::new());

    let mut registry = Registry::new();

    registry.insert(QualifiedTypeName::root("MyStruct1".to_string()), struct1);
    registry.insert(QualifiedTypeName::root("MyStruct2".to_string()), struct2);

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("public struct MyStruct1: Hashable, Equatable {"),
        "MyStruct1 is not hashable and equatable:\n{output}"
    );
    assert!(
        output.contains("public struct MyStruct2: Hashable, Equatable {"),
        "MyStruct2 is not hashable and equatable:\n{output}"
    );
}

#[test]
#[allow(clippy::similar_names)]
fn test_mutual_recursion_equatable() {
    let config = CodeGeneratorConfig::new("MyPackage".to_string());

    let struct_a_fields = vec![
        Named {
            name: "value".to_string(),
            doc: Doc::new(),
            value: Format::U32,
        },
        Named {
            name: "other".to_string(),
            doc: Doc::new(),
            value: Format::TypeName(QualifiedTypeName {
                namespace: Namespace::Root,
                name: "StructB".to_string(),
            }),
        },
    ];

    let struct_b_fields = vec![
        Named {
            name: "value".to_string(),
            doc: Doc::new(),
            value: Format::U32,
        },
        Named {
            name: "other".to_string(),
            doc: Doc::new(),
            value: Format::TypeName(QualifiedTypeName {
                namespace: Namespace::Root,
                name: "StructA".to_string(),
            }),
        },
    ];

    let struct_a = ContainerFormat::Struct(struct_a_fields, Doc::new());
    let struct_b = ContainerFormat::Struct(struct_b_fields, Doc::new());

    let mut registry = Registry::new();
    registry.insert(QualifiedTypeName::root("StructA".to_string()), struct_a);
    registry.insert(QualifiedTypeName::root("StructB".to_string()), struct_b);

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("public struct StructA: Hashable, Equatable {"),
        "StructA should be hashable and equatable:\n{output}"
    );
    assert!(
        output.contains("public struct StructB: Hashable, Equatable {"),
        "StructB should be hashable and equatable:\n{output}"
    );
}

#[test]
fn test_named_namespace_map_key_value_are_hashable() {
    let config = CodeGeneratorConfig::new("other".to_string());

    let child = ContainerFormat::Struct(
        vec![Named {
            name: "value".to_string(),
            doc: Doc::new(),
            value: Format::Map {
                key: Box::new(Format::TypeName(QualifiedTypeName::namespaced(
                    "other".to_string(),
                    "Key".to_string(),
                ))),
                value: Box::new(Format::TypeName(QualifiedTypeName::namespaced(
                    "other".to_string(),
                    "Value".to_string(),
                ))),
            },
        }],
        Doc::new(),
    );
    let key = ContainerFormat::NewTypeStruct(Box::new(Format::Str), Doc::new());
    let value = ContainerFormat::NewTypeStruct(Box::new(Format::I32), Doc::new());

    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Child".to_string()),
        child,
    );
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Key".to_string()),
        key,
    );
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Value".to_string()),
        value,
    );

    let output = generate(&config, vec![], &registry);

    assert!(
        output.contains("public var value: [Key: Value]"),
        "same-module map key/value should render with bare names and not be rejected as non-Hashable:\n{output}"
    );
}

#[test]
fn test_named_namespace_struct_implements_hashable_and_equatable() {
    let config = CodeGeneratorConfig::new("other".to_string());

    let parent = ContainerFormat::Struct(
        vec![Named {
            name: "child".to_string(),
            doc: Doc::new(),
            value: Format::TypeName(QualifiedTypeName::namespaced(
                "other".to_string(),
                "Child".to_string(),
            )),
        }],
        Doc::new(),
    );
    let child = ContainerFormat::NewTypeStruct(Box::new(Format::Str), Doc::new());

    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Parent".to_string()),
        parent,
    );
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Child".to_string()),
        child,
    );

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("public struct Parent: Hashable, Equatable {"),
        "Parent referencing a same-module Hashable type should be Hashable, Equatable:\n{output}"
    );
    assert!(
        output.contains("public struct Child: Hashable, Equatable {"),
        "Child in a named namespace should be Hashable, Equatable:\n{output}"
    );
}

/// Builds the post-split `other` registry: `Child { value: [Key: Value] }`
/// with `Key(String)` and `Value(i32)`, all in namespace `other`.
fn other_namespace_map_registry() -> Registry {
    let child = ContainerFormat::Struct(
        vec![Named {
            name: "value".to_string(),
            doc: Doc::new(),
            value: Format::Map {
                key: Box::new(Format::TypeName(QualifiedTypeName::namespaced(
                    "other".to_string(),
                    "Key".to_string(),
                ))),
                value: Box::new(Format::TypeName(QualifiedTypeName::namespaced(
                    "other".to_string(),
                    "Value".to_string(),
                ))),
            },
        }],
        Doc::new(),
    );
    let key = ContainerFormat::NewTypeStruct(Box::new(Format::Str), Doc::new());
    let value = ContainerFormat::NewTypeStruct(Box::new(Format::I32), Doc::new());

    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Child".to_string()),
        child,
    );
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Key".to_string()),
        key,
    );
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Value".to_string()),
        value,
    );
    registry
}

#[test]
fn test_compute_hashable_types_covers_named_namespace() {
    let registry = other_namespace_map_registry();

    let hashable = Conformance::of(&registry, &ExternalPackages::new()).hashable;

    for name in ["Child", "Key", "Value"] {
        assert!(
            hashable.contains(&QualifiedTypeName::namespaced(
                "other".to_string(),
                name.to_string()
            )),
            "other::{name} should be computed as hashable, got {hashable:?}"
        );
    }
}

#[test]
fn test_compute_hashable_types_is_collision_safe() {
    let root_child = ContainerFormat::Struct(
        vec![Named {
            name: "x".to_string(),
            doc: Doc::new(),
            value: Format::U32,
        }],
        Doc::new(),
    );
    // A multi-element native tuple is not Hashable in Swift.
    let other_child = ContainerFormat::Struct(
        vec![Named {
            name: "pair".to_string(),
            doc: Doc::new(),
            value: Format::Tuple(vec![Format::U32, Format::U32]),
        }],
        Doc::new(),
    );

    let mut registry = Registry::new();
    registry.insert(QualifiedTypeName::root("Child".to_string()), root_child);
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Child".to_string()),
        other_child,
    );

    let hashable = Conformance::of(&registry, &ExternalPackages::new()).hashable;

    assert!(
        hashable.contains(&QualifiedTypeName::root("Child".to_string())),
        "root Child (scalar field) should be hashable: {hashable:?}"
    );
    assert!(
        !hashable.contains(&QualifiedTypeName::namespaced(
            "other".to_string(),
            "Child".to_string()
        )),
        "other::Child (tuple field) should not be hashable; same bare name must not leak: {hashable:?}"
    );
}

#[test]
fn test_compute_hashable_types_assumes_external_hashable() {
    let parent = ContainerFormat::Struct(
        vec![Named {
            name: "m".to_string(),
            doc: Doc::new(),
            value: Format::Map {
                key: Box::new(Format::TypeName(QualifiedTypeName::namespaced(
                    "external".to_string(),
                    "Key".to_string(),
                ))),
                value: Box::new(Format::Str),
            },
        }],
        Doc::new(),
    );

    let mut registry = Registry::new();
    registry.insert(QualifiedTypeName::root("Parent".to_string()), parent);

    let hashable = Conformance::of(&registry, &ExternalPackages::new()).hashable;

    assert!(
        hashable.contains(&QualifiedTypeName::root("Parent".to_string())),
        "Parent keyed by an absent external type should be hashable: {hashable:?}"
    );
}

#[test]
fn test_non_hashable_local_named_map_key_is_rejected() {
    let config = CodeGeneratorConfig::new("other".to_string());

    let child = ContainerFormat::Struct(
        vec![Named {
            name: "m".to_string(),
            doc: Doc::new(),
            value: Format::Map {
                key: Box::new(Format::TypeName(QualifiedTypeName::namespaced(
                    "other".to_string(),
                    "Bad".to_string(),
                ))),
                value: Box::new(Format::Str),
            },
        }],
        Doc::new(),
    );
    let bad = ContainerFormat::Struct(
        vec![Named {
            name: "pair".to_string(),
            doc: Doc::new(),
            value: Format::Tuple(vec![Format::U32, Format::U32]),
        }],
        Doc::new(),
    );

    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Child".to_string()),
        child,
    );
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Bad".to_string()),
        bad,
    );

    let generator = SwiftCodeGenerator::new(&config);
    let mut output = Vec::new();
    let result = generator.output(&mut output, &registry);

    assert!(
        result.is_err(),
        "a local Named map key that is not Hashable must be rejected"
    );
}

#[test]
fn test_self_referential_named_type_is_hashable() {
    let node = ContainerFormat::Struct(
        vec![Named {
            name: "next".to_string(),
            doc: Doc::new(),
            value: Format::Option(Box::new(Format::TypeName(QualifiedTypeName::namespaced(
                "other".to_string(),
                "Node".to_string(),
            )))),
        }],
        Doc::new(),
    );

    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::namespaced("other".to_string(), "Node".to_string()),
        node,
    );

    let hashable = Conformance::of(&registry, &ExternalPackages::new()).hashable;

    assert!(
        hashable.contains(&QualifiedTypeName::namespaced(
            "other".to_string(),
            "Node".to_string()
        )),
        "a self-referential Named type should be hashable via cycle optimism: {hashable:?}"
    );
}

#[test]
fn test_named_namespace_map_struct_implements_hashable_with_plugin() {
    let config = CodeGeneratorConfig::new("other".to_string());
    let registry = other_namespace_map_registry();

    let output = generate(&config, vec![Arc::new(BincodePlugin)], &registry);

    assert!(
        output.contains("public struct Child: Hashable, Equatable {"),
        "Child with a [Key: Value] map of same-module Hashable types should be Hashable, Equatable:\n{output}"
    );
    assert!(
        output.contains("public struct Key: Hashable, Equatable {"),
        "Key should be Hashable, Equatable:\n{output}"
    );
    assert!(
        output.contains("public struct Value: Hashable, Equatable {"),
        "Value should be Hashable, Equatable:\n{output}"
    );
}

#[test]
fn test_named_namespace_map_struct_omits_conformance_without_plugin() {
    let config = CodeGeneratorConfig::new("other".to_string());
    let registry = other_namespace_map_registry();

    let output = generate(&config, vec![], &registry);

    // Conformance display is intentionally gated on an active serialization
    // plugin: plain generation emits bare declarations even for types that
    // qualify. Changing this should be a deliberate decision, not silent drift.
    assert!(
        output.contains("public struct Child {"),
        "without a plugin, Child should render bare (no conformance suffix):\n{output}"
    );
    assert!(
        !output.contains("public struct Child: "),
        "without a plugin, Child should not declare any conformance:\n{output}"
    );
}

// ---------------------------------------------------------------------------
// Reserved-name pre-pass
// ---------------------------------------------------------------------------

#[test]
fn type_named_serializer_is_rejected() {
    #[derive(facet::Facet)]
    #[facet(rename = "Serializer")]
    struct Renamed {
        name: String,
    }

    let registry = crate::reflect!(Renamed).unwrap();
    let cfg = CodeGeneratorConfig::new("Testing".to_string());
    let err = SwiftCodeGenerator::new(&cfg)
        .output(&mut Vec::new(), &registry)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        err.to_string(),
        "Swift: type `Serializer` collides with the runtime protocol `Serde.Serializer` used by the generated code; rename it with #[facet(rename = \"...\")]"
    );
}

#[test]
fn field_named_serializer_is_rejected() {
    #[derive(facet::Facet)]
    struct Foo {
        serializer: String,
    }

    let registry = crate::reflect!(Foo).unwrap();
    let cfg = CodeGeneratorConfig::new("Testing".to_string());
    let err = SwiftCodeGenerator::new(&cfg)
        .output(&mut Vec::new(), &registry)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        err.to_string(),
        "Swift: field `serializer` of `Foo` would become `serializer`, which shadows the `serializer` parameter in the generated serialize method; rename it with #[facet(rename = \"...\")]"
    );
}

// ---------------------------------------------------------------------------
// References to ROOT types
// ---------------------------------------------------------------------------

/// A `kv` struct holding the ROOT `Shared` and `Presence`, and the `kv`
/// `Presence` that shares the ROOT enum's name.
fn kv_registry() -> Registry {
    let field = |name: &str, qtn: QualifiedTypeName| Named {
        name: name.to_string(),
        doc: Doc::new(),
        value: Format::TypeName(qtn),
    };
    let kv = |name: &str| QualifiedTypeName::namespaced("kv".to_string(), name.to_string());

    let mut registry = Registry::new();
    registry.insert(
        kv("Entry"),
        ContainerFormat::Struct(
            vec![
                field("shared", QualifiedTypeName::root("Shared".to_string())),
                field("status", QualifiedTypeName::root("Presence".to_string())),
                field("local", kv("Presence")),
            ],
            Doc::new(),
        ),
    );
    registry.insert(
        kv("Presence"),
        ContainerFormat::Struct(
            vec![Named {
                name: "since".to_string(),
                doc: Doc::new(),
                value: Format::U64,
            }],
            Doc::new(),
        ),
    );
    registry
}

/// The `kv` module as the installer configures it for package `Example`.
fn kv_config() -> CodeGeneratorConfig {
    let mut config = CodeGeneratorConfig::new("kv".to_string());
    config.parent = Some("Example".to_string());
    config
}

#[test]
fn root_reference_from_a_namespaced_module_is_qualified_with_the_root_package() {
    let shared = QualifiedTypeName::root("Shared".to_string());
    assert_eq!(
        SwiftCodeGenerator::requalify(&kv_config(), &shared),
        QualifiedTypeName::namespaced("Example".to_string(), "Shared".to_string()),
    );

    // The module's own types and those of other namespaces are unchanged.
    for name in [
        QualifiedTypeName::namespaced("kv".to_string(), "Entry".to_string()),
        QualifiedTypeName::namespaced("kit".to_string(), "Row".to_string()),
    ] {
        assert_eq!(SwiftCodeGenerator::requalify(&kv_config(), &name), name);
    }
}

#[test]
fn root_reference_is_unchanged_in_the_root_module_or_without_a_parent() {
    let shared = QualifiedTypeName::root("Shared".to_string());
    for config in [
        CodeGeneratorConfig::new("Example".to_string()),
        CodeGeneratorConfig::new("kv".to_string()),
    ] {
        assert_eq!(SwiftCodeGenerator::requalify(&config, &shared), shared);
    }
}

#[test]
fn namespaced_module_imports_the_root_package_and_qualifies_its_types() {
    let output = generate(&kv_config(), vec![Arc::new(BincodePlugin)], &kv_registry());

    assert!(output.contains("import Example\n"), "{output}");
    assert!(
        output.contains("public var shared: Example.Shared\n"),
        "{output}"
    );
    assert!(
        output.contains("let shared = try Example.Shared.deserialize(deserializer: deserializer)"),
        "{output}"
    );
    // The ROOT enum is qualified, and the module's own type of the same name
    // is not.
    assert!(
        output.contains("public var status: Example.Presence\n"),
        "{output}"
    );
    assert!(output.contains("public var local: Presence\n"), "{output}");
}

#[test]
fn namespaced_module_without_a_parent_is_unchanged() {
    let config = CodeGeneratorConfig::new("kv".to_string());
    let output = generate(&config, vec![Arc::new(BincodePlugin)], &kv_registry());

    assert!(!output.contains("Example"), "{output}");
    assert!(output.contains("public var shared: Shared\n"), "{output}");
}

#[test]
fn root_type_imported_into_a_namespaced_module_shadows_a_builtin() {
    let mut registry = kv_registry();
    registry.insert(
        QualifiedTypeName::namespaced("kv".to_string(), "Tags".to_string()),
        ContainerFormat::Struct(
            vec![Named {
                name: "ids".to_string(),
                doc: Doc::new(),
                value: Format::Set(Box::new(Format::U32)),
            }],
            Doc::new(),
        ),
    );

    // A ROOT struct `Set`, in scope in `kv` once it imports `Example`.
    let mut config = kv_config();
    config
        .registry_type_names
        .insert(QualifiedTypeName::root("Set".to_string()));

    let output = generate(&config, vec![], &registry);
    assert!(
        output.contains("public var ids: Swift.Set<UInt32>\n"),
        "{output}"
    );

    // Without a ROOT reference, `kv` does not import `Example`, so `Set` is
    // the builtin.
    registry.retain(|name, _| name.name == "Tags");
    let output = generate(&config, vec![], &registry);
    assert!(output.contains("public var ids: Set<UInt32>\n"), "{output}");
}
