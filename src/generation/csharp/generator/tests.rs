use super::*;
use crate::{
    generation::{CodeGeneratorConfig, Encoding},
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

fn render_output(config: &CodeGeneratorConfig, registry: &Registry) -> String {
    let generator = CodeGenerator::new(config);
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

    let updated = CodeGenerator::update_qualified_names(&config, &registry);

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

    let updated = CodeGenerator::update_qualified_names(&config, &registry);

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

    let updated = CodeGenerator::update_qualified_names(&config, &registry);

    let Format::TypeName(type_name) = first_field_type(&updated) else {
        panic!("expected type name");
    };
    assert_eq!(
        type_name.namespace,
        Namespace::Named("Company.Models".to_string())
    );
    assert_eq!(type_name.name, "User");
}

#[test]
fn output_writes_preamble_and_namespace() {
    let config =
        CodeGeneratorConfig::new("Company.Models".to_string()).with_encoding(Encoding::None);
    let registry = registry_with_struct_field(Format::Str);

    let output = render_output(&config, &registry);

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

    let output = render_output(&config, &registry);
    assert!(output.contains("private Company.Models.Shared.Child _value;"));
}

#[test]
fn output_json_encoding_adds_json_imports() {
    let config =
        CodeGeneratorConfig::new("Company.Models".to_string()).with_encoding(Encoding::Json);
    let registry = registry_with_struct_field(Format::Str);

    let output = render_output(&config, &registry);
    assert!(output.contains("using System.Text.Json.Serialization;"));
}

#[test]
fn output_bincode_encoding_adds_runtime_imports() {
    let config =
        CodeGeneratorConfig::new("Company.Models".to_string()).with_encoding(Encoding::Bincode);
    let registry = registry_with_struct_field(Format::Str);

    let output = render_output(&config, &registry);
    assert!(output.contains("using Facet.Runtime.Serde;"));
    assert!(output.contains("using Facet.Runtime.Bincode;"));
}
