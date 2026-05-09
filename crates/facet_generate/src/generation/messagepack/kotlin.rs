//! `EmitterPlugin<Kotlin>` implementation for [`MessagePackPlugin`].
//!
//! Provides MessagePack-specific imports, `@Serializable` / `@SerialName`
//! type annotations, field-level tuple serializer annotations, runtime
//! support files, and manifest dependencies for Kotlin code generation,
//! using `kotlinx-serialization-msgpack`.
//!
//! # Tuple wire-format compatibility
//!
//! Rust tuples like `(A, B)` are encoded by `rmp_serde` as a fixed-size
//! MessagePack array `[a, b]`.  Kotlin's default `Pair<A, B>` serializer
//! encodes as a map `{"first": a, "second": b}`, which does **not** match.
//!
//! This plugin fixes the mismatch at the field level by:
//!
//! 1. Shipping `PairAsArraySerializer` / `TripleAsArraySerializer` runtime
//!    files (in `com.facet.generate.msgpack`) that encode `Pair` / `Triple`
//!    as msgpack arrays.
//! 2. Annotating every `Pair<A, B>` or `Triple<A, B, C>` field with
//!    `@Serializable(with = PairAsArraySerializer::class)` (or the triple
//!    variant) so the compiler plugin selects the correct serializer.
//! 3. Adding the necessary `import` statements when [`Feature::Tuples`] is
//!    detected in the module being generated.

use crate::generation::{
    CodeGeneratorConfig, Feature,
    kotlin::Kotlin,
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
};
use crate::reflection::format::{Format, Named};

use super::MessagePackPlugin;

impl EmitterPlugin<Kotlin> for MessagePackPlugin {
    /// Returns the kotlinx-serialization-msgpack Gradle dependency.
    fn manifest_dependencies(&self) -> Vec<String> {
        vec![
            r#"    implementation("com.ensarsarajcic.kotlinx:serialization-msgpack:0.5.7")"#
                .to_string(),
        ]
    }

    /// Ships `PairAsArraySerializer.kt` and `TripleAsArraySerializer.kt`
    /// alongside the generated code.
    ///
    /// These serializers encode `Pair<A, B>` / `Triple<A, B, C>` as
    /// fixed-size msgpack arrays rather than maps, matching rmp-serde's
    /// wire format for Rust tuples.
    fn runtime_files(&self) -> Vec<RuntimeFile> {
        static MSGPACK: include_dir::Dir<'static> = include_dir::include_dir!(
            "$CARGO_MANIFEST_DIR/runtime/kotlin/com/facet/generate/msgpack"
        );
        MSGPACK
            .files()
            .map(|f| RuntimeFile {
                relative_path: format!(
                    "com/facet/generate/msgpack/{}",
                    f.path().file_name().unwrap_or_default().to_string_lossy()
                ),
                contents: f.contents().to_vec(),
            })
            .collect()
    }

    /// `MessagePack` / kotlinx.serialization imports for a Kotlin module.
    ///
    /// Always emits `Serializable` and `SerialName`.  When [`Feature::Tuples`]
    /// is present the serializer classes for `Pair` / `Triple` are also
    /// imported so the generated `@Serializable(with = …)` field annotations
    /// compile correctly.
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let mut imports = vec![
            "import kotlinx.serialization.Serializable".to_string(),
            "import kotlinx.serialization.SerialName".to_string(),
        ];

        if config.features.contains(&Feature::Tuples) {
            imports.push("import com.facet.generate.msgpack.PairAsArraySerializer".to_string());
            imports.push("import com.facet.generate.msgpack.TripleAsArraySerializer".to_string());
        }

        imports
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

    /// `@Serializable(with = …)` annotation for `Pair` / `Triple` fields.
    ///
    /// Without this override, kotlinx-serialization uses Kotlin's default
    /// `Pair` / `Triple` descriptors, which encode as named maps and do not
    /// round-trip with rmp-serde's array encoding for Rust tuples.
    fn field_annotations(
        &self,
        field: &Named<crate::reflection::format::Format>,
        _ctx: &EmitContext,
    ) -> Vec<String> {
        match &field.value {
            Format::Tuple(formats) if formats.len() == 2 => {
                vec!["@Serializable(with = PairAsArraySerializer::class)".to_string()]
            }
            Format::Tuple(formats) if formats.len() == 3 => {
                vec!["@Serializable(with = TripleAsArraySerializer::class)".to_string()]
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::plugin::VariantInfo;
    use crate::generation::{CodeGeneratorConfig, Container, Feature};
    use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName, VariantFormat};
    use std::collections::BTreeSet;

    fn make_config(features: &[Feature]) -> CodeGeneratorConfig {
        let mut cfg = CodeGeneratorConfig::new("com.example".to_string());
        cfg.features = features.iter().copied().collect::<BTreeSet<_>>();
        cfg
    }

    fn make_field(name: &str, value: Format) -> Named<Format> {
        Named::new(&value, name.to_string())
    }

    // -------------------------------------------------------------------------
    // imports
    // -------------------------------------------------------------------------

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
    fn imports_tuple_feature_adds_pair_and_triple_serializer_imports() {
        let cfg = make_config(&[Feature::Tuples]);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let imports = plugin.imports(&cfg);

        assert!(
            imports.iter().any(|i| i.contains("PairAsArraySerializer")),
            "expected PairAsArraySerializer in imports, got: {imports:?}"
        );
        assert!(
            imports
                .iter()
                .any(|i| i.contains("TripleAsArraySerializer")),
            "expected TripleAsArraySerializer in imports, got: {imports:?}"
        );
    }

    #[test]
    fn imports_no_tuple_serializers_without_feature() {
        let cfg = make_config(&[]);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let imports = plugin.imports(&cfg);

        assert!(
            !imports.iter().any(|i| i.contains("AsArraySerializer")),
            "unexpected array serializer import without Tuples feature: {imports:?}"
        );
    }

    // -------------------------------------------------------------------------
    // field_annotations
    // -------------------------------------------------------------------------

    #[test]
    fn field_annotations_pair_field_gets_pair_serializer() {
        let config = make_config(&[]);
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;

        let field = make_field("b", Format::Tuple(vec![Format::I64, Format::U64]));
        let annotations = plugin.field_annotations(&field, &ctx);

        assert_eq!(annotations.len(), 1);
        assert!(
            annotations[0].contains("PairAsArraySerializer"),
            "expected PairAsArraySerializer annotation, got: {:?}",
            annotations[0]
        );
    }

    #[test]
    fn field_annotations_triple_field_gets_triple_serializer() {
        let config = make_config(&[]);
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;

        let field = make_field(
            "t",
            Format::Tuple(vec![Format::I64, Format::U64, Format::I32]),
        );
        let annotations = plugin.field_annotations(&field, &ctx);

        assert_eq!(annotations.len(), 1);
        assert!(
            annotations[0].contains("TripleAsArraySerializer"),
            "expected TripleAsArraySerializer annotation, got: {:?}",
            annotations[0]
        );
    }

    #[test]
    fn field_annotations_non_tuple_field_returns_empty() {
        let config = make_config(&[]);
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;

        let field = make_field("x", Format::I32);
        assert!(plugin.field_annotations(&field, &ctx).is_empty());

        let field = make_field("s", Format::Str);
        assert!(plugin.field_annotations(&field, &ctx).is_empty());
    }

    #[test]
    fn field_annotations_1_tuple_returns_empty() {
        // Single-element tuples are treated as the element itself — no annotation.
        let config = make_config(&[]);
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;

        let field = make_field("x", Format::Tuple(vec![Format::I32]));
        assert!(plugin.field_annotations(&field, &ctx).is_empty());
    }

    // -------------------------------------------------------------------------
    // runtime_files
    // -------------------------------------------------------------------------

    #[test]
    fn runtime_files_contains_pair_and_triple_serializers() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let files = plugin.runtime_files();

        let paths: Vec<&str> = files.iter().map(|f| f.relative_path.as_str()).collect();
        assert!(
            paths.iter().any(|p| p.contains("PairAsArraySerializer")),
            "expected PairAsArraySerializer in runtime files, got: {paths:?}"
        );
        assert!(
            paths.iter().any(|p| p.contains("TripleAsArraySerializer")),
            "expected TripleAsArraySerializer in runtime files, got: {paths:?}"
        );
    }

    #[test]
    fn runtime_files_are_non_empty() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<Kotlin>;
        let files = plugin.runtime_files();
        assert!(!files.is_empty(), "runtime_files should not be empty");
        for f in &files {
            assert!(
                !f.contents.is_empty(),
                "runtime file {} should not be empty",
                f.relative_path
            );
        }
    }

    // -------------------------------------------------------------------------
    // manifest + type annotations (unchanged, kept for regression)
    // -------------------------------------------------------------------------

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
    fn type_annotations_include_serializable_and_serial_name() {
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
