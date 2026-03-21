//! `EmitterPlugin<Kotlin>` implementation for the [`JsonPlugin`].
//!
//! Provides JSON-specific imports, `@Serializable` / `@SerialName` type
//! annotations, `BigInt` helper snippets, and the `serialName` accessor for
//! all-unit enum classes.

use std::io;

use crate::generation::{
    CodeGeneratorConfig, Feature,
    indent::IndentWrite,
    kotlin::Kotlin,
    plugin::{EmitContext, EmitterPlugin},
};
use crate::reflection::format::{ContainerFormat, VariantFormat};

use super::JsonPlugin;

/// The `BigInt` JSON helper — a custom `KSerializer<BigInteger>` that
/// round-trips `BigInteger` values through JSON unquoted literals.
const FEATURE_BIGINT: &[u8] = include_bytes!("../../generation/kotlin/emitter/features/BigInt.kt");

impl EmitterPlugin<Kotlin> for JsonPlugin {
    /// JSON / kotlinx.serialization imports for a Kotlin module.
    ///
    /// Returns the base `Serializable` and `SerialName` imports, plus
    /// feature-specific imports when `BigInt` types are present.
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let mut imports = vec![
            "import kotlinx.serialization.Serializable".to_string(),
            "import kotlinx.serialization.SerialName".to_string(),
        ];

        // BigInt JSON-specific imports
        if config.features.contains(&Feature::BigInt) {
            imports.extend([
                "import kotlinx.serialization.KSerializer".to_string(),
                "import kotlinx.serialization.descriptors.PrimitiveKind".to_string(),
                "import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor".to_string(),
                "import kotlinx.serialization.encoding.Decoder".to_string(),
                "import kotlinx.serialization.encoding.Encoder".to_string(),
                "import kotlinx.serialization.json.JsonDecoder".to_string(),
                "import kotlinx.serialization.json.JsonEncoder".to_string(),
                "import kotlinx.serialization.json.JsonUnquotedLiteral".to_string(),
                "import kotlinx.serialization.json.jsonPrimitive".to_string(),
            ]);
        }

        imports
    }

    /// `BigInt` JSON helper snippet for a Kotlin module.
    ///
    /// When `BigInt` types are present, emits the custom `KSerializer<BigInteger>`
    /// that serializes big integers as unquoted JSON number literals.
    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        if config.features.contains(&Feature::BigInt) {
            w.write_all(FEATURE_BIGINT)?;
            writeln!(w)?;
        }
        Ok(())
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
    ///
    /// This preserves the original Rust variant name as the serialized tag,
    /// while Kotlin's convention uppercases the entry identifier.
    fn enum_variant_annotations(&self, name: &str) -> Vec<String> {
        vec![format!(r#"@SerialName("{name}")"#)]
    }

    /// For all-unit enum classes, emits the `serialName` computed property
    /// that extracts the `@SerialName` annotation value at runtime.
    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        // Only applies to all-unit enum classes
        if ctx.is_variant() {
            return Ok(());
        }

        if let ContainerFormat::Enum(variants, _) = ctx.container.format {
            let all_unit = variants
                .values()
                .all(|v| matches!(v.value, VariantFormat::Unit));

            if all_unit {
                writeln!(w)?;
                writeln!(w, "val serialName: String")?;
                writeln!(
                    w,
                    "    get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value"
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::CodeGeneratorConfig;
    use std::collections::BTreeSet;

    fn make_config(features: &[Feature]) -> CodeGeneratorConfig {
        let mut cfg = CodeGeneratorConfig::new("com.example".to_string());
        cfg.features = features.iter().copied().collect::<BTreeSet<_>>();
        cfg
    }

    #[test]
    fn base_imports_are_present() {
        let cfg = make_config(&[]);
        let plugin = JsonPlugin::new();
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("Serializable")));
        assert!(imports.iter().any(|i| i.contains("SerialName")));
    }

    #[test]
    fn bigint_adds_json_imports() {
        let cfg = make_config(&[Feature::BigInt]);
        let plugin = JsonPlugin::new();
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("KSerializer")));
        assert!(imports.iter().any(|i| i.contains("PrimitiveKind")));
        assert!(imports.iter().any(|i| i.contains("JsonDecoder")));
        assert!(imports.iter().any(|i| i.contains("JsonEncoder")));
        assert!(imports.iter().any(|i| i.contains("JsonUnquotedLiteral")));
    }

    #[test]
    fn bigint_module_helpers_emit_feature() {
        let cfg = make_config(&[Feature::BigInt]);
        let plugin = JsonPlugin::new();

        let mut buf = Vec::new();
        {
            use crate::generation::indent::IndentedWriter;
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin.module_helpers(&mut w, &cfg).unwrap();
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn type_annotations_include_serializable_and_serial_name() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);
        let plugin = JsonPlugin::new();
        let annotations = plugin.type_annotations(&ctx);

        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0], "@Serializable");
        assert_eq!(annotations[1], r#"@SerialName("Foo")"#);
    }
}
