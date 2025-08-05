// This test verifies that unit structs created without bracket syntax can still be generated.

use facet::Facet;

#[derive(Facet)]
struct UnitStruct;

mod tests {
    use crate::check;

    use super::*;

    use anyhow::Result;
    use expect_test::expect_file;
    use facet_generate::{
        generation::{CodeGeneratorConfig, Language, java, swift, typescript},
        reflection::RegistryBuilder,
    };

    #[test]
    #[ignore = "TODO"]
    fn can_generate_unit_structs_java() -> Result<()> {
        let registry = RegistryBuilder::new().add_type::<UnitStruct>().build();
        let cfg = CodeGeneratorConfig::new("com.example.test".to_string()).without_serialization();
        let lang = <java::CodeGenerator as Language>::new(&cfg);
        let expect = expect_file!("./output.java");

        check(&registry, lang, &expect)?;

        Ok(())
    }

    #[test]
    #[ignore = "TODO"]
    fn can_generate_unit_structs_swift() -> Result<()> {
        let registry = RegistryBuilder::new().add_type::<UnitStruct>().build();
        let cfg = CodeGeneratorConfig::new("Test".to_string()).without_serialization();
        let lang = <swift::CodeGenerator as Language>::new(&cfg);
        let expect = expect_file!("./output.swift");

        check(&registry, lang, &expect)?;

        Ok(())
    }

    #[test]
    #[ignore = "TODO"]
    fn can_generate_unit_structs_typescript() -> Result<()> {
        let registry = RegistryBuilder::new().add_type::<UnitStruct>().build();
        let cfg = CodeGeneratorConfig::new("test".to_string()).without_serialization();
        let lang = <typescript::CodeGenerator as Language>::new(&cfg);
        let expect = expect_file!("./output.ts");

        check(&registry, lang, &expect)?;

        Ok(())
    }
}
