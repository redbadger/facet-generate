//! Unit tests for [`KotlinKotlinCodeGenerator::update_qualified_names`].
//!
//! The Kotlin generator must rewrite the short, namespace-relative type names
//! produced by the registry into fully-qualified Kotlin package paths before
//! emitting code. Getting this wrong means generated `import` statements or
//! inline type references won't compile.
//!
//! These tests build small [`Registry`] values by hand (rather than via the
//! [`reflect!`] macro) so that namespace and external-package configurations
//! can be controlled precisely.
//!
//! # Coverage
//!
//! | Area | What is tested |
//! |------|----------------|
//! | Local types | Root-namespace and named-namespace types get the current module name prepended |
//! | External definitions | Types listed in `external_definitions` are resolved under the module's package path |
//! | External packages | `PackageLocation::Path` overrides the package prefix; `PackageLocation::Url` falls back to local resolution |
//! | Priority | External packages take precedence over external definitions for the same namespace |
//! | Nesting | Qualified-name rewriting recurses into `Option`, `Seq`, and enum-variant payloads |
//! | Immutability | The original registry is not mutated — a new copy is returned |
//! | Deserialization | Generated Bincode `deserialize` calls use fully-qualified names for both local and external types |

use super::*;
use crate::{
    generation::{
        CodeGeneratorConfig,
        config::{ExternalPackage, PackageLocation},
    },
    reflection::format::{
        ContainerFormat, Doc, EnumTagging, Named, Namespace, QualifiedTypeName, VariantFormat,
    },
};
use std::collections::BTreeMap;

fn create_test_config(
    module_name: &str,
    external_definitions: BTreeMap<String, Vec<String>>,
) -> CodeGeneratorConfig {
    CodeGeneratorConfig::new(module_name.to_string())
        .with_external_definitions(external_definitions)
}

fn create_test_config_with_external_packages(
    module_name: &str,
    external_packages: BTreeMap<String, ExternalPackage>,
) -> CodeGeneratorConfig {
    let mut config = CodeGeneratorConfig::new(module_name.to_string());
    config.external_packages = external_packages;
    config
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
    let field_format = Format::TypeName(field_qualified_name);
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

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

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

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

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

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

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

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

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

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &registry);

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
    // Test that named namespaces not in external_definitions preserve their namespace
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("other".to_string(), vec!["ExternalType".to_string()]);

    let config = create_test_config("com.example", external_definitions);
    let original_registry = create_test_registry_with_struct_field(
        Namespace::Named("local".to_string()),
        "LocalNamedType",
    );

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            // Should preserve the namespace for local named types
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.local.LocalNamedType"
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

    let _updated_registry =
        KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

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

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &registry);

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
    let enum_container = ContainerFormat::Enum(variants, EnumTagging::External, Doc::new());
    let enum_qualified_name = QualifiedTypeName::root("Event".to_string());
    registry.insert(enum_qualified_name, enum_container);

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &registry);

    // Check the enum variant type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Enum(variants, _, _) = container {
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

#[test]
fn test_update_qualified_names_external_package_path() {
    // Test that external packages with path locations get properly qualified
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "Other".to_string(),
        ExternalPackage {
            for_namespace: "Other".to_string(),
            location: PackageLocation::Path("com.crux.example.other".to_string()),
            module_name: None,
            version: None,
        },
    );

    let config =
        create_test_config_with_external_packages("com.crux.example.cat_facts", external_packages);
    let original_registry =
        create_test_registry_with_struct_field(Namespace::Named("Other".to_string()), "Other");

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.crux.example.other.Other.Other"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_user_example_scenario() {
    // Test that exactly matches the user's example scenario
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "Other".to_string(),
        ExternalPackage {
            for_namespace: "Other".to_string(),
            location: PackageLocation::Path("com.crux.example.other".to_string()),
            module_name: None,
            version: None,
        },
    );

    let config =
        create_test_config_with_external_packages("com.crux.example.cat_facts", external_packages);

    // Create a registry that matches the user's ViewModel struct
    let mut registry = Registry::new();

    // Create types that match the ViewModel structure
    let fact_field = Named {
        name: "fact".to_string(),
        doc: Doc::new(),
        value: Format::Str,
    };

    let cat_image_qualified_name =
        QualifiedTypeName::namespaced("App".to_string(), "CatImage".to_string());
    let image_field = Named {
        name: "image".to_string(),
        doc: Doc::new(),
        value: Format::Option(Box::new(Format::TypeName(cat_image_qualified_name))),
    };

    let platform_field = Named {
        name: "platform".to_string(),
        doc: Doc::new(),
        value: Format::Str,
    };

    let other_qualified_name =
        QualifiedTypeName::namespaced("Other".to_string(), "Other".to_string());
    let other_field = Named {
        name: "other".to_string(),
        doc: Doc::new(),
        value: Format::TypeName(other_qualified_name.clone()),
    };

    let another_field = Named {
        name: "another".to_string(),
        doc: Doc::new(),
        value: Format::TypeName(other_qualified_name),
    };

    let struct_container = ContainerFormat::Struct(
        vec![
            fact_field,
            image_field,
            platform_field,
            other_field,
            another_field,
        ],
        Doc::new(),
    );
    let view_model_qualified_name =
        QualifiedTypeName::namespaced("App".to_string(), "ViewModel".to_string());
    registry.insert(view_model_qualified_name, struct_container);

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &registry);

    // Check the field types
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        // Check image field (should be App.CatImage - local type)
        if let Format::Option(inner) = &fields[1].value {
            if let Format::TypeName(qualified_name) = inner.as_ref() {
                assert_eq!(
                    qualified_name.format(ToString::to_string, "."),
                    "com.crux.example.cat_facts.App.CatImage"
                );
            } else {
                panic!("Expected inner TypeName format for image field");
            }
        } else {
            panic!("Expected Option format for image field");
        }

        // Check other field (should use external package location)
        if let Format::TypeName(qualified_name) = &fields[3].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.crux.example.other.Other.Other"
            );
        } else {
            panic!("Expected TypeName format for other field");
        }

        // Check another field (should also use external package location)
        if let Format::TypeName(qualified_name) = &fields[4].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.crux.example.other.Other.Other"
            );
        } else {
            panic!("Expected TypeName format for another field");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_external_package_url_ignored() {
    // Test that external packages with URL locations are ignored for Kotlin
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "Remote".to_string(),
        ExternalPackage {
            for_namespace: "Remote".to_string(),
            location: PackageLocation::Url("https://example.com/remote".to_string()),
            module_name: None,
            version: None,
        },
    );

    let config = create_test_config_with_external_packages("com.example.local", external_packages);
    let original_registry = create_test_registry_with_struct_field(
        Namespace::Named("Remote".to_string()),
        "RemoteType",
    );

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            // URL locations are ignored, so it should fall back to local behavior
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.local.Remote.RemoteType"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_external_package_priority() {
    // Test that external packages take priority over external definitions
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("Shared".to_string(), vec!["Type".to_string()]);

    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "Shared".to_string(),
        ExternalPackage {
            for_namespace: "Shared".to_string(),
            location: PackageLocation::Path("com.external.shared".to_string()),
            module_name: None,
            version: None,
        },
    );

    let mut config = create_test_config("com.example.main", external_definitions);
    config.external_packages = external_packages;

    let original_registry =
        create_test_registry_with_struct_field(Namespace::Named("Shared".to_string()), "Type");

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            // Should use external package location, not external definitions logic
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.external.shared.Shared.Type"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_multiple_external_packages() {
    // Test multiple external packages with different locations
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "Auth".to_string(),
        ExternalPackage {
            for_namespace: "Auth".to_string(),
            location: PackageLocation::Path("com.auth.service".to_string()),
            module_name: None,
            version: None,
        },
    );
    external_packages.insert(
        "Billing".to_string(),
        ExternalPackage {
            for_namespace: "Billing".to_string(),
            location: PackageLocation::Path("com.billing.api".to_string()),
            module_name: None,
            version: None,
        },
    );

    let config = create_test_config_with_external_packages("com.example.main", external_packages);

    // Create a registry with types from both external packages
    let mut registry = Registry::new();

    // Auth type
    let auth_qualified_name = QualifiedTypeName::namespaced("Auth".to_string(), "User".to_string());
    let auth_field = Named {
        name: "user".to_string(),
        doc: Doc::new(),
        value: Format::TypeName(auth_qualified_name),
    };

    // Billing type
    let billing_qualified_name =
        QualifiedTypeName::namespaced("Billing".to_string(), "Invoice".to_string());
    let billing_field = Named {
        name: "invoice".to_string(),
        doc: Doc::new(),
        value: Format::TypeName(billing_qualified_name),
    };

    let struct_container = ContainerFormat::Struct(vec![auth_field, billing_field], Doc::new());
    let struct_qualified_name = QualifiedTypeName::root("TestStruct".to_string());
    registry.insert(struct_qualified_name, struct_container);

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &registry);

    // Check both field types
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        // Check auth field
        if let Format::TypeName(qualified_name) = &fields[0].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.auth.service.Auth.User"
            );
        } else {
            panic!("Expected TypeName format for auth field");
        }

        // Check billing field
        if let Format::TypeName(qualified_name) = &fields[1].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.billing.api.Billing.Invoice"
            );
        } else {
            panic!("Expected TypeName format for billing field");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_update_qualified_names_fallback_to_external_definitions() {
    // Test that when no external package matches, we fall back to external definitions
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("Legacy".to_string(), vec!["OldType".to_string()]);

    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "Modern".to_string(),
        ExternalPackage {
            for_namespace: "Modern".to_string(),
            location: PackageLocation::Path("com.modern.lib".to_string()),
            module_name: None,
            version: None,
        },
    );

    let mut config = create_test_config("com.example.main", external_definitions);
    config.external_packages = external_packages;

    let original_registry =
        create_test_registry_with_struct_field(Namespace::Named("Legacy".to_string()), "OldType");

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &original_registry);

    // Find the struct container and check its field type
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        if let Format::TypeName(qualified_name) = &fields[0].value {
            // Should use external definitions logic since no external package matches "Legacy"
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.main.Legacy.OldType"
            );
        } else {
            panic!("Expected TypeName format");
        }
    } else {
        panic!("Expected Struct container");
    }
}

#[test]
fn test_deserialization_uses_fully_qualified_names() {
    // Test that deserialization code uses fully qualified type names
    use crate::{
        generation::{
            CodeGeneratorConfig,
            config::{ExternalPackage, PackageLocation},
            kotlin::KotlinCodeGenerator,
        },
        reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName},
    };
    use std::collections::BTreeMap;

    // Set up external packages
    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "Other".to_string(),
        ExternalPackage {
            for_namespace: "Other".to_string(),
            location: PackageLocation::Path("com.external.other".to_string()),
            module_name: None,
            version: None,
        },
    );

    let mut config = CodeGeneratorConfig::new("com.example.main".to_string());
    config.external_packages = external_packages;

    // Create a registry with types that reference external types
    let mut registry = Registry::new();

    // Create an external type reference
    let external_type = Format::TypeName(QualifiedTypeName::namespaced(
        "Other".to_string(),
        "ExternalType".to_string(),
    ));

    // Create a local type reference
    let local_type = Format::TypeName(QualifiedTypeName::namespaced(
        "App".to_string(),
        "LocalType".to_string(),
    ));

    // Create a struct that uses both
    let external_field = Named {
        name: "external_field".to_string(),
        doc: Doc::new(),
        value: external_type,
    };

    let local_field = Named {
        name: "local_field".to_string(),
        doc: Doc::new(),
        value: local_type,
    };

    let struct_container = ContainerFormat::Struct(vec![external_field, local_field], Doc::new());

    let struct_qualified_name =
        QualifiedTypeName::namespaced("App".to_string(), "TestStruct".to_string());
    registry.insert(struct_qualified_name, struct_container);

    // Generate the Kotlin code
    let code_generator = KotlinCodeGenerator::new(&config).with_plugins(vec![std::sync::Arc::new(
        crate::generation::bincode::BincodePlugin,
    )]);
    let mut output = Vec::new();
    code_generator.output(&mut output, &registry).unwrap();
    let generated_code = String::from_utf8(output).unwrap();

    // Verify the deserialization code uses fully qualified names
    assert!(
        generated_code.contains("com.external.other.Other.ExternalType.deserialize(deserializer)"),
        "Generated code should use fully qualified name for external type in deserialization: {generated_code}"
    );

    assert!(
        generated_code.contains("com.example.main.App.LocalType.deserialize(deserializer)"),
        "Generated code should use fully qualified name for local type in deserialization: {generated_code}"
    );
}

#[test]
fn test_update_qualified_names_mixed_external_and_local() {
    // Test a mix of external package types, external definition types, and local types
    let mut external_definitions = BTreeMap::new();
    external_definitions.insert("Legacy".to_string(), vec!["OldType".to_string()]);

    let mut external_packages = BTreeMap::new();
    external_packages.insert(
        "Modern".to_string(),
        ExternalPackage {
            for_namespace: "Modern".to_string(),
            location: PackageLocation::Path("com.modern.lib".to_string()),
            module_name: None,
            version: None,
        },
    );

    let mut config = create_test_config("com.example.main", external_definitions);
    config.external_packages = external_packages;

    // Create a registry with all three types of references
    let mut registry = Registry::new();

    // External package type
    let modern_qualified_name =
        QualifiedTypeName::namespaced("Modern".to_string(), "NewType".to_string());
    let modern_field = Named {
        name: "modern".to_string(),
        doc: Doc::new(),
        value: Format::TypeName(modern_qualified_name),
    };

    // External definitions type
    let legacy_qualified_name =
        QualifiedTypeName::namespaced("Legacy".to_string(), "OldType".to_string());
    let legacy_field = Named {
        name: "legacy".to_string(),
        doc: Doc::new(),
        value: Format::TypeName(legacy_qualified_name),
    };

    // Local type
    let local_qualified_name = QualifiedTypeName::root("LocalType".to_string());
    let local_field = Named {
        name: "local".to_string(),
        doc: Doc::new(),
        value: Format::TypeName(local_qualified_name),
    };

    let struct_container =
        ContainerFormat::Struct(vec![modern_field, legacy_field, local_field], Doc::new());
    let struct_qualified_name = QualifiedTypeName::root("TestStruct".to_string());
    registry.insert(struct_qualified_name, struct_container);

    let updated_registry = KotlinCodeGenerator::update_qualified_names(&config, &registry);

    // Check all field types
    let (_, container) = updated_registry.iter().next().unwrap();
    if let ContainerFormat::Struct(fields, _) = container {
        // Check external package field
        if let Format::TypeName(qualified_name) = &fields[0].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.modern.lib.Modern.NewType"
            );
        } else {
            panic!("Expected TypeName format for modern field");
        }

        // Check external definitions field
        if let Format::TypeName(qualified_name) = &fields[1].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.main.Legacy.OldType"
            );
        } else {
            panic!("Expected TypeName format for legacy field");
        }

        // Check local field
        if let Format::TypeName(qualified_name) = &fields[2].value {
            assert_eq!(
                qualified_name.format(ToString::to_string, "."),
                "com.example.main.LocalType"
            );
        } else {
            panic!("Expected TypeName format for local field");
        }
    } else {
        panic!("Expected Struct container");
    }
}

// ---------------------------------------------------------------------------
// References to ROOT types
// ---------------------------------------------------------------------------

/// A namespaced module nested under `package` by the installer.
fn namespaced_config(package: &str, namespace: &str) -> CodeGeneratorConfig {
    CodeGeneratorConfig::new(namespace.to_string()).with_parent(package)
}

#[test]
fn root_reference_from_a_namespaced_module_is_rooted_at_the_root_package() {
    let shared = QualifiedTypeName::root("Shared".to_string());

    // Dotted and undotted packages, and one whose last segment is the
    // module's own namespace.
    for package in ["com.example", "example", "com.kv"] {
        let config = namespaced_config(package, "kv");
        assert_eq!(
            KotlinCodeGenerator::requalify(&config, &shared),
            QualifiedTypeName::namespaced(package.to_string(), "Shared".to_string()),
            "from {}",
            config.module_name(),
        );
    }
}

#[test]
fn root_reference_from_the_root_module_is_unchanged() {
    let shared = QualifiedTypeName::root("Shared".to_string());

    for package in ["com.example", "example"] {
        // The installer's `with_parent` leaves the root module as it is.
        let config = CodeGeneratorConfig::new(package.to_string()).with_parent(package);
        assert_eq!(
            KotlinCodeGenerator::requalify(&config, &shared),
            QualifiedTypeName::namespaced(package.to_string(), "Shared".to_string()),
        );
    }
}

#[test]
fn own_namespace_reference_is_unchanged_when_the_package_ends_in_it() {
    // `com.kv` + `kv` is `com.kv.kv`: the leaf is still the module's own
    // namespace, so its types stay in `com.kv.kv`, and the ROOT ones (above)
    // in `com.kv`.
    let config = namespaced_config("com.kv", "kv");
    let entry = QualifiedTypeName::namespaced("kv".to_string(), "Entry".to_string());
    assert_eq!(
        KotlinCodeGenerator::requalify(&config, &entry),
        QualifiedTypeName::namespaced("com.kv.kv".to_string(), "Entry".to_string()),
    );
}

#[test]
fn root_reference_is_written_with_the_root_package() {
    let mut registry = Registry::new();
    registry.insert(
        QualifiedTypeName::namespaced("kv".to_string(), "Entry".to_string()),
        ContainerFormat::Struct(
            vec![Named {
                name: "shared".to_string(),
                doc: Doc::new(),
                value: Format::TypeName(QualifiedTypeName::root("Shared".to_string())),
            }],
            Doc::new(),
        ),
    );

    let config = namespaced_config("com.example", "kv");
    let mut out = Vec::new();
    KotlinCodeGenerator::new(&config)
        .output(&mut out, &registry)
        .unwrap();
    let out = String::from_utf8(out).unwrap();

    assert!(out.contains("val shared: com.example.Shared,"), "{out}");
    assert!(!out.contains("com.example.kv.Shared"), "{out}");
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
    let cfg = CodeGeneratorConfig::new("com.example".to_string());
    let err = KotlinCodeGenerator::new(&cfg)
        .output(&mut Vec::new(), &registry)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        err.to_string(),
        "Kotlin: type `Serializer` collides with the `Serializer` import used by the generated code; rename it with #[facet(rename = \"...\")]"
    );
}

/// A nested class only shadows the file's imports inside the enum that
/// declares it, so a variant may take the name of an import as long as nothing
/// in its own enum refers to that import — even when another type in the
/// module does, and the import is written.
#[test]
fn variant_named_bytes_is_allowed_when_its_enum_has_no_bytes_field() {
    use crate as fg;

    #[derive(facet::Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Value {
        None,
        Bytes(Vec<u8>),
    }

    #[derive(facet::Facet)]
    struct Set {
        key: String,
        #[facet(fg::bytes)]
        value: Vec<u8>,
    }

    let registry = crate::reflect!(Value, Set).unwrap();
    let mut cfg = CodeGeneratorConfig::new("com.example".to_string());
    cfg.update_from(&registry);
    let mut out = Vec::new();
    KotlinCodeGenerator::new(&cfg)
        .output(&mut out, &registry)
        .unwrap();

    let source = String::from_utf8(out).unwrap();
    assert!(source.contains("class Bytes("), "{source}");
    assert!(source.contains("val value: Bytes"), "{source}");
}

/// Inside that enum, though, `Bytes` would name the nested class, so a bytes
/// field in a sibling variant makes the name a genuine collision.
#[test]
fn variant_named_bytes_is_rejected_when_its_enum_has_a_bytes_field() {
    use crate as fg;

    #[derive(facet::Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Blob {
        Bytes(Vec<u8>),
        Raw(#[facet(fg::bytes)] Vec<u8>),
    }

    let registry = crate::reflect!(Blob).unwrap();
    let cfg = CodeGeneratorConfig::new("com.example".to_string());
    let err = KotlinCodeGenerator::new(&cfg)
        .output(&mut Vec::new(), &registry)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        err.to_string(),
        "Kotlin: type `Bytes` collides with the `Bytes` import used by the generated code; rename it with #[facet(rename = \"...\")]"
    );
}

/// The `Bytes` import is only written for a module with a bytes field, so a
/// top-level type of that name is fine in a module without one.
#[test]
fn type_named_bytes_is_allowed_when_nothing_in_the_module_is_bytes() {
    #[derive(facet::Facet)]
    #[facet(rename = "Bytes")]
    struct Renamed {
        name: String,
    }

    let registry = crate::reflect!(Renamed).unwrap();
    let cfg = CodeGeneratorConfig::new("com.example".to_string());
    let mut out = Vec::new();
    KotlinCodeGenerator::new(&cfg)
        .output(&mut out, &registry)
        .unwrap();

    assert!(String::from_utf8(out).unwrap().contains("class Bytes("));
}

/// And it is rejected as soon as one is, because the import then outranks the
/// same-package declaration everywhere.
#[test]
fn type_named_bytes_is_rejected_when_the_module_has_a_bytes_field() {
    use crate as fg;

    #[derive(facet::Facet)]
    #[facet(rename = "Bytes")]
    struct Renamed {
        name: String,
    }

    #[derive(facet::Facet)]
    struct Set {
        #[facet(fg::bytes)]
        value: Vec<u8>,
    }

    let registry = crate::reflect!(Renamed, Set).unwrap();
    let cfg = CodeGeneratorConfig::new("com.example".to_string());
    let err = KotlinCodeGenerator::new(&cfg)
        .output(&mut Vec::new(), &registry)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        err.to_string()
            .starts_with("Kotlin: type `Bytes` collides with")
    );
}

#[test]
fn field_named_to_string_is_rejected() {
    #[derive(facet::Facet)]
    struct Foo {
        to_string: String,
    }

    let registry = crate::reflect!(Foo).unwrap();
    let cfg = CodeGeneratorConfig::new("com.example".to_string());
    let err = KotlinCodeGenerator::new(&cfg)
        .output(&mut Vec::new(), &registry)
        .unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        err.to_string(),
        "Kotlin: field `to_string` of `Foo` would become `toString`, which Kotlin generates for every data class; rename it with #[facet(rename = \"...\")]"
    );
}
