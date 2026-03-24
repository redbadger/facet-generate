//! `EmitterPlugin<CSharp>` implementation for the [`JsonPlugin`].
//!
//! Provides JSON-specific code generation for C# types: `System.Text.Json`
//! `using` directives, `[JsonPropertyName]` / `[JsonPolymorphic]` /
//! `[JsonDerivedType]` / `[JsonConverter]` annotations, and
//! `JsonSerialize` / `JsonDeserialize` convenience methods.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `using Facet.Runtime.Json;` + `using System.Text.Json.Serialization;` |
//! | `type_annotations` | `[JsonConverter]` (unit enums), `[JsonPolymorphic]` + `[JsonDerivedType(…)]` (variant hierarchies) |
//! | `field_annotations` | `[JsonPropertyName("camelCaseName")]` |
//! | `has_type_body` | `true` for non-unit-enum types |
//! | `type_body` | `JsonSerialize` / `JsonDeserialize` static helper methods |

use std::io;

use heck::{ToLowerCamelCase, ToUpperCamelCase};

use crate::generation::{
    CodeGeneratorConfig,
    csharp::CSharp,
    indent::{IndentWrite, Newlines, with_block},
    plugin::{EmitContext, EmitterPlugin},
};
use crate::reflection::format::{ContainerFormat, Format, Named, VariantFormat};

use super::JsonPlugin;

// ---------------------------------------------------------------------------
// EmitterPlugin implementation
// ---------------------------------------------------------------------------

impl EmitterPlugin<CSharp> for JsonPlugin {
    /// Returns `using` directives for JSON support.
    fn imports(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec![
            "using Facet.Runtime.Json;".to_string(),
            "using System.Text.Json.Serialization;".to_string(),
        ]
    }

    /// Emits JSON type-level annotations.
    ///
    /// - All-unit enum → `[JsonConverter(typeof(JsonStringEnumConverter))]`
    /// - Non-unit enum (variant hierarchy) → `[JsonPolymorphic(…)]` +
    ///   one `[JsonDerivedType(…)]` per variant
    /// - Everything else → nothing
    fn type_annotations(&self, ctx: &EmitContext) -> Vec<String> {
        match ctx.container.format {
            ContainerFormat::Enum(variants, _) => {
                let all_unit = variants
                    .values()
                    .all(|v| matches!(v.value, VariantFormat::Unit));

                if all_unit {
                    vec!["[JsonConverter(typeof(JsonStringEnumConverter))]".to_string()]
                } else {
                    let mut annotations = vec![
                        "[JsonPolymorphic(TypeDiscriminatorPropertyName = \"type\")]".to_string(),
                    ];
                    for variant in variants.values() {
                        let variant_name = variant.name.to_upper_camel_case();
                        annotations.push(format!(
                            "[JsonDerivedType(typeof({variant_name}), \"{}\")]",
                            variant.name
                        ));
                    }
                    annotations
                }
            }
            _ => vec![],
        }
    }

    /// Emits `[JsonPropertyName("camelCaseName")]` before each field.
    fn field_annotations(&self, field: &Named<Format>, _ctx: &EmitContext) -> Vec<String> {
        vec![format!(
            "[JsonPropertyName(\"{}\")]",
            field.name.to_lower_camel_case()
        )]
    }

    /// Returns `true` for types that need `JsonSerialize` / `JsonDeserialize`
    /// methods — i.e. everything except all-unit enums (plain C# `enum` types
    /// don't need instance helpers).
    fn has_type_body(&self, ctx: &EmitContext) -> bool {
        if let ContainerFormat::Enum(variants, _) = ctx.container.format
            && variants
                .values()
                .all(|v| matches!(v.value, VariantFormat::Unit))
        {
            return false;
        }
        true
    }

    /// Emits `JsonSerialize` and `JsonDeserialize` convenience methods.
    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        let type_name = ctx.name().to_upper_camel_case();
        write_json_helpers(w, &type_name)
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Writes `JsonSerialize` / `JsonDeserialize` methods backed by `JsonSerde`.
fn write_json_helpers(w: &mut dyn IndentWrite, type_name: &str) -> io::Result<()> {
    writeln!(w, "public string JsonSerialize()")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "return JsonSerde.Serialize(this);")
    })?;
    writeln!(w)?;
    writeln!(w, "public static {type_name} JsonDeserialize(string input)")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "return JsonSerde.Deserialize<{type_name}>(input);")
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{
        CodeGeneratorConfig, Container,
        indent::{IndentConfig, IndentedWriter},
        plugin::EmitContext,
    };
    use crate::reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName};

    fn render(f: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>) -> String {
        let mut buf = Vec::new();
        let mut w = IndentedWriter::new(&mut buf, IndentConfig::Space(4));
        f(&mut w).unwrap();
        String::from_utf8(buf).unwrap()
    }

    // -------------------------------------------------------------------------
    // imports
    // -------------------------------------------------------------------------

    #[test]
    fn imports_returns_json_usings() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let imports = plugin.imports(&cfg);
        assert!(imports.iter().any(|i| i.contains("Facet.Runtime.Json")));
        assert!(
            imports
                .iter()
                .any(|i| i.contains("Text.Json.Serialization"))
        );
    }

    // -------------------------------------------------------------------------
    // type_annotations
    // -------------------------------------------------------------------------

    #[test]
    fn type_annotations_unit_enum() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = std::collections::BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "Alpha".to_string()));
        variants.insert(1u32, Named::new(&VariantFormat::Unit, "Beta".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let annotations = plugin.type_annotations(&ctx);
        assert_eq!(annotations.len(), 1);
        assert!(
            annotations[0].contains("JsonStringEnumConverter"),
            "{annotations:?}"
        );
    }

    #[test]
    fn type_annotations_variant_hierarchy() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = std::collections::BTreeMap::new();
        variants.insert(
            0u32,
            Named::new(
                &VariantFormat::NewType(Box::new(Format::Str)),
                "Ok".to_string(),
            ),
        );
        variants.insert(
            1u32,
            Named::new(
                &VariantFormat::NewType(Box::new(Format::I32)),
                "Err".to_string(),
            ),
        );

        let name = QualifiedTypeName::root("Result".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let annotations = plugin.type_annotations(&ctx);
        assert!(
            annotations.iter().any(|a| a.contains("JsonPolymorphic")),
            "{annotations:?}"
        );
        assert!(
            annotations
                .iter()
                .any(|a| a.contains("JsonDerivedType") && a.contains("Ok")),
            "{annotations:?}"
        );
        assert!(
            annotations
                .iter()
                .any(|a| a.contains("JsonDerivedType") && a.contains("Err")),
            "{annotations:?}"
        );
    }

    #[test]
    fn type_annotations_struct_returns_empty() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        assert!(plugin.type_annotations(&ctx).is_empty());
    }

    // -------------------------------------------------------------------------
    // field_annotations
    // -------------------------------------------------------------------------

    #[test]
    fn field_annotations_returns_json_property_name() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let field = Named::new(&Format::Str, "firstName".to_string());

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let annotations = plugin.field_annotations(&field, &ctx);
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0], "[JsonPropertyName(\"firstName\")]");
    }

    #[test]
    fn field_annotations_pascal_case_to_camel_case() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let field = Named::new(&Format::I32, "MyField".to_string());

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let annotations = plugin.field_annotations(&field, &ctx);
        assert_eq!(annotations[0], "[JsonPropertyName(\"myField\")]");
    }

    // -------------------------------------------------------------------------
    // has_type_body
    // -------------------------------------------------------------------------

    #[test]
    fn has_type_body_true_for_struct() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        assert!(plugin.has_type_body(&ctx));
    }

    #[test]
    fn has_type_body_false_for_unit_enum() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = std::collections::BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "A".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        assert!(!plugin.has_type_body(&ctx));
    }

    // -------------------------------------------------------------------------
    // type_body
    // -------------------------------------------------------------------------

    #[test]
    fn type_body_emits_json_serialize_and_deserialize() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let name = QualifiedTypeName::root("MyRecord".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let out = render(|w| plugin.type_body(w, &ctx));
        assert!(out.contains("public string JsonSerialize()"), "{out}");
        assert!(out.contains("return JsonSerde.Serialize(this);"), "{out}");
        assert!(
            out.contains("public static MyRecord JsonDeserialize(string input)"),
            "{out}"
        );
        assert!(
            out.contains("return JsonSerde.Deserialize<MyRecord>(input);"),
            "{out}"
        );
    }
}
