//! Unit tests for [`CSharpCodeGenerator`] — namespace resolution and preamble
//! generation.
//!
//! Tests build small registries by hand to exercise the qualified-name
//! rewriting rules without depending on snapshot output.
//!
//! # Coverage
//!
//! - Same-leaf-namespace stripping (`Users` inside `Company.Models.Users` → bare name)
//! - External namespace rooting under module (`Payments` → `Company.Models.Payments`)
//! - Root-to-dotted promotion (`Root` inside `Company.Models` → `Named("Company.Models")`)
//! - References from a namespaced module rooted at the root package, not the
//!   module (`Root` and `kit` inside `Company.Models.feature` →
//!   `Company.Models.Shared`, `Company.Models.Kit.Row`)
//! - Preamble (`using` directives + `namespace` declaration)
//! - Plugin-specific imports (JSON adds `System.Text.Json.Serialization`,
//!   Bincode adds `Facet.Runtime.Bincode`)

use std::sync::Arc;

use super::*;
use crate::{
    generation::{
        CodeGeneratorConfig, bincode::BincodePlugin, csharp::emitter::CSharp, json::JsonPlugin,
        plugin::EmitterPlugin,
    },
    reflection::format::{ContainerFormat, Doc, Format, Named, Namespace, QualifiedTypeName},
};

fn registry_with_struct_field(field_type: Format) -> Registry {
    let mut registry = Registry::new();
    let fields = vec![Named {
        name: "value".to_string(),
        doc: Doc::new(),
        value: field_type,
    }];
    registry.insert(
        QualifiedTypeName::root("Holder".to_string()),
        ContainerFormat::Struct(fields, Doc::new()),
    );
    registry
}

fn first_field_type(registry: &Registry) -> &Format {
    let (_, container) = registry.iter().next().unwrap();
    let ContainerFormat::Struct(fields, _) = container else {
        panic!("expected struct container");
    };
    &fields[0].value
}

fn render_output(
    config: &CodeGeneratorConfig,
    plugins: Vec<Arc<dyn EmitterPlugin<CSharp>>>,
    registry: &Registry,
) -> String {
    let generator = CSharpCodeGenerator::new(config).with_plugins(plugins);
    let mut output = Vec::new();
    generator.output(&mut output, registry).unwrap();
    String::from_utf8(output).unwrap()
}

#[test]
fn update_qualified_names_strips_same_leaf_namespace() {
    let config = CodeGeneratorConfig::new("Company.Models.Users".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "Users".to_string(),
        "UserSummary".to_string(),
    )));

    let updated = CSharpCodeGenerator::update_qualified_names(&config, &registry);

    let Format::TypeName(type_name) = first_field_type(&updated) else {
        panic!("expected type name");
    };
    assert_eq!(type_name.namespace, Namespace::Root);
    assert_eq!(type_name.name, "UserSummary");
}

#[test]
fn update_qualified_names_roots_external_namespace_under_module() {
    let config = CodeGeneratorConfig::new("Company.Models".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "Payments".to_string(),
        "Invoice".to_string(),
    )));

    let updated = CSharpCodeGenerator::update_qualified_names(&config, &registry);

    let Format::TypeName(type_name) = first_field_type(&updated) else {
        panic!("expected type name");
    };
    assert_eq!(
        type_name.namespace,
        Namespace::Named("Company.Models.Payments".to_string())
    );
    assert_eq!(type_name.name, "Invoice");
}

#[test]
fn update_qualified_names_roots_root_namespace_for_dotted_module() {
    let config = CodeGeneratorConfig::new("Company.Models".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::root(
        "User".to_string(),
    )));

    let updated = CSharpCodeGenerator::update_qualified_names(&config, &registry);

    let Format::TypeName(type_name) = first_field_type(&updated) else {
        panic!("expected type name");
    };
    assert_eq!(
        type_name.namespace,
        Namespace::Named("Company.Models".to_string())
    );
    assert_eq!(type_name.name, "User");
}

/// A module named `namespace` (or the root module, for `None`) the way the
/// installer configures it under `root_package`.
fn module_config(root_package: &str, namespace: Option<&str>) -> CodeGeneratorConfig {
    CodeGeneratorConfig::new(namespace.unwrap_or(root_package).to_string())
        .with_parent(root_package)
}

fn requalified(config: &CodeGeneratorConfig, name: &QualifiedTypeName) -> QualifiedTypeName {
    CSharpCodeGenerator::requalify(config, name)
}

#[test]
fn requalify_roots_a_root_reference_at_the_root_package() {
    let shared = QualifiedTypeName::root("Shared".to_string());

    // From the root module: qualified when the package is dotted, bare when
    // it is not (the bare name is declared right there).
    assert_eq!(
        requalified(&module_config("Company.Models", None), &shared),
        QualifiedTypeName::namespaced("Company.Models".to_string(), "Shared".to_string())
    );
    assert_eq!(
        requalified(&module_config("Example", None), &shared),
        shared
    );

    // From a namespaced module: always the root package, never the module's
    // own namespace (`Company.Models.kv`), dotted or not.
    assert_eq!(
        requalified(&module_config("Company.Models", Some("kv")), &shared),
        QualifiedTypeName::namespaced("Company.Models".to_string(), "Shared".to_string())
    );
    assert_eq!(
        requalified(&module_config("Example", Some("kv")), &shared),
        QualifiedTypeName::namespaced("Example".to_string(), "Shared".to_string())
    );
}

#[test]
fn requalify_roots_a_sibling_namespace_at_the_root_package() {
    let row = QualifiedTypeName::namespaced("kit".to_string(), "Row".to_string());

    for root_package in ["Company.Models", "Example"] {
        let expected =
            QualifiedTypeName::namespaced(format!("{root_package}.kit"), "Row".to_string());

        // From the root module, and from another namespace, which must not
        // nest `kit` inside itself (`Company.Models.feature.kit`).
        assert_eq!(
            requalified(&module_config(root_package, None), &row),
            expected
        );
        assert_eq!(
            requalified(&module_config(root_package, Some("feature")), &row),
            expected
        );
    }
}

#[test]
fn requalify_leaves_a_same_namespace_reference_bare() {
    let row = QualifiedTypeName::namespaced("kit".to_string(), "Row".to_string());

    for root_package in ["Company.Models", "Example"] {
        assert_eq!(
            requalified(&module_config(root_package, Some("kit")), &row),
            QualifiedTypeName::root("Row".to_string())
        );
    }
}

/// The qualified paths match the `namespace` declaration each module writes,
/// which upper-camel-cases every segment (`Company.Models.Kit`).
#[test]
fn output_from_a_namespaced_module_names_root_and_sibling_types_as_declared() {
    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::namespaced("feature".to_string(), "FeatureView".to_string()),
        ContainerFormat::Struct(
            vec![
                Named {
                    name: "row".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::namespaced(
                        "kit".to_string(),
                        "Row".to_string(),
                    )),
                },
                Named {
                    name: "shared".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::root("Shared".to_string())),
                },
            ],
            Doc::new(),
        ),
    );

    let output = render_output(
        &module_config("Company.Models", Some("feature")),
        vec![],
        &registry,
    );
    assert!(
        output.contains("namespace Company.Models.Feature;"),
        "{output}"
    );
    assert!(
        output.contains("private Company.Models.Kit.Row _row;"),
        "{output}"
    );
    assert!(
        output.contains("private Company.Models.Shared _shared;"),
        "{output}"
    );

    let output = render_output(
        &module_config("Example", Some("feature")),
        vec![],
        &registry,
    );
    assert!(output.contains("namespace Example.Feature;"), "{output}");
    assert!(output.contains("private Example.Kit.Row _row;"), "{output}");
    assert!(
        output.contains("private Example.Shared _shared;"),
        "{output}"
    );
}

#[test]
fn output_writes_preamble_and_namespace() {
    let config = CodeGeneratorConfig::new("Company.Models".to_string());
    let registry = registry_with_struct_field(Format::Str);

    let output = render_output(&config, vec![], &registry);

    assert!(output.contains("using CommunityToolkit.Mvvm.ComponentModel;"));
    assert!(output.contains("namespace Company.Models;"));
    assert!(output.contains("public partial class Holder : ObservableObject"));
}

#[test]
fn output_uses_rooted_namespace_for_external_types() {
    let config = CodeGeneratorConfig::new("Company.Models".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "Shared".to_string(),
        "Child".to_string(),
    )));

    let output = render_output(&config, vec![], &registry);
    assert!(output.contains("private Company.Models.Shared.Child _value;"));
}

#[test]
fn output_json_encoding_adds_json_imports() {
    let config = CodeGeneratorConfig::new("Company.Models".to_string());
    let registry = registry_with_struct_field(Format::Str);

    let output = render_output(&config, vec![Arc::new(JsonPlugin)], &registry);
    assert!(output.contains("using System.Text.Json.Serialization;"));
}

#[test]
fn output_bincode_encoding_adds_runtime_imports() {
    let config = CodeGeneratorConfig::new("Company.Models".to_string());
    let registry = registry_with_struct_field(Format::Str);

    let output = render_output(&config, vec![Arc::new(BincodePlugin)], &registry);
    assert!(output.contains("using Facet.Runtime.Serde;"));
    assert!(output.contains("using Facet.Runtime.Bincode;"));
}

// ---------------------------------------------------------------------------
// Reserved-name pre-pass
// ---------------------------------------------------------------------------

#[test]
fn type_named_bincode_serializer_is_rejected() {
    #[derive(facet::Facet)]
    #[facet(rename = "BincodeSerializer")]
    struct Renamed {
        name: String,
    }

    let registry = crate::reflect!(Renamed).unwrap();
    let cfg = CodeGeneratorConfig::new("Example".to_string());
    let err = CSharpCodeGenerator::new(&cfg)
        .output(&mut Vec::new(), &registry)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        err.to_string(),
        "C#: type `BincodeSerializer` collides with the runtime type `Facet.Runtime.Bincode.BincodeSerializer` used by the generated code; rename it with #[facet(rename = \"...\")]"
    );
}

#[test]
fn field_named_get_hash_code_is_rejected() {
    #[derive(facet::Facet)]
    struct Foo {
        get_hash_code: String,
    }

    let registry = crate::reflect!(Foo).unwrap();
    let cfg = CodeGeneratorConfig::new("Example".to_string());
    let err = CSharpCodeGenerator::new(&cfg)
        .output(&mut Vec::new(), &registry)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        err.to_string(),
        "C#: field `get_hash_code` of `Foo` would become `GetHashCode`, which hides an inherited member of every C# object; rename it with #[facet(rename = \"...\")]"
    );
}

#[test]
fn field_named_after_its_enclosing_type_is_rejected() {
    #[derive(facet::Facet)]
    struct Keys {
        keys: Vec<String>,
    }

    let registry = crate::reflect!(Keys).unwrap();
    let cfg = CodeGeneratorConfig::new("Example".to_string());
    let err = CSharpCodeGenerator::new(&cfg)
        .output(&mut Vec::new(), &registry)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        err.to_string(),
        "C#: field `keys` of `Keys` would become property `Keys`, the same name as its enclosing type (CS0542); rename it with #[facet(rename = \"...\")]"
    );
}
