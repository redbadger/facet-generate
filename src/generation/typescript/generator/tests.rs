use super::*;
use crate::reflection::format::{
    ContainerFormat, Doc, Format, Named, Namespace, QualifiedTypeName,
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

#[test]
fn update_qualified_names_strips_same_module_namespace() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "root".to_string(),
        "Child".to_string(),
    )));

    let updated = CodeGenerator::update_qualified_names(&config, &registry);

    let Format::TypeName(type_name) = first_field_type(&updated) else {
        panic!("expected type name");
    };
    assert_eq!(type_name.namespace, Namespace::Root);
    assert_eq!(type_name.name, "Child");
}

#[test]
fn update_qualified_names_keeps_external_namespace() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "other".to_string(),
        "Child".to_string(),
    )));

    let updated = CodeGenerator::update_qualified_names(&config, &registry);

    let Format::TypeName(type_name) = first_field_type(&updated) else {
        panic!("expected type name");
    };
    assert_eq!(type_name.namespace, Namespace::Named("other".to_string()));
    assert_eq!(type_name.name, "Child");
}

#[test]
fn update_qualified_names_does_not_mutate_input_registry() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "root".to_string(),
        "Child".to_string(),
    )));
    let original = registry.clone();

    let _updated = CodeGenerator::update_qualified_names(&config, &registry);
    assert_eq!(registry, original);
}

#[test]
fn output_adds_import_for_external_namespace() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "other".to_string(),
        "Child".to_string(),
    )));

    let generator = CodeGenerator::new(&config, InstallTarget::Node);
    let mut output = Vec::new();
    generator.output(&mut output, &registry).unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(output.contains(r#"import * as Other from "../other";"#));
}

#[test]
fn output_does_not_import_current_module() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "root".to_string(),
        "Child".to_string(),
    )));

    let generator = CodeGenerator::new(&config, InstallTarget::Node);
    let mut output = Vec::new();
    generator.output(&mut output, &registry).unwrap();
    let output = String::from_utf8(output).unwrap();

    assert!(!output.contains(r#"import * as Root from "../root";"#));
}
