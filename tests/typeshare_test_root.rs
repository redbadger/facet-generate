use anyhow::Result;
use expect_test::ExpectFile;
use facet_generate::{Registry, generation::Language};

mod typeshare_tests;

fn check<'a, L: Language<'a>>(registry: &Registry, mut lang: L, expect: &ExpectFile) -> Result<()> {
    let mut output: Vec<u8> = Vec::new();

    lang.write_output(&mut output, registry)?;

    let actual = String::from_utf8(output)?;
    expect.assert_eq(&actual);

    Ok(())
}

/// Register the given types and output code for the given languages.
/// e.g.
/// ```rust
/// tests! {
///    UnitStruct, AnotherType for java, swift, typescript
/// }
/// ```
#[macro_export]
macro_rules! tests {
    ($($ty:ident),* for $($language:ident),*) => {
        mod tests {
            use $crate::{check, tests};
            use super::*;

            use anyhow::Result;
            use expect_test::expect_file;
            use facet_generate::{
                generation::{CodeGeneratorConfig, Language},
                reflection::RegistryBuilder,
            };
            use facet_generate::generation::{$($language),*};

            tests!(@generate_tests [$($ty),*] $($language),*);
        }
    };

    (@generate_tests [$($ty:ident),*] $language:ident $(, $rest:ident)*) => {
        #[test]
        #[ignore = "TODO"]
        fn $language() -> Result<()> {
            let registry = RegistryBuilder::new()
                $(.add_type::<$ty>())*
                .build();
            let package_name = tests!(@package $language).to_string();
            let cfg = CodeGeneratorConfig::new(package_name).without_serialization();
            let generator = <$language::CodeGenerator as Language>::new(&cfg);
            let expect = expect_file!(tests!(@out $language));

            check(&registry, generator, &expect)?;

            Ok(())
        }

        tests!(@generate_tests [$($ty),*] $($rest),*);
    };

    (@generate_tests [$($ty:ident),*]) => {};

    (@package java) => { "example.com" };
    (@package kotlin) => { "example.com" };
    (@package swift) => { "ExamplePackage" };
    (@package typescript) => { "example_package" };

    (@out java) => { "output.java" };
    (@out kotlin) => { "output.kt" };
    (@out swift) => { "output.swift" };
    (@out typescript) => { "output.ts" };
}
