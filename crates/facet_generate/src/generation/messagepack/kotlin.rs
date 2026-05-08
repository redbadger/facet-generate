//! `EmitterPlugin<Kotlin>` implementation for [`MessagePackPlugin`].
//!
//! Provides MessagePack-specific imports, `@Serializable` / `@SerialName`
//! type annotations, and manifest dependencies for Kotlin code generation,
//! using `kotlinx-serialization-msgpack`.

use crate::generation::{
    CodeGeneratorConfig,
    kotlin::Kotlin,
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
};

use super::MessagePackPlugin;

impl EmitterPlugin<Kotlin> for MessagePackPlugin {
    /// Returns the kotlinx-serialization-msgpack Gradle dependency.
    fn manifest_dependencies(&self) -> Vec<String> {
        vec![
            r#"    implementation("com.ensarsarajcic.kotlinx:serialization-msgpack:0.5.7")"#
                .to_string(),
        ]
    }

    /// No bundled runtime files — the library arrives via Gradle.
    fn runtime_files(&self) -> Vec<RuntimeFile> {
        vec![]
    }

    /// `MessagePack` / kotlinx.serialization imports for a Kotlin module.
    ///
    /// Returns the base `Serializable` and `SerialName` imports.
    /// Unlike the JSON plugin, no BigInt-specific imports are needed.
    fn imports(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec![
            "import kotlinx.serialization.Serializable".to_string(),
            "import kotlinx.serialization.SerialName".to_string(),
        ]
    }

    /// `@Serializable` and `@SerialName("…")` annotations for each type.
    ///
    /// These are emitted on separate lines above every `data class`,
    /// `data object`, `enum class`, and `sealed interface`.
    fn type_annotations(&self, ctx: &EmitContext) -> Vec<String> {
        let name = ctx.name();
        vec![
            "@Serializable".to_string(),
            format!(r#"@SerialName("{name}")"#),
        ]
    }

    /// `@SerialName("…")` inline annotation for each all-unit enum class
    /// variant.
    ///
    /// Emitted on the same line as the uppercased variant name, e.g.:
    ///
    /// ```text
    /// @SerialName("Variant1") VARIANT1,
    /// ```
    fn enum_variant_annotations(&self, name: &str) -> Vec<String> {
        vec![format!(r#"@SerialName("{name}")"#)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{CodeGeneratorConfig, Feature};
    use std::collections::BTreeSet;

    fn make_config(features: &[Feature]) -> CodeGeneratorConfig {
        let mut cfg = CodeGeneratorConfig::new("com.example".to_string());
        cfg.features = features.iter().copied().collect::<BTreeSet<_>>();
        cfg
    }

    #[test]
    fn imports_returns_serializable_and_serial_name() {
        let cfg = make_config(&[]);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("Serializable")));
        assert!(imports.iter().any(|i| i.contains("SerialName")));
    }

    #[test]
    fn imports_no_bigint_imports() {
        let cfg = make_config(&[Feature::BigInt]);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let imports = plugin.imports(&cfg);

        assert!(!imports.iter().any(|i| i.contains("KSerializer")));
        assert!(!imports.iter().any(|i| i.contains("JsonDecoder")));
    }

    #[test]
    fn manifest_dependency_contains_msgpack() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let deps = plugin.manifest_dependencies();

        assert!(
            deps.iter()
                .any(|d| d.contains("serialization-msgpack") && d.contains("0.5.7"))
        );
    }

    #[test]
    fn runtime_files_is_empty() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        assert!(plugin.runtime_files().is_empty());
    }

    #[test]
    fn type_annotations_include_serializable_and_serial_name() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

        let config = make_config(&[]);
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let annotations = plugin.type_annotations(&ctx);

        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0], "@Serializable");
        assert_eq!(annotations[1], r#"@SerialName("Foo")"#);
    }

    #[test]
    fn type_annotations_for_variant() {
        use crate::generation::Container;
        use crate::generation::plugin::VariantInfo;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName, VariantFormat};

        let config = make_config(&[]);
        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let vf = VariantFormat::Unit;
        let variant = VariantInfo {
            name: "Ok",
            index: 0,
            format: &vf,
            fields: &[],
            parent_name: "MyEnum",
        };
        let ctx = EmitContext::for_variant(&container, &config, variant);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let annotations = plugin.type_annotations(&ctx);

        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0], "@Serializable");
        assert_eq!(annotations[1], r#"@SerialName("Ok")"#);
    }

    #[test]
    fn enum_variant_annotations_returns_serial_name() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let annotations = plugin.enum_variant_annotations("Red");

        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0], r#"@SerialName("Red")"#);
    }
}
