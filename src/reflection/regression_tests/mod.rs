//! Regression tests for Variable placeholder resolution issues.
//!
//! These tests verify that the Variable resolution problems in `RegistryBuilder` have been fixed
//! and serve as regression tests to prevent these issues from reoccurring.
//!
//! Previously, these scenarios would cause panics with "There was a problem reflecting type".
//! Now they should all work correctly, demonstrating that the underlying issues have been resolved.

use crate::reflection::RegistryBuilder;
use crate::reflection::format::{ContainerFormat, Format, VariantFormat};

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashSet};

    use crate::{
        error::Error,
        reflect,
        reflection::format::{Doc, Namespace, QualifiedTypeName},
    };

    use super::*;
    use facet::Facet;

    /// Regression test: Verify that Set types now work correctly
    /// Previously this would panic due to unhandled `Def::Set` variants
    #[test]
    fn test_set_types_work() {
        #[derive(Facet)]
        struct WithSets {
            btree_set: BTreeSet<String>,
            hash_set: HashSet<u32>,
        }

        #[derive(Facet)]
        #[repr(u8)]
        #[allow(unused)]
        enum EnumWithSet {
            SetVariant(BTreeSet<i64>),
            Regular(u32),
        }

        // This should complete successfully - no more panics on Set types
        let registry = reflect!(WithSets, EnumWithSet).unwrap();
        assert!(!registry.is_empty());
    }

    /// Regression test: Verify complex nested structures with sets work
    /// Previously would panic in `process_newtype_variant_with_temp_container`
    #[test]
    fn test_complex_nested_sets() {
        #[derive(Facet)]
        struct NestedSets {
            sets_in_vec: Vec<BTreeSet<String>>,
            optional_set: Option<HashSet<u64>>,
        }

        // All Variable placeholders should be resolved correctly
        let registry = reflect!(NestedSets).unwrap();
        assert!(!registry.is_empty());
    }

    /// Test that demonstrates what happens when Variables are properly resolved
    /// This shows the system working correctly now
    #[test]
    fn test_normal_variable_resolution() {
        #[derive(Facet)]
        struct SimpleStruct {
            field: u32,
        }

        // This should work fine - all Variables should be resolved
        let registry = reflect!(SimpleStruct).unwrap();
        assert!(!registry.is_empty());

        // Verify the type was processed correctly
        let type_name = QualifiedTypeName {
            namespace: Namespace::Root,
            name: "SimpleStruct".to_string(),
        };
        assert!(registry.contains_key(&type_name));
    }

    /// Diagnostic test: Verify `Variable::visit` still returns `UnknownFormat` for unresolved variables
    /// This demonstrates the underlying mechanism but should only occur in synthetic scenarios now
    #[test]
    #[should_panic(expected = "UnknownFormat")]
    fn test_variable_visit_mechanism() {
        use crate::reflection::format::{FormatHolder, Variable};

        // Create an unresolved Variable (this should only happen in test scenarios now)
        let variable: Variable<Format> = Variable::new(None);

        // Calling visit on an unresolved Variable should still return UnknownFormat error
        let result = variable.visit(&mut |_| Ok(()));

        // This demonstrates the underlying error mechanism
        result.unwrap();
    }

    /// Regression test: Verify that temporary containers now work correctly
    /// Previously, temp containers could be left with unresolved Variables
    #[test]
    fn test_temp_container_resolution() {
        #[derive(Facet)]
        #[repr(u8)]
        #[allow(unused)]
        enum WithNewtypeVariant {
            Variant(BTreeSet<String>),
        }

        // The temp container workflow in process_newtype_variant_with_temp_container
        // should now properly resolve all Variables
        let registry = reflect!(WithNewtypeVariant).unwrap();
        assert!(!registry.is_empty());
    }

    /// Test multiple types that previously caused issues to ensure they all work together
    #[test]
    fn test_comprehensive_previously_problematic_types() {
        use std::collections::{BTreeSet, HashMap, HashSet};

        #[derive(Facet)]
        struct Complex {
            // Set types (previously unhandled)
            btree_set: BTreeSet<String>,
            hash_set: HashSet<u32>,

            // Nested collections with sets
            nested: Vec<BTreeSet<i32>>,
            mapped: HashMap<String, HashSet<u64>>,

            // Optional collections
            maybe_set: Option<HashSet<i32>>,
        }

        #[derive(Facet)]
        #[repr(u8)]
        #[allow(unused)]
        enum ComplexEnum {
            // Newtype variants with sets (previously caused temp container issues)
            SetVariant(BTreeSet<String>),
            NestedVariant(Vec<HashSet<u32>>),
            OptionalVariant(Option<HashSet<i64>>),
            Regular(u32),
        }

        // All of these previously problematic patterns should now work
        let registry = reflect!(Complex, ComplexEnum).unwrap();
        assert!(!registry.is_empty());

        // Should have processed both types
        let mut found_types = 0;
        for key in registry.keys() {
            if key.name.contains("Complex") {
                found_types += 1;
            }
        }
        assert!(found_types >= 2, "Should have processed both Complex types");
    }

    /// Diagnostic test: Manually create unresolved Variables to verify `build()` catches them
    /// This simulates what used to happen in the problematic code paths
    #[test]
    fn test_build_catches_unresolved_variables() {
        let mut builder = RegistryBuilder::new();

        // Manually insert an unresolved Variable (simulating the old bug)
        let type_name = QualifiedTypeName {
            namespace: Namespace::Named("test".to_string()),
            name: "UnresolvedType".to_string(),
        };

        let unresolved_container = ContainerFormat::NewTypeStruct(Box::default(), Doc::new());
        builder
            .registry
            .insert(type_name.clone(), unresolved_container);

        assert_eq!(
            builder.build(),
            Err(Error::ReflectionError {
                type_name: type_name.to_string(),
                message: "incomplete reflection detected".to_string()
            })
        );
    }

    /// Test with enum variants that have unresolved Variables
    #[test]
    fn test_enum_with_unresolved_variant_caught() {
        let mut builder = RegistryBuilder::new();

        // Create an enum with an unresolved variant Variable
        let mut variants = BTreeMap::new();
        variants.insert(
            0,
            crate::reflection::format::Named {
                name: "UnresolvedVariant".to_string(),
                doc: Doc::new(),
                value: VariantFormat::unknown(), // Unresolved Variable
            },
        );

        let enum_container = ContainerFormat::Enum(variants, Doc::new());
        let type_name = QualifiedTypeName {
            namespace: Namespace::Named("test".to_string()),
            name: "EnumWithUnresolvedVariant".to_string(),
        };

        builder.registry.insert(type_name.clone(), enum_container);

        assert_eq!(
            builder.build(),
            Err(Error::ReflectionError {
                type_name: type_name.to_string(),
                message: "incomplete reflection detected".to_string()
            })
        );
    }

    /// Test with struct fields that have unresolved Variables
    #[test]
    fn test_struct_with_unresolved_field_caught() {
        let mut builder = RegistryBuilder::new();

        // Create a struct with an unresolved field Variable
        let fields = vec![crate::reflection::format::Named {
            name: "unresolved_field".to_string(),
            doc: Doc::new(),
            value: Format::unknown(), // Unresolved Variable
        }];

        let struct_container = ContainerFormat::Struct(fields, Doc::new());
        let type_name = QualifiedTypeName {
            namespace: Namespace::Named("test".to_string()),
            name: "StructWithUnresolvedField".to_string(),
        };

        builder.registry.insert(type_name.clone(), struct_container);

        assert_eq!(
            builder.build(),
            Err(Error::ReflectionError {
                type_name: type_name.to_string(),
                message: "incomplete reflection detected".to_string()
            })
        );
    }

    /// Regression test: Verify the original failing case now works
    /// This replicates the `set_of_string` test pattern
    #[test]
    fn test_original_set_of_string_pattern_works() {
        use std::collections::BTreeSet;

        #[derive(Facet)]
        #[repr(u8)]
        #[allow(unused)]
        enum MyEnum {
            Set(BTreeSet<String>),
            Other(u32),
        }

        // This used to panic, now it should work
        let registry = reflect!(MyEnum).unwrap();
        assert!(!registry.is_empty());

        // Should contain the enum type
        let type_name = QualifiedTypeName {
            namespace: Namespace::Root,
            name: "MyEnum".to_string(),
        };
        assert!(registry.contains_key(&type_name));
    }

    /// This will pass eventually, once we fully support generic types.
    #[test]
    fn test_enum_with_anon_struct_variants_with_result_of_t() {
        use thiserror::Error;

        #[derive(Facet, Clone, PartialEq, Eq, Error, Debug)]
        #[error("AnotherError")]
        struct AnotherError;

        #[derive(Facet, Clone, PartialEq, Eq, Error, Debug)]
        #[repr(C)]
        #[allow(unused)]
        pub enum MyError {
            #[error("disconnected")]
            Disconnect(#[from] AnotherError),
            #[error("the data for key `{0}` is not available")]
            Redaction(String),
            #[error("invalid header (expected {expected:?}, found {found:?})")]
            InvalidHeader { expected: String, found: String },
            #[error("unknown data store error")]
            Unknown,
        }
        #[derive(Facet, Debug, Clone, PartialEq, Eq)]
        #[repr(C)]
        #[allow(unused)]
        pub enum MyResult<T> {
            Ok(T),
            Err(MyError),
        }

        #[derive(Facet)]
        #[repr(C)]
        #[allow(unused)]
        enum MyEnum {
            Variant1 { result: MyResult<String> },
            Variant2 { result: MyResult<i32> },
        }

        let err = reflect!(MyEnum).unwrap_err();

        insta::assert_snapshot!(
            err.root_cause(),
            @"failed to add type MyEnum: unsupported generic type: MyResult<i32>, the type may have already been used with different parameters"
        );
    }
}
