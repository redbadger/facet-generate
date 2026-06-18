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
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName},
};

use super::*;
use crate::generation::swift::emitter::Swift;

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
        ContainerFormat::Enum(variants, Doc::new()),
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
        ContainerFormat::Enum(variants, Doc::new()),
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
        ContainerFormat::Enum(variants, Doc::new()),
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
