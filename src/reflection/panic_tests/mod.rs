//! Tests that demonstrate scenarios where Variable placeholders are not resolved,
//! leading to panics in `RegistryBuilder::build()`

use crate::reflection::RegistryBuilder;
use crate::reflection::format::{ContainerFormat, Format};

#[cfg(test)]
mod tests {
    use crate::reflection::format;

    use super::*;
    use facet::Facet;

    /// Test that demonstrates the panic by manually creating an unresolved Variable
    /// This simulates what happens when `process_type` doesn't call `update_container_format`
    #[test]
    #[should_panic(expected = "should not have any remaining placeholders")]
    fn test_unresolved_variable_causes_panic() {
        let mut builder = RegistryBuilder::new();

        // Create a container with an unresolved Variable (this is what Box::default() creates)
        let unresolved_format = Format::unknown(); // This creates Variable(Variable::new(None))
        let container = ContainerFormat::NewTypeStruct(Box::new(unresolved_format));

        // Add this directly to the registry to simulate the scenario where
        // process_type creates a temporary container but fails to resolve the Variable
        let type_name = format::QualifiedTypeName {
            namespace: format::Namespace::Named("test".to_string()),
            name: "UnresolvedType".to_string(),
        };

        builder.registry.insert(type_name, container);

        // This should panic because we have an unresolved Variable in the registry
        let _registry = builder.build();
    }

    /// Test that demonstrates the issue with temporary containers
    /// This simulates the `process_newtype_variant_with_temp_container` workflow
    #[test]
    #[should_panic(expected = "should not have any remaining placeholders")]
    fn test_temp_container_with_unresolved_variable() {
        let mut builder = RegistryBuilder::new();

        // Simulate creating a temporary container like in process_newtype_variant_with_temp_container
        let _temp_name = builder.push_temporary(
            "test_variant".to_string(),
            ContainerFormat::NewTypeStruct(Box::default()), // Box::default() creates Format::unknown()
        );

        // Normally process_type would be called here and would update the Variable
        // But if process_type encounters an unhandled Def variant or other issue,
        // it won't call update_container_format and the Variable stays unresolved

        // Don't call process_type to simulate the failure case
        // builder.process_type(&some_shape, None);

        // Don't remove the temporary - this simulates a bug where the temp isn't cleaned up
        // builder.registry.remove(&temp_name);
        // builder.pop();

        // This should panic because the temporary container has an unresolved Variable
        let _registry = builder.build();
    }

    /// Test using a real struct to trigger the Variable resolution paths
    /// This test will pass normally, but if we can modify it to hit an unhandled case,
    /// it should panic
    #[test]
    fn test_normal_struct_processing_works() {
        #[derive(Facet)]
        struct TestStruct {
            field: u32,
        }

        let builder = RegistryBuilder::new();
        let builder = builder.add_type::<TestStruct>();

        // This should work fine - all Variables should be resolved
        let _registry = builder.build();
    }

    /// Test demonstrating what happens when we have nested unknown formats
    #[test]
    #[should_panic(expected = "should not have any remaining placeholders")]
    fn test_nested_unresolved_variables() {
        let mut builder = RegistryBuilder::new();

        // Create a struct container with a field that has an unresolved Variable
        let fields = vec![format::Named {
            name: "unresolved_field".to_string(),
            value: Format::unknown(), // Unresolved Variable
        }];

        let container = ContainerFormat::Struct(fields);

        let type_name = format::QualifiedTypeName {
            namespace: format::Namespace::Named("test".to_string()),
            name: "StructWithUnresolvedField".to_string(),
        };

        builder.registry.insert(type_name, container);

        // This should panic because the struct has a field with an unresolved Variable
        let _registry = builder.build();
    }

    /// Test demonstrating unresolved Variables in enum variants
    #[test]
    #[should_panic(expected = "should not have any remaining placeholders")]
    fn test_enum_with_unresolved_variant() {
        let mut builder = RegistryBuilder::new();

        // Create an enum container with a variant that has an unresolved Variable
        let mut variants = std::collections::BTreeMap::new();
        variants.insert(
            0,
            format::Named {
                name: "UnresolvedVariant".to_string(),
                value: format::VariantFormat::unknown(), // Unresolved Variable
            },
        );

        let container = ContainerFormat::Enum(variants);

        let type_name = format::QualifiedTypeName {
            namespace: format::Namespace::Named("test".to_string()),
            name: "EnumWithUnresolvedVariant".to_string(),
        };

        builder.registry.insert(type_name, container);

        // This should panic because the enum has a variant with an unresolved Variable
        let _registry = builder.build();
    }

    /// Test to verify that the panic message is correct
    #[test]
    #[should_panic(expected = "UnknownFormat")]
    fn test_variable_visit_returns_unknown_format_error() {
        use crate::reflection::format::{FormatHolder, Variable};

        // Create an unresolved Variable
        let variable: Variable<Format> = Variable::new(None);

        // Calling visit on an unresolved Variable should return UnknownFormat error
        let result = variable.visit(&mut |_| Ok(()));

        // This should panic with UnknownFormat when unwrapped
        result.unwrap();
    }

    /// Test to show what the `build()` method actually does
    #[test]
    fn test_build_method_visits_all_formats() {
        #[derive(Facet)]
        struct SimpleStruct(u32);

        let builder = RegistryBuilder::new();
        let builder = builder.add_type::<SimpleStruct>();

        // This should work - all formats should be properly resolved
        let registry = builder.build();

        // Verify the registry has content (don't assume exact namespace structure)
        assert!(
            !registry.is_empty(),
            "Registry should contain at least one type"
        );

        // Print the actual keys for debugging if needed
        for key in registry.keys() {
            println!("Registry contains: {key:?}");
        }
    }

    /// Test that demonstrates how a catch-all in `format_from_def_system` can cause panics
    /// This test shows what happens when `process_type` is called but doesn't update the container
    #[test]
    #[should_panic(expected = "should not have any remaining placeholders")]
    fn test_format_from_def_system_catch_all_issue() {
        // Create a builder and add a temporary container with unresolved Variable
        let mut builder = RegistryBuilder::new();

        let _temp_name = builder.push_temporary(
            "test_catch_all".to_string(),
            ContainerFormat::NewTypeStruct(Box::default()), // Creates Format::unknown()
        );

        // Create a shape that would hit the catch-all case in format_from_def_system
        // We'll use a mock approach since the real Shape construction is complex

        // The key insight is that if process_type encounters any Def variant that isn't
        // explicitly handled in format_from_def_system, it hits the catch-all _ => {}
        // and doesn't call any update_container_format method, leaving the Variable unresolved.

        // For this test, we'll simulate the effect by simply not calling process_type at all,
        // which has the same effect as calling process_type with an unhandled Def variant.

        // In a real scenario, this would be:
        // builder.process_type(&shape_with_unhandled_def, None);
        // where shape_with_unhandled_def.def would be something like:
        // - Def::Set(_) that isn't handled
        // - Def::Pointer(PointerDef { pointee: None, .. }) that isn't handled
        // - Any future Def variant added to facet that isn't handled in the match

        // Since we can't easily construct such shapes in tests, we demonstrate the effect
        // by showing what happens when the Variable isn't updated (same end result)

        // This should panic because the temporary container has an unresolved Variable
        let _registry = builder.build();
    }

    /// Test that shows the error message that gets converted to a panic
    #[test]
    #[should_panic(expected = "Incomplete reflection detected")]
    fn test_error_message_from_unresolved_variable() {
        let mut builder = RegistryBuilder::new();

        // Add an unresolved Variable to the registry
        let type_name = format::QualifiedTypeName {
            namespace: format::Namespace::Named("test".to_string()),
            name: "UnresolvedType".to_string(),
        };

        let container = ContainerFormat::NewTypeStruct(Box::default());
        builder.registry.insert(type_name, container);

        // The build method visits all formats, encounters the Variable, gets UnknownFormat error,
        // and panics with "should not have any remaining placeholders: Incomplete reflection detected"
        let _registry = builder.build();
    }
}
