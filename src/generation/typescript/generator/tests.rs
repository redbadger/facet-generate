//! Unit tests for [`CodeGenerator`] — import generation and qualified-name
//! resolution.
//!
//! Tests build small [`Registry`](crate::Registry) values by hand (rather than
//! via the `reflect!` macro) so that module and external-package configurations
//! can be controlled precisely.
//!
//! # Coverage
//!
//! | Area | What is tested |
//! |------|----------------|
//! | Same-module stripping | `Named` namespace matching module name → stripped to `Root` (bare name) |
//! | External namespace | `Named` namespace for a different module → preserved as `Namespace.Type` |
//! | Root namespace | Already-`Root` names pass through unchanged |
//! | Nested complex types | `Option<Seq<TypeName>>` — namespace stripping recurses into nested formats |
//! | Enum variants | Variant payloads containing `TypeName` are updated correctly |
//! | Immutability | `update_qualified_names` does not mutate the input registry |
//! | Import generation | Relative (`../namespace`), external package paths, `module_name` sub-paths, URL packages |
//! | Priority | External packages override relative imports for the same namespace |
//! | Deserialization | Qualified names appear correctly in `deserialize` call sites |

use std::collections::BTreeMap;

use super::*;
use crate::{
    generation::{
        CodeGeneratorConfig, Encoding,
        config::{ExternalPackage, PackageLocation},
    },
    reflection::format::{
        ContainerFormat, Doc, Format, Named, Namespace, QualifiedTypeName, VariantFormat,
    },
};

fn create_test_config(
    module_name: &str,
    external_definitions: BTreeMap<String, Vec<String>>,
) -> CodeGeneratorConfig {
    CodeGeneratorConfig::new(module_name.to_string())
        .with_external_definitions(external_definitions)
        .with_encoding(Encoding::None)
}

fn create_test_config_with_external_packages(
    module_name: &str,
    external_packages: BTreeMap<String, ExternalPackage>,
) -> CodeGeneratorConfig {
    let mut config =
        CodeGeneratorConfig::new(module_name.to_string()).with_encoding(Encoding::None);
    config.external_packages = external_packages;
    config
}

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
    let generator = CodeGenerator::new(config, InstallTarget::Node);
    let mut output = Vec::new();
    generator.output(&mut output, registry).unwrap();
    String::from_utf8(output).unwrap()
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
fn update_qualified_names_root_namespace_is_unchanged() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::root(
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
fn update_qualified_names_non_external_named_namespace_is_preserved() {
    let config = create_test_config("root", BTreeMap::new());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "legacy".to_string(),
        "OldType".to_string(),
    )));

    let updated = CodeGenerator::update_qualified_names(&config, &registry);

    let Format::TypeName(type_name) = first_field_type(&updated) else {
        panic!("expected type name");
    };
    assert_eq!(type_name.namespace, Namespace::Named("legacy".to_string()));
    assert_eq!(type_name.name, "OldType");
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
fn update_qualified_names_nested_complex_types() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let field_type = Format::Option(Box::new(Format::Seq(Box::new(Format::TypeName(
        QualifiedTypeName::namespaced("root".to_string(), "Child".to_string()),
    )))));
    let registry = registry_with_struct_field(field_type);

    let updated = CodeGenerator::update_qualified_names(&config, &registry);

    let Format::Option(inner) = first_field_type(&updated) else {
        panic!("expected option");
    };
    let Format::Seq(inner) = inner.as_ref() else {
        panic!("expected seq");
    };
    let Format::TypeName(type_name) = inner.as_ref() else {
        panic!("expected type name");
    };
    assert_eq!(type_name.namespace, Namespace::Root);
    assert_eq!(type_name.name, "Child");
}

#[test]
fn update_qualified_names_enum_variants() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let mut registry = Registry::new();
    let variant = Named {
        name: "Child".to_string(),
        doc: Doc::new(),
        value: VariantFormat::NewType(Box::new(Format::TypeName(QualifiedTypeName::namespaced(
            "root".to_string(),
            "Child".to_string(),
        )))),
    };
    let mut variants = BTreeMap::new();
    variants.insert(0, variant);
    registry.insert(
        QualifiedTypeName::root("Event".to_string()),
        ContainerFormat::Enum(variants, Doc::new()),
    );

    let updated = CodeGenerator::update_qualified_names(&config, &registry);
    let (_, container) = updated.iter().next().unwrap();
    let ContainerFormat::Enum(variants, _) = container else {
        panic!("expected enum");
    };
    let variant = variants.get(&0).unwrap();
    let VariantFormat::NewType(field) = &variant.value else {
        panic!("expected newtype variant");
    };
    let Format::TypeName(type_name) = field.as_ref() else {
        panic!("expected type name");
    };
    assert_eq!(type_name.namespace, Namespace::Root);
    assert_eq!(type_name.name, "Child");
}

#[test]
fn output_adds_import_for_external_namespace() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "other".to_string(),
        "Child".to_string(),
    )));

    let output = render_output(&config, &registry);
    assert!(output.contains(r#"import * as Other from "../other";"#));
}

#[test]
fn output_does_not_import_current_module() {
    let config = CodeGeneratorConfig::new("root".to_string());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "root".to_string(),
        "Child".to_string(),
    )));

    let output = render_output(&config, &registry);
    assert!(!output.contains(r#"import * as Root from "../root";"#));
}

#[test]
fn output_uses_external_package_path_for_namespace() {
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "other".to_string(),
        ExternalPackage {
            for_namespace: "shared-types".to_string(),
            location: PackageLocation::Path("../shared-types".to_string()),
            module_name: None,
            version: None,
        },
    );
    let config = create_test_config_with_external_packages("root", external_packages);
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "other".to_string(),
        "Child".to_string(),
    )));

    let output = render_output(&config, &registry);
    assert!(output.contains(r#"import * as Other from "shared-types";"#));
    assert!(!output.contains(r#"import * as Other from "../other";"#));
}

#[test]
fn output_uses_external_package_module_name_when_present() {
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "other".to_string(),
        ExternalPackage {
            for_namespace: "shared-types".to_string(),
            location: PackageLocation::Path("../shared-types".to_string()),
            module_name: Some("models".to_string()),
            version: None,
        },
    );
    let config = create_test_config_with_external_packages("root", external_packages);
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "other".to_string(),
        "Child".to_string(),
    )));

    let output = render_output(&config, &registry);
    assert!(output.contains(r#"import * as Other from "shared-types/models";"#));
}

#[test]
fn output_uses_for_namespace_for_url_packages() {
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "remote".to_string(),
        ExternalPackage {
            for_namespace: "@org/remote".to_string(),
            location: PackageLocation::Url("https://registry.npmjs.org/@org/remote".to_string()),
            module_name: None,
            version: Some("^1.0.0".to_string()),
        },
    );
    let config = create_test_config_with_external_packages("root", external_packages);
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "remote".to_string(),
        "RemoteType".to_string(),
    )));

    // TS imports use the package namespace identifier; URL metadata is used by installer logic.
    let output = render_output(&config, &registry);
    assert!(output.contains(r#"import * as Remote from "@org/remote";"#));
}

#[test]
fn output_external_package_takes_priority_over_relative_import() {
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("other".to_string(), vec!["Child".to_string()]);

    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "other".to_string(),
        ExternalPackage {
            for_namespace: "shared-types".to_string(),
            location: PackageLocation::Path("../shared-types".to_string()),
            module_name: None,
            version: None,
        },
    );

    let mut config = create_test_config("root", external_definitions);
    config.external_packages = external_packages;
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "other".to_string(),
        "Child".to_string(),
    )));

    let output = render_output(&config, &registry);
    assert!(output.contains(r#"import * as Other from "shared-types";"#));
    assert!(!output.contains(r#"import * as Other from "../other";"#));
}

#[test]
fn output_falls_back_to_relative_import_without_external_package() {
    let config = create_test_config("root", BTreeMap::new());
    let registry = registry_with_struct_field(Format::TypeName(QualifiedTypeName::namespaced(
        "legacy".to_string(),
        "OldType".to_string(),
    )));

    let output = render_output(&config, &registry);
    assert!(output.contains(r#"import * as Legacy from "../legacy";"#));
}

#[test]
fn output_handles_multiple_external_packages() {
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "auth".to_string(),
        ExternalPackage {
            for_namespace: "@org/auth".to_string(),
            location: PackageLocation::Url("https://registry.npmjs.org/@org/auth".to_string()),
            module_name: None,
            version: Some("^2.0.0".to_string()),
        },
    );
    external_packages.insert(
        "billing".to_string(),
        ExternalPackage {
            for_namespace: "billing-types".to_string(),
            location: PackageLocation::Path("../billing-types".to_string()),
            module_name: Some("v1".to_string()),
            version: None,
        },
    );
    let config = create_test_config_with_external_packages("root", external_packages);
    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::root("Holder".to_string()),
        ContainerFormat::Struct(
            vec![
                Named {
                    name: "user".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::namespaced(
                        "auth".to_string(),
                        "User".to_string(),
                    )),
                },
                Named {
                    name: "invoice".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::namespaced(
                        "billing".to_string(),
                        "Invoice".to_string(),
                    )),
                },
            ],
            Doc::new(),
        ),
    );

    let output = render_output(&config, &registry);
    assert!(output.contains(r#"import * as Auth from "@org/auth";"#));
    assert!(output.contains(r#"import * as Billing from "billing-types/v1";"#));
}

#[test]
fn output_deserialization_uses_local_and_external_qualification() {
    let mut config = CodeGeneratorConfig::new("root".to_string()).with_encoding(Encoding::Bincode);
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "other".to_string(),
        ExternalPackage {
            for_namespace: "shared-types".to_string(),
            location: PackageLocation::Path("../shared-types".to_string()),
            module_name: None,
            version: None,
        },
    );
    config.external_packages = external_packages;

    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::root("Holder".to_string()),
        ContainerFormat::Struct(
            vec![
                Named {
                    name: "local".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::namespaced(
                        "root".to_string(),
                        "LocalType".to_string(),
                    )),
                },
                Named {
                    name: "external".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::namespaced(
                        "other".to_string(),
                        "ExternalType".to_string(),
                    )),
                },
            ],
            Doc::new(),
        ),
    );

    let output = render_output(&config, &registry);
    assert!(output.contains("const local = LocalType.deserialize(deserializer);"));
    assert!(output.contains("const external = Other.ExternalType.deserialize(deserializer);"));
}

#[test]
fn output_mixed_external_and_local_references() {
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "other".to_string(),
        ExternalPackage {
            for_namespace: "shared-types".to_string(),
            location: PackageLocation::Path("../shared-types".to_string()),
            module_name: None,
            version: None,
        },
    );
    let config = create_test_config_with_external_packages("root", external_packages);
    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::root("Holder".to_string()),
        ContainerFormat::Struct(
            vec![
                Named {
                    name: "local_root".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::root("LocalRoot".to_string())),
                },
                Named {
                    name: "local_namespaced".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::namespaced(
                        "root".to_string(),
                        "LocalNamespaced".to_string(),
                    )),
                },
                Named {
                    name: "external".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::namespaced(
                        "other".to_string(),
                        "Child".to_string(),
                    )),
                },
            ],
            Doc::new(),
        ),
    );

    let output = render_output(&config, &registry);
    assert!(output.contains(r#"import * as Other from "shared-types";"#));
    assert!(!output.contains(r#"import * as Root from "../root";"#));
    assert!(output.contains("public local_root: LocalRoot"));
    assert!(output.contains("public local_namespaced: LocalNamespaced"));
    assert!(output.contains("public external: Other.Child"));
}

#[test]
fn output_user_example_multiple_external_references() {
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "other".to_string(),
        ExternalPackage {
            for_namespace: "other-package".to_string(),
            location: PackageLocation::Path("../other-package".to_string()),
            module_name: Some("models".to_string()),
            version: None,
        },
    );
    let mut config = create_test_config_with_external_packages("app", external_packages);
    config.encoding = Encoding::Bincode;

    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::root("ViewModel".to_string()),
        ContainerFormat::Struct(
            vec![
                Named {
                    name: "fact".to_string(),
                    doc: Doc::new(),
                    value: Format::Str,
                },
                Named {
                    name: "image".to_string(),
                    doc: Doc::new(),
                    value: Format::Option(Box::new(Format::TypeName(
                        QualifiedTypeName::namespaced("app".to_string(), "CatImage".to_string()),
                    ))),
                },
                Named {
                    name: "other".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::namespaced(
                        "other".to_string(),
                        "Other".to_string(),
                    )),
                },
                Named {
                    name: "another".to_string(),
                    doc: Doc::new(),
                    value: Format::TypeName(QualifiedTypeName::namespaced(
                        "other".to_string(),
                        "Other".to_string(),
                    )),
                },
            ],
            Doc::new(),
        ),
    );

    let output = render_output(&config, &registry);
    assert!(output.contains(r#"import * as Other from "other-package/models";"#));
    assert_eq!(output.matches("Other.Other").count(), 4);
    assert!(output.contains("public image: Optional<CatImage>"));
    assert!(output.contains("const image = deserializeOption(deserializer, (deserializer) => {"));
}
