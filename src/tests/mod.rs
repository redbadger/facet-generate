use crate::{Registry, generation::CodeGen};
use anyhow::Result;
use expect_test::ExpectFile;

fn check<'a, L: CodeGen<'a>>(registry: &Registry, mut lang: L, expect: &ExpectFile) -> Result<()> {
    let mut output: Vec<u8> = Vec::new();

    lang.write_output(&mut output, registry)?;

    let actual = String::from_utf8(output)?;
    expect.assert_eq(&actual);

    Ok(())
}

/// Register the given types and output code for the given languages.
/// e.g.
/// ```rust
/// test! {
///    UnitStruct, AnotherType for java, swift, typescript
/// }
/// ```
#[macro_export]
macro_rules! test {
    ($($ty:ident),* for $($language:ident),*) => {
        mod tests {
            use $crate::{tests::check, test};
            use super::*;

            use anyhow::Result;
            use expect_test::expect_file;
            use $crate::{
                generation::{CodeGen, CodeGeneratorConfig},
                reflection::RegistryBuilder,
            };
            use $crate::generation::{$($language),*};

            test!(@generate_tests [$($ty),*] $($language),*);
        }
    };

    (@generate_tests [$($ty:ident),*] $language:ident $(, $rest:ident)*) => {
        #[test]
        fn $language() -> Result<()> {
            let registry = RegistryBuilder::new()
                $(.add_type::<$ty>())*
                .build();
            let package_name = test!(@package $language).to_string();
            let cfg = CodeGeneratorConfig::new(package_name);
            let generator = <$language::CodeGenerator as CodeGen>::new(&cfg);
            let expect = expect_file!(test!(@out $language));

            check(&registry, generator, &expect)?;
            // panic!("This test should fail");

            Ok(())
        }

        test!(@generate_tests [$($ty),*] $($rest),*);
    };

    (@generate_tests [$($ty:ident),*]) => {};

    (@package java) => { "com.example" };
    (@package kotlin) => { "com.example" };
    (@package swift) => { "ExamplePackage" };
    (@package typescript) => { "example_package" };

    (@out java) => { "output.java" };
    (@out kotlin) => { "output.kt" };
    (@out swift) => { "output.swift" };
    (@out typescript) => { "output.ts" };
}

mod adjacently_tagged_enum_decorator;
mod anonymous_struct_with_rename;
mod can_apply_namespace_correctly;
mod can_generate_adjacently_tagged_enum;
mod can_generate_adjacently_tagged_enum_with_skipped_variants;
mod can_generate_anonymous_struct_with_skipped_fields;
mod can_generate_bare_string_enum;
mod can_generate_empty_adjacently_tagged_enum;
mod can_generate_empty_externally_tagged_enum;
mod can_generate_empty_internally_tagged_enum;
mod can_generate_externally_tagged_enum;
mod can_generate_generic_adjacently_tagged_enum;
mod can_generate_generic_struct;
mod can_generate_generic_type_alias;
mod can_generate_internally_tagged_enum;
mod can_generate_readonly_fields;
mod can_generate_simple_struct_with_a_comment;
mod can_generate_slice_of_user_type;
mod can_generate_struct_with_skipped_fields;
mod can_generate_unit_enum;
mod can_generate_unit_structs;
mod can_handle_anonymous_struct;
mod can_handle_quote_in_serde_rename;
mod can_handle_serde_rename;
mod can_handle_serde_rename_all;
mod can_handle_serde_rename_on_top_level;
mod can_handle_space_in_serde_rename;
mod can_handle_unit_type;
mod can_override_types;
mod can_recognize_types_inside_modules;
mod deprecation_notice;
mod enum_with_discriminant;
mod generate_types;
mod generate_types_with_keywords;
mod generates_empty_structs_and_initializers;
mod kebab_case_rename;
mod orders_types;
mod parcelize_decorator;
mod property_wrapper;
mod recursive_adjacently_tagged_enum_decorator;
mod resolves_qualified_type;
mod serialize_anonymous_field_as;
mod serialize_field_as;
mod serialize_type_alias;
mod smart_pointers;
mod struct_decorator;
mod test_adjacently_tagged_enum_case_name_support;
mod test_branded_type_alias;
mod test_default_decorators;
mod test_default_generic_constraints;
mod test_generate_char;
mod test_internally_tagged_enum_case_name_support;
mod test_keypath_mutable_atomic_enum_field;
mod test_keypath_mutable_enum;
mod test_keypath_mutable_nested_optional;
mod test_keypath_mutable_nested_type;
mod test_keypath_mutable_optional;
mod test_keypath_mutable_simple;
mod test_only_attribute;
mod test_optional_type_alias;
mod test_preamble;
mod test_serde_default_struct;
mod test_serde_iso8601;
mod test_serde_url;
mod test_serialized_as;
mod test_serialized_as_tuple;
mod test_skip_by_language;
mod test_type_alias;
mod test_unit_enum_case_name_support;
mod test_unit_enum_serde_other;
mod test_visibility_modifiers;
mod unit_enum_decorator;
mod unit_enum_is_properly_named_with_serde_overrides;
mod use_correct_decoded_variable_name;
mod use_correct_integer_types;
