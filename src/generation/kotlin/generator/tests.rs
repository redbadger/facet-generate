use super::*;
use crate::{
    generation::{CodeGeneratorConfig, Encoding},
    reflection::format::{
        ContainerFormat, Doc, Named, Namespace, QualifiedTypeName, VariantFormat,
    },
};
use std::collections::BTreeMap;

fn create_test_config(
    module_name: &str,
    external_definitions: BTreeMap<String, Vec<String>>,
) -> CodeGeneratorConfig {
    CodeGeneratorConfig::new(module_name.to_string())
        .with_external_definitions(external_definitions)
        .with_encoding(Encoding::None)
}

fn create_test_registry_with_struct_field(
    field_type_namespace: Namespace,
    field_type_name: &str,
) -> Registry {
    let mut registry = Registry::new();

    // Create the field type's qualified name
    let field_qualified_name = match field_type_namespace {
        Namespace::Root => QualifiedTypeName::root(field_type_name.to_string()),
        Namespace::Named(ns) => QualifiedTypeName::namespaced(ns, field_type_name.to_string()),
    };

    // Create a struct that references this type
    let field_format = Format::TypeName(field_qualified_name.clone());
    let named_field = Named {
        name: "field".to_string(),
        doc: Doc::new(),
        value: field_format,
    };
    let struct_container = ContainerFormat::Struct(vec![named_field], Doc::new());
    let struct_qualified_name = QualifiedTypeName::root("TestStruct".to_string());

    registry.insert(struct_qualified_name, struct_container);
    registry
}

#[test]
fn test_update_qualified_names_no_external_definitions() {
    // Test that when there are no external definitions, root types still get qualified
    let config = create_test_config("com.example", BTreeMap::new());
    let original_registry = create_test_registry_with_struct_field(Namespace::Root, "LocalType");

    let updated_registry = CodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.LocalType"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_external_type() {
    // Test that external types get fully qualified paths
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("other".to_string(), vec!["ExternalType".to_string()]);

    let config = create_test_config("com.example", external_definitions);
    let original_registry = create_test_registry_with_struct_field(
        Namespace::Named("other".to_string()),
        "ExternalType",
    );

    let updated_registry = CodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.other.ExternalType"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_same_namespace_type() {
    // Test that types in the same namespace as current module don't get treated as external
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("other".to_string(), vec!["SomeExternalType".to_string()]);

    let config = create_test_config("com.example.other", external_definitions);
    let original_registry = create_test_registry_with_struct_field(
        Namespace::Named("other".to_string()),
        "LocalTypeInOtherModule",
    );

    let updated_registry = CodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            // Should use the current module name, not treat as external
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.other.LocalTypeInOtherModule"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_root_namespace() {
    // Test that root namespace types get the current module name
    let config = create_test_config("com.example.service", BTreeMap::new());
    let original_registry = create_test_registry_with_struct_field(Namespace::Root, "RootType");

    let updated_registry = CodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.service.RootType"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_multiple_external_namespaces() {
    // Test multiple external definitions
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("auth".to_string(), vec!["User".to_string()]);
    external_definitions.insert("billing".to_string(), vec!["Invoice".to_string()]);

    let config = create_test_config("com.example.main", external_definitions);

    // Create a registry with types from both external namespaces
    let mut registry = Registry::new();

    // Auth type
    let auth_qualified_name = QualifiedTypeName::namespaced("auth".to_string(), "User".to_string());
    let auth_field = Named {
        name: "user".to_string(),
        doc: Doc::new(),
        value: Format::TypeName(auth_qualified_name),
    };

    // Billing type
    let billing_qualified_name =
        QualifiedTypeName::namespaced("billing".to_string(), "Invoice".to_string());
    let billing_field = Named {
        name: "invoice".to_string(),
        doc: Doc::new(),
        value: Format::TypeName(billing_qualified_name),
    };

    let struct_container = ContainerFormat::Struct(vec![auth_field, billing_field], Doc::new());
    let struct_qualified_name = QualifiedTypeName::root("TestStruct".to_string());
    registry.insert(struct_qualified_name, struct_container);

    let updated_registry = CodeGenerator::update_qualified_names(&config, &registry);

    // Check both field types
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        // Check auth field
        if let Format::TypeName(qualified_name) = &fields[0].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.main.auth.User"
            );
        } else {
            panic!("Expected TypeName format for auth field");
        }

        // Check billing field
        if let Format::TypeName(qualified_name) = &fields[1].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.main.billing.Invoice"
            );
        } else {
            panic!("Expected TypeName format for billing field");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_non_external_named_namespace() {
    // Test that named namespaces not in external_definitions get treated as local
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("other".to_string(), vec!["ExternalType".to_string()]);

    let config = create_test_config("com.example", external_definitions);
    let original_registry = create_test_registry_with_struct_field(
        Namespace::Named("local".to_string()),
        "LocalNamedType",
    );

    let updated_registry = CodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            // Should use the current module name since "local" is not in external_definitions
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.LocalNamedType"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_immutability() {
    // Test that the original registry is not modified
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("other".to_string(), vec!["ExternalType".to_string()]);

    let config = create_test_config("com.example", external_definitions);
    let original_registry = create_test_registry_with_struct_field(
        Namespace::Named("other".to_string()),
        "ExternalType",
    );

    // Keep a copy of the original to compare
    let original_copy = original_registry.clone();

    let _updated_registry = CodeGenerator::update_qualified_names(&config, &original_registry);

    // Original registry should be unchanged
    assert_eq!(original_registry, original_copy);

    // Check that the original still has the short namespace
    let (_, container) = original_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "other.ExternalType"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_nested_complex_types() {
    // Test with complex nested structures like Option<Vec<ExternalType>>
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("service".to_string(), vec!["Item".to_string()]);

    let config = create_test_config("com.example.api", external_definitions);

    // Create a registry with Option<Vec<ExternalType>>
    let mut registry = Registry::new();

    let external_type = Format::TypeName(QualifiedTypeName::namespaced(
        "service".to_string(),
        "Item".to_string(),
    ));
    let vec_type = Format::Seq(Box::new(external_type));
    let option_type = Format::Option(Box::new(vec_type));

    let field = Named {
        name: "items".to_string(),
        doc: Doc::new(),
        value: option_type,
    };

    let struct_container = ContainerFormat::Struct(vec![field], Doc::new());
    let struct_qualified_name = QualifiedTypeName::root("Container".to_string());
    registry.insert(struct_qualified_name, struct_container);

    let updated_registry = CodeGenerator::update_qualified_names(&config, &registry);

    // Navigate through the nested structure to check the inner type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::Option(inner) = &fields[0].value {
            if let Format::Seq(inner2) = inner.as_ref() {
                if let Format::TypeName(qualified_name) = inner2.as_ref() {
                    assert_eq!(
                        qualified_name.format(ToString::to_string, "."),
                        "com.example.api.service.Item"
                    );
                } else {
                    panic!("Expected inner TypeName format");
                }
            } else {
                panic!("Expected inner Seq format");
            }
        } else {
            panic!("Expected Option format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_enum_variants() {
    // Test that type names in enum variants get properly qualified
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("models".to_string(), vec!["User".to_string()]);

    let config = create_test_config("com.example.handlers", external_definitions);

    // Create an enum with variants that reference external types
    let mut registry = Registry::new();

    let user_type = Format::TypeName(QualifiedTypeName::namespaced(
        "models".to_string(),
        "User".to_string(),
    ));

    let variant = Named {
        name: "UserCreated".to_string(),
        doc: Doc::new(),
        value: VariantFormat::NewType(Box::new(user_type)),
    };

    let mut variants = BTreeMap::new();
    variants.insert(0, variant);
    let enum_container = ContainerFormat::Enum(variants, Doc::new());
    let enum_qualified_name = QualifiedTypeName::root("Event".to_string());
    registry.insert(enum_qualified_name, enum_container);

    let updated_registry = CodeGenerator::update_qualified_names(&config, &registry);

    // Check the enum variant type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Enum(variants, _) = container {
        let variant = variants.get(&0).unwrap();
        if let VariantFormat::NewType(boxed_format) = &variant.value {
            if let Format::TypeName(qualified_name) = boxed_format.as_ref() {
                assert_eq!(
                    qualified_name.format(ToString::to_string, "."),
                    "com.example.handlers.models.User"
                );
            } else {
                panic!("Expected TypeName format in NewType");
            }
        } else {
            panic!("Expected NewType variant");
        }
    } else {
        panic!("Expected Enum container");
    }
}
