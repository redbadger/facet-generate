//! `EmitterPlugin<CSharp>` implementation for [`MessagePackPlugin`].
//!
//! Provides MessagePack-specific code generation for C# types using
//! `Nerdbank.MessagePack` with a per-module `PolyType` witness class.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `using Facet.Runtime.MessagePack;` + NerdBank + PolyType |
//! | `module_helpers` | `FacetMessagePackWitness` partial class with `[GenerateShapeFor<T>]` |
//! | `type_annotations` | `[MessagePackConverter]` on enums |
//! | `has_type_body` | `true` for non-unit-enum types |
//! | `type_body` | Nested `TypeNameConverter` class + `MessagePackSerialize`/`MessagePackDeserialize` |
//! | `manifest_dependencies` | `Nerdbank.MessagePack` + `PolyType` package refs |

use std::collections::BTreeMap;
use std::io;

use heck::{ToLowerCamelCase, ToUpperCamelCase};

use crate::generation::{
    CodeGeneratorConfig,
    csharp::CSharp,
    indent::{IndentWrite, Newlines, with_block},
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
};
use crate::reflection::format::{ContainerFormat, Format, Named, VariantFormat};

use super::MessagePackPlugin;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Returns true for C# value types (primitives, structs) that use `Nullable<T>`
/// rather than nullable reference types.
fn is_value_type(format: &Format) -> bool {
    matches!(
        format,
        Format::Bool
            | Format::I8
            | Format::I16
            | Format::I32
            | Format::I64
            | Format::U8
            | Format::U16
            | Format::U32
            | Format::U64
            | Format::F32
            | Format::F64
            | Format::Char
            | Format::I128
            | Format::U128
            | Format::Unit
    )
}

/// Convert a [`Format`] to its C# type name.
///
/// Mirrors the private `csharp_type()` in the emitter, which is not accessible
/// from the plugin.
fn to_csharp_type(format: &Format) -> String {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not reach the plugin"),
        Format::TypeName(qtn) => qtn.name.to_upper_camel_case(),
        Format::Unit => "Unit".to_string(),
        Format::Bool => "bool".to_string(),
        Format::I8 => "sbyte".to_string(),
        Format::I16 => "short".to_string(),
        Format::I32 => "int".to_string(),
        Format::I64 => "long".to_string(),
        Format::I128 => "Int128".to_string(),
        Format::U8 => "byte".to_string(),
        Format::U16 => "ushort".to_string(),
        Format::U32 => "uint".to_string(),
        Format::U64 => "ulong".to_string(),
        Format::U128 => "UInt128".to_string(),
        Format::F32 => "float".to_string(),
        Format::F64 => "double".to_string(),
        Format::Char => "char".to_string(),
        Format::Str => "string".to_string(),
        Format::Bytes => "byte[]".to_string(),
        Format::Option(inner) => format!("{}?", to_csharp_type(inner)),
        Format::Seq(inner) => format!("ObservableCollection<{}>", to_csharp_type(inner)),
        Format::Set(inner) => format!("HashSet<{}>", to_csharp_type(inner)),
        Format::Map { key, value } => {
            format!(
                "Dictionary<{}, {}>",
                to_csharp_type(key),
                to_csharp_type(value)
            )
        }
        Format::Tuple(formats) => {
            if formats.is_empty() {
                return "Unit".to_string();
            }
            let values = formats
                .iter()
                .map(to_csharp_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({values})")
        }
        Format::TupleArray { content, .. } => format!("{}[]", to_csharp_type(content)),
    }
}

/// Emit a read expression for the given format.
///
/// For scalar primitives we use the direct `reader.ReadXxx()` methods (as
/// recommended by the Nerdbank.MessagePack custom-converter docs).  For all
/// other types we delegate to `context.GetConverter<T>(null).Read(…)`.
fn read_expr(format: &Format) -> String {
    match format {
        Format::Bool => "reader.ReadBoolean()".to_string(),
        Format::I8 => "reader.ReadSByte()".to_string(),
        Format::I16 => "reader.ReadInt16()".to_string(),
        Format::I32 => "reader.ReadInt32()".to_string(),
        Format::I64 => "reader.ReadInt64()".to_string(),
        Format::U8 => "reader.ReadByte()".to_string(),
        Format::U16 => "reader.ReadUInt16()".to_string(),
        Format::U32 => "reader.ReadUInt32()".to_string(),
        Format::U64 => "reader.ReadUInt64()".to_string(),
        Format::F32 => "reader.ReadSingle()".to_string(),
        Format::F64 => "reader.ReadDouble()".to_string(),
        Format::Str => "reader.ReadString()!".to_string(),
        _ => {
            let ty = to_csharp_type(format);
            if is_value_type(format) {
                format!(
                    "context.GetConverter<{ty}>(null).Read(ref reader, context).GetValueOrDefault()"
                )
            } else {
                format!("context.GetConverter<{ty}>(null).Read(ref reader, context)!")
            }
        }
    }
}

/// Emit a write expression for the given format.
///
/// For scalar primitives we call `writer.Write(value)` directly.  For all
/// other types we delegate to `context.GetConverter<T>(null).Write(…)`.
fn write_expr(format: &Format, value_expr: &str) -> String {
    match format {
        Format::Bool
        | Format::I8
        | Format::I16
        | Format::I32
        | Format::I64
        | Format::U8
        | Format::U16
        | Format::U32
        | Format::U64
        | Format::F32
        | Format::F64
        | Format::Str => format!("writer.Write({value_expr});"),
        _ => {
            let ty = to_csharp_type(format);
            format!("context.GetConverter<{ty}>(null).Write(ref writer, {value_expr}, context);")
        }
    }
}

/// Returns true if the container's enum format contains only unit variants.
fn is_all_unit_enum(format: &ContainerFormat) -> bool {
    matches!(
        format,
        ContainerFormat::Enum(variants, _)
            if variants.values().all(|v| matches!(v.value, VariantFormat::Unit))
    )
}

// ---------------------------------------------------------------------------
// impl EmitterPlugin<CSharp>
// ---------------------------------------------------------------------------

impl EmitterPlugin<CSharp> for MessagePackPlugin {
    fn imports(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec![
            "using Facet.Runtime.MessagePack;".to_string(),
            "using Nerdbank.MessagePack;".to_string(),
            "using PolyType;".to_string(),
        ]
    }

    fn runtime_files(&self) -> Vec<RuntimeFile> {
        vec![
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/Unit.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/core/Unit.cs").to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/DeserializationError.cs".to_string(),
                contents: include_bytes!(
                    "../csharp/installer/runtime/serde/DeserializationError.cs"
                )
                .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/MessagePack/MessagePackSerde.cs".to_string(),
                contents: include_bytes!(
                    "../csharp/installer/runtime/messagepack/MessagePackSerde.cs"
                )
                .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/MessagePack/EnumAsStringConverter.cs".to_string(),
                contents: include_bytes!(
                    "../csharp/installer/runtime/messagepack/EnumAsStringConverter.cs"
                )
                .to_vec(),
            },
        ]
    }

    fn manifest_dependencies(&self) -> Vec<String> {
        vec![
            r#"    <PackageReference Include="Nerdbank.MessagePack" Version="1.1.*" />"#
                .to_string(),
            r#"    <PackageReference Include="PolyType" Version="1.3.*" />"#.to_string(),
        ]
    }

    /// Emits a `FacetMessagePackWitness` partial class annotated with
    /// `[GenerateShapeFor<T>]` for each type in `config.entry_types`.
    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        if config.entry_types.is_empty() {
            return Ok(());
        }
        writeln!(w)?;
        for type_name in &config.entry_types {
            let pascal = type_name.to_upper_camel_case();
            writeln!(w, "[GenerateShapeFor<{pascal}>]")?;
        }
        writeln!(w, "internal partial class FacetMessagePackWitness;")
    }

    /// Emits `MessagePack` converter annotations on enum types.
    ///
    /// - All-unit enum → `[MessagePackConverter(typeof(EnumAsStringConverter<T>))]`
    /// - Non-unit enum → `[MessagePackConverter(typeof(TypeNameConverter))]`
    /// - Everything else → no annotations
    fn type_annotations(&self, ctx: &EmitContext) -> Vec<String> {
        let name = ctx.name().to_upper_camel_case();
        match ctx.container.format {
            ContainerFormat::Enum(_, _) if is_all_unit_enum(ctx.container.format) => {
                vec![format!(
                    "[MessagePackConverter(typeof(EnumAsStringConverter<{name}>))]"
                )]
            }
            ContainerFormat::Enum(_, _) => {
                vec![format!("[MessagePackConverter(typeof({name}Converter))]")]
            }
            _ => vec![],
        }
    }

    /// Returns `false` for unit-only enums (plain C# `enum` types can't have
    /// instance methods), `true` for everything else.
    fn has_type_body(&self, ctx: &EmitContext) -> bool {
        !is_all_unit_enum(ctx.container.format)
    }

    /// Emits the `TypeNameConverter` nested class (for non-unit enums) plus
    /// `MessagePackSerialize`/`MessagePackDeserialize` convenience methods.
    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        let name = ctx.name().to_upper_camel_case();
        if let ContainerFormat::Enum(variants, _) = ctx.container.format
            && !is_all_unit_enum(ctx.container.format)
        {
            write_enum_converter(w, &name, variants)?;
            writeln!(w)?;
        }
        write_msgpack_helpers(w, &name)
    }
}

// ---------------------------------------------------------------------------
// Helper writers
// ---------------------------------------------------------------------------

/// Writes `MessagePackSerialize` / `MessagePackDeserialize` expression-body methods.
fn write_msgpack_helpers(w: &mut dyn IndentWrite, type_name: &str) -> io::Result<()> {
    writeln!(w, "public byte[] MessagePackSerialize()")?;
    writeln!(
        w,
        "    => MessagePackSerde.Serialize<{type_name}, FacetMessagePackWitness>(this);"
    )?;
    writeln!(w)?;
    writeln!(
        w,
        "public static {type_name} MessagePackDeserialize(byte[] input)"
    )?;
    writeln!(
        w,
        "    => MessagePackSerde.Deserialize<{type_name}, FacetMessagePackWitness>(input);"
    )
}

/// Generates the `TypeNameConverter` nested class that encodes the
/// rmp-serde externally-tagged wire format for non-unit enums.
///
/// Wire format:
/// - Unit variant    → bare string `"VariantName"`
/// - `NewType` variant → 1-element map `{"VariantName": value}`
/// - Tuple variant   → 1-element map `{"VariantName": [v0, v1, …]}`
/// - Struct variant  → 1-element map `{"VariantName": {"field": v, …}}`
#[allow(clippy::too_many_lines)]
fn write_enum_converter(
    w: &mut dyn IndentWrite,
    enum_name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> io::Result<()> {
    let unit_variants: Vec<&Named<VariantFormat>> = variants
        .values()
        .filter(|v| matches!(v.value, VariantFormat::Unit))
        .collect();
    let non_unit_variants: Vec<&Named<VariantFormat>> = variants
        .values()
        .filter(|v| !matches!(v.value, VariantFormat::Unit))
        .collect();

    writeln!(
        w,
        "internal sealed class {enum_name}Converter : MessagePackConverter<{enum_name}>"
    )?;
    with_block(w, Newlines::BOTH, |w| {
        // ----------------------------------------------------------------
        // Read method
        // ----------------------------------------------------------------
        writeln!(
            w,
            "public override {enum_name}? Read(ref MessagePackReader reader, SerializationContext context)"
        )?;
        with_block(w, Newlines::BOTH, |w| {
            // String branch: unit variants serialised as bare strings
            if !unit_variants.is_empty() {
                writeln!(
                    w,
                    "if (reader.NextMessagePackType == MessagePackType.String)"
                )?;
                with_block(w, Newlines::BOTH, |w| {
                    writeln!(w, "var tag = reader.ReadString()!;")?;
                    writeln!(w, "return tag switch")?;
                    writeln!(w, "{{")?;
                    w.indent();
                    for v in &unit_variants {
                        let vname = v.name.to_upper_camel_case();
                        writeln!(w, r#""{}" => new {vname}(),"#, v.name)?;
                    }
                    writeln!(
                        w,
                        r#"_ => throw new MessagePackSerializationException($"Unknown unit variant for {enum_name}: {{tag}}"),"#
                    )?;
                    w.unindent();
                    writeln!(w, "}};")
                })?;
            }

            // Map branch: non-unit variants serialised as 1-element maps
            if !non_unit_variants.is_empty() {
                writeln!(w, "if (reader.NextMessagePackType == MessagePackType.Map)")?;
                with_block(w, Newlines::BOTH, |w| {
                    writeln!(w, "var count = reader.ReadMapHeader();")?;
                    writeln!(
                        w,
                        r#"if (count != 1) throw new MessagePackSerializationException($"Expected 1-element map for {enum_name}, got {{count}} entries");"#
                    )?;
                    writeln!(w, "var tag = reader.ReadString()!;")?;
                    writeln!(w, "return tag switch")?;
                    writeln!(w, "{{")?;
                    w.indent();
                    for v in &non_unit_variants {
                        let vname = v.name.to_upper_camel_case();
                        match &v.value {
                            VariantFormat::NewType(inner) => {
                                let read = read_expr(inner);
                                writeln!(w, r#""{}" => new {vname}({read}),"#, v.name)?;
                            }
                            VariantFormat::Tuple(_) | VariantFormat::Struct(_) => {
                                writeln!(
                                    w,
                                    r#""{}" => Read{vname}(ref reader, context),"#,
                                    v.name
                                )?;
                            }
                            VariantFormat::Unit | VariantFormat::Variable(_) => {
                                unreachable!()
                            }
                        }
                    }
                    writeln!(
                        w,
                        r#"_ => throw new MessagePackSerializationException($"Unknown variant for {enum_name}: {{tag}}"),"#
                    )?;
                    w.unindent();
                    writeln!(w, "}};")
                })?;
            }

            writeln!(
                w,
                r#"throw new MessagePackSerializationException($"Unexpected MessagePack type for {enum_name}");"#
            )
        })?;

        // ----------------------------------------------------------------
        // Private read helpers for Tuple / Struct variants
        // ----------------------------------------------------------------
        for v in variants.values() {
            let vname = v.name.to_upper_camel_case();
            match &v.value {
                VariantFormat::Tuple(formats) => {
                    writeln!(w)?;
                    writeln!(
                        w,
                        "private static {vname} Read{vname}(ref MessagePackReader reader, SerializationContext context)"
                    )?;
                    with_block(w, Newlines::BOTH, |w| {
                        writeln!(w, "var len = reader.ReadArrayHeader();")?;
                        writeln!(w, "_ = len;")?;
                        for (i, fmt) in formats.iter().enumerate() {
                            let expr = read_expr(fmt);
                            writeln!(w, "var field{i} = {expr};")?;
                        }
                        let args = (0..formats.len())
                            .map(|i| format!("field{i}"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        writeln!(w, "return new {vname}({args});")
                    })?;
                }
                VariantFormat::Struct(fields) => {
                    writeln!(w)?;
                    writeln!(
                        w,
                        "private static {vname} Read{vname}(ref MessagePackReader reader, SerializationContext context)"
                    )?;
                    with_block(w, Newlines::BOTH, |w| {
                        // Declare locals for each field
                        for field in fields {
                            let ty = to_csharp_type(&field.value);
                            let fname = field.name.to_lower_camel_case();
                            if is_value_type(&field.value) {
                                writeln!(w, "{ty} {fname} = default;")?;
                            } else {
                                writeln!(w, "{ty}? {fname} = null;")?;
                            }
                        }
                        writeln!(w, "var mapLen = reader.ReadMapHeader();")?;
                        writeln!(w, "for (var i = 0; i < mapLen; i++)")?;
                        with_block(w, Newlines::BOTH, |w| {
                            writeln!(w, "var key = reader.ReadString()!;")?;
                            writeln!(w, "switch (key)")?;
                            with_block(w, Newlines::BOTH, |w| {
                                for field in fields {
                                    let fname = field.name.to_lower_camel_case();
                                    let expr = read_expr(&field.value);
                                    writeln!(w, r#"case "{fname}": {fname} = {expr}; break;"#)?;
                                }
                                writeln!(w, "default: reader.Skip(context); break;")
                            })
                        })?;
                        let args = fields
                            .iter()
                            .map(|f| {
                                let fname = f.name.to_lower_camel_case();
                                if is_value_type(&f.value) {
                                    fname
                                } else {
                                    format!("{fname}!")
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        writeln!(w, "return new {vname}({args});")
                    })?;
                }
                _ => {} // Unit and NewType variants handled inline in the switch
            }
        }

        // ----------------------------------------------------------------
        // Write method
        // ----------------------------------------------------------------
        writeln!(w)?;
        writeln!(
            w,
            "public override void Write(ref MessagePackWriter writer, in {enum_name}? value, SerializationContext context)"
        )?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "switch (value)")?;
            with_block(w, Newlines::BOTH, |w| {
                writeln!(w, "case null: writer.WriteNil(); break;")?;
                for v in variants.values() {
                    let vname = v.name.to_upper_camel_case();
                    match &v.value {
                        VariantFormat::Unit => {
                            writeln!(w, r#"case {vname}: writer.Write("{}"); break;"#, v.name)?;
                        }
                        VariantFormat::NewType(inner) => {
                            let expr = write_expr(inner, "v.Value");
                            writeln!(w, "case {vname} v:")?;
                            writeln!(w, "    writer.WriteMapHeader(1);")?;
                            writeln!(w, r#"    writer.Write("{}");"#, v.name)?;
                            writeln!(w, "    {expr}")?;
                            writeln!(w, "    break;")?;
                        }
                        VariantFormat::Tuple(formats) => {
                            writeln!(w, "case {vname} v:")?;
                            writeln!(w, "    writer.WriteMapHeader(1);")?;
                            writeln!(w, r#"    writer.Write("{}");"#, v.name)?;
                            writeln!(w, "    writer.WriteArrayHeader({});", formats.len())?;
                            for (i, fmt) in formats.iter().enumerate() {
                                let expr = write_expr(fmt, &format!("v.Field{i}"));
                                writeln!(w, "    {expr}")?;
                            }
                            writeln!(w, "    break;")?;
                        }
                        VariantFormat::Struct(fields) => {
                            writeln!(w, "case {vname} v:")?;
                            writeln!(w, "    writer.WriteMapHeader(1);")?;
                            writeln!(w, r#"    writer.Write("{}");"#, v.name)?;
                            writeln!(w, "    writer.WriteMapHeader({});", fields.len())?;
                            for field in fields {
                                // rmp-serde uses the original (snake_case) field name as the map key
                                let key = &field.name;
                                let prop = field.name.to_upper_camel_case();
                                let expr = write_expr(&field.value, &format!("v.{prop}"));
                                writeln!(w, r#"    writer.Write("{key}");"#)?;
                                writeln!(w, "    {expr}")?;
                            }
                            writeln!(w, "    break;")?;
                        }
                        VariantFormat::Variable(_) => {
                            unreachable!("placeholders should not reach the plugin")
                        }
                    }
                }
                Ok(())
            })
        })
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

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
    fn imports_returns_three_usings() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let imports = plugin.imports(&cfg);
        assert_eq!(
            imports.len(),
            3,
            "expected 3 using directives, got: {imports:?}"
        );
        assert!(
            imports
                .iter()
                .any(|i| i.contains("Facet.Runtime.MessagePack")),
            "missing Facet.Runtime.MessagePack: {imports:?}"
        );
        assert!(
            imports.iter().any(|i| i.contains("Nerdbank.MessagePack")),
            "missing Nerdbank.MessagePack: {imports:?}"
        );
        assert!(
            imports.iter().any(|i| i.contains("PolyType")),
            "missing PolyType: {imports:?}"
        );
    }

    // -------------------------------------------------------------------------
    // module_helpers
    // -------------------------------------------------------------------------

    #[test]
    fn module_helpers_emits_witness_class() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;
        let mut cfg = CodeGeneratorConfig::new("test".to_string());
        cfg.entry_types = vec!["HttpHeader".to_string(), "HttpResponse".to_string()];

        let out = render(|w| plugin.module_helpers(w, &cfg));

        assert!(
            out.contains("[GenerateShapeFor<HttpHeader>]"),
            "missing HttpHeader shape: {out}"
        );
        assert!(
            out.contains("[GenerateShapeFor<HttpResponse>]"),
            "missing HttpResponse shape: {out}"
        );
        assert!(
            out.contains("FacetMessagePackWitness"),
            "missing witness class: {out}"
        );
    }

    #[test]
    fn module_helpers_empty_when_no_types() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;
        let cfg = CodeGeneratorConfig::new("test".to_string());
        // entry_types is empty by default
        let out = render(|w| plugin.module_helpers(w, &cfg));
        assert!(out.is_empty(), "expected empty output, got: {out:?}");
    }

    // -------------------------------------------------------------------------
    // type_annotations
    // -------------------------------------------------------------------------

    #[test]
    fn type_annotations_unit_enum() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "Alpha".to_string()));
        variants.insert(1u32, Named::new(&VariantFormat::Unit, "Beta".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        let annotations = plugin.type_annotations(&ctx);
        assert_eq!(annotations.len(), 1, "{annotations:?}");
        assert!(
            annotations[0].contains("EnumAsStringConverter<MyEnum>"),
            "{annotations:?}"
        );
    }

    #[test]
    fn type_annotations_non_unit_enum() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = BTreeMap::new();
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
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        let annotations = plugin.type_annotations(&ctx);
        assert_eq!(annotations.len(), 1, "{annotations:?}");
        assert!(
            annotations[0].contains("ResultConverter"),
            "{annotations:?}"
        );
        assert!(
            annotations[0].contains("MessagePackConverter"),
            "{annotations:?}"
        );
    }

    #[test]
    fn type_annotations_struct_returns_empty() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert!(plugin.type_annotations(&ctx).is_empty());
    }

    // -------------------------------------------------------------------------
    // has_type_body
    // -------------------------------------------------------------------------

    #[test]
    fn has_type_body_true_for_struct() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert!(plugin.has_type_body(&ctx));
    }

    #[test]
    fn has_type_body_false_for_unit_enum() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "A".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert!(!plugin.has_type_body(&ctx));
    }

    #[test]
    fn has_type_body_true_for_non_unit_enum() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = BTreeMap::new();
        variants.insert(
            0u32,
            Named::new(
                &VariantFormat::NewType(Box::new(Format::Str)),
                "Ok".to_string(),
            ),
        );

        let name = QualifiedTypeName::root("Result".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert!(plugin.has_type_body(&ctx));
    }

    // -------------------------------------------------------------------------
    // runtime_files
    // -------------------------------------------------------------------------

    #[test]
    fn runtime_files_includes_msgpack_serde() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;
        let files = plugin.runtime_files();
        assert!(
            files
                .iter()
                .any(|f| f.relative_path.contains("MessagePackSerde.cs")),
            "missing MessagePackSerde.cs: {files:?}",
            files = files.iter().map(|f| &f.relative_path).collect::<Vec<_>>()
        );
    }

    // -------------------------------------------------------------------------
    // manifest_dependencies
    // -------------------------------------------------------------------------

    #[test]
    fn manifest_dependencies_contain_nerdbank() {
        let plugin = &MessagePackPlugin as &dyn EmitterPlugin<CSharp>;
        let deps = plugin.manifest_dependencies();
        assert!(
            deps.iter().any(|d| d.contains("Nerdbank.MessagePack")),
            "missing Nerdbank.MessagePack: {deps:?}"
        );
        assert!(
            deps.iter().any(|d| d.contains("PolyType")),
            "missing PolyType: {deps:?}"
        );
    }
}
