//! C#-specific bincode plugin for the `facet-generate` code generation pipeline.
//!
//! This module implements [`EmitterPlugin<CSharp>`](crate::generation::plugin::EmitterPlugin)
//! for [`BincodePlugin`], injecting bincode serialization and
//! deserialization methods into generated C# types.
//!
//! The plugin reads the set of all-unit-variant enum names from
//! [`EmitContext::config`](crate::generation::plugin::EmitContext)
//! (`unit_variant_enums`) at call time.  C-style enums (all-unit-variant enums) are
//! emitted as plain C# `enum` types and must be serialized via a static
//! `{EnumName}Bincode` helper class rather than instance methods.
//!
//! # Extension points implemented
//!
//! | Method | What it provides |
//! |---|---|
//! | `imports` | `using Facet.Runtime.Bincode;` |
//! | `type_conformances` | `IFacetSerializable`, `IFacetDeserializable<T>` (non-unit enums/structs) |
//! | `has_type_body` | `false` for all-unit enums, `true` for everything else |
//! | `type_body` | `Serialize`/`Deserialize`/`BincodeSerialize`/`BincodeDeserialize` methods |
//! | `after_type` | `{EnumName}Bincode` static helper class for all-unit enums |

use std::collections::{BTreeMap, BTreeSet};
use std::io;

use super::BincodePlugin;

use heck::{ToLowerCamelCase, ToUpperCamelCase};

use crate::generation::{
    CodeGeneratorConfig,
    csharp::CSharp,
    indent::{IndentWrite, Newlines, with_block},
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
};
use crate::reflection::format::{
    ContainerFormat, Format, Named, Namespace, QualifiedTypeName, VariantFormat,
};

impl EmitterPlugin<CSharp> for BincodePlugin {
    /// Returns the core, serde, and bincode C# runtime sources to be written
    /// into the output directory alongside the generated code.
    fn runtime_files(&self) -> Vec<RuntimeFile> {
        vec![
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/Unit.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/core/Unit.cs").to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/ISerializer.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/serde/ISerializer.cs")
                    .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/IDeserializer.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/serde/IDeserializer.cs")
                    .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/DeserializationError.cs".to_string(),
                contents: include_bytes!(
                    "../csharp/installer/runtime/serde/DeserializationError.cs"
                )
                .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/SerializationError.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/serde/SerializationError.cs")
                    .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Bincode/BincodeSerializer.cs".to_string(),
                contents: include_bytes!(
                    "../csharp/installer/runtime/bincode/BincodeSerializer.cs"
                )
                .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Bincode/BincodeDeserializer.cs".to_string(),
                contents: include_bytes!(
                    "../csharp/installer/runtime/bincode/BincodeDeserializer.cs"
                )
                .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Bincode/IFacetSerializable.cs".to_string(),
                contents: include_bytes!(
                    "../csharp/installer/runtime/bincode/IFacetSerializable.cs"
                )
                .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Bincode/IFacetDeserializable.cs".to_string(),
                contents: include_bytes!(
                    "../csharp/installer/runtime/bincode/IFacetDeserializable.cs"
                )
                .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Bincode/FacetHelpers.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/bincode/FacetHelpers.cs")
                    .to_vec(),
            },
        ]
    }

    /// Returns the single `using` directive needed for bincode support.
    fn imports(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec!["using Facet.Runtime.Bincode;".to_string()]
    }

    /// Injects `IFacetSerializable` and `IFacetDeserializable<T>` conformances.
    ///
    /// All-unit enums are plain C# `enum` types that cannot implement interfaces, so
    /// they return an empty list here; their bincode helpers are emitted in
    /// [`after_type`](Self::after_type) instead.
    fn type_conformances(&self, ctx: &EmitContext) -> Vec<String> {
        if is_all_unit_enum(ctx.container.format) {
            vec![]
        } else {
            let name = ctx.name().to_upper_camel_case();
            vec![
                "IFacetSerializable".to_string(),
                format!("IFacetDeserializable<{name}>"),
            ]
        }
    }

    /// Returns `false` for all-unit enums (their helpers live outside the type in
    /// [`after_type`](Self::after_type)); `true` for everything else.
    fn has_type_body(&self, ctx: &EmitContext) -> bool {
        !is_all_unit_enum(ctx.container.format)
    }

    /// Emits bincode methods inside the type body.
    ///
    /// - All-unit enum → nothing (helpers go in `after_type`)
    /// - Non-unit enum → abstract `Serialize`, per-variant helpers, static `Deserialize`
    /// - Everything else → `Serialize`, `Deserialize`, `BincodeSerialize`, `BincodeDeserialize`
    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        if let ContainerFormat::Enum(variants_map, _) = ctx.container.format {
            if is_all_unit_enum(ctx.container.format) {
                return Ok(());
            }
            let variants: Vec<Named<VariantFormat>> = variants_map.values().cloned().collect();
            write_record_bincode_helpers(w, ctx.name(), &variants, &ctx.config.unit_variant_enums)
        } else {
            write_class_bincode_methods(
                w,
                &ctx.name().to_upper_camel_case(),
                &ctx.fields(),
                &ctx.config.unit_variant_enums,
            )
        }
    }

    /// Emits the `{EnumName}Bincode` static helper class after all-unit enum declarations.
    fn after_type(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        if let ContainerFormat::Enum(variants_map, _) = ctx.container.format
            && is_all_unit_enum(ctx.container.format)
        {
            writeln!(w)?;
            return write_enum_bincode_helpers(w, &ctx.name().to_upper_camel_case(), variants_map);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Internal predicate
// ---------------------------------------------------------------------------

/// Returns `true` when every variant of the given `ContainerFormat::Enum` is
/// [`VariantFormat::Unit`] (i.e. this is a C-style enum).
fn is_all_unit_enum(format: &ContainerFormat) -> bool {
    if let ContainerFormat::Enum(variants, _) = format {
        variants
            .values()
            .all(|v| matches!(v.value, VariantFormat::Unit))
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// Main code-generation functions
// ---------------------------------------------------------------------------

/// Writes `Serialize`, `Deserialize`, `BincodeSerialize`, and `BincodeDeserialize`
/// methods into the body of a `class` or `sealed record` type.
fn write_class_bincode_methods(
    w: &mut dyn IndentWrite,
    class_name: &str,
    fields: &[Named<Format>],
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    writeln!(w, "public void Serialize(ISerializer serializer)")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "serializer.IncreaseContainerDepth();")?;
        for field in fields {
            let field_name = field.name.to_upper_camel_case();
            write_serialize_value(w, &field_name, &field.value, c_style_enums)?;
        }
        writeln!(w, "serializer.DecreaseContainerDepth();")?;
        Ok(())
    })?;

    writeln!(w)?;
    writeln!(
        w,
        "public static {class_name} Deserialize(IDeserializer deserializer)"
    )?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "deserializer.IncreaseContainerDepth();")?;
        for field in fields {
            let local_name = field.name.to_lower_camel_case();
            write_deserialize_binding(w, &local_name, &field.value, c_style_enums)?;
        }
        writeln!(w, "deserializer.DecreaseContainerDepth();")?;
        if fields.is_empty() {
            writeln!(w, "return new {class_name}();")?;
        } else {
            write!(w, "return new {class_name} ")?;
            with_block(w, Newlines::OPEN, |w| {
                for field in fields {
                    let prop_name = field.name.to_upper_camel_case();
                    let local_name = field.name.to_lower_camel_case();
                    writeln!(w, "{prop_name} = {local_name},")?;
                }
                Ok(())
            })?;
            writeln!(w, ";")?;
        }
        Ok(())
    })?;

    writeln!(w)?;
    writeln!(w, "public byte[] BincodeSerialize()")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "var serializer = new BincodeSerializer();")?;
        writeln!(w, "Serialize(serializer);")?;
        writeln!(w, "return serializer.GetBytes();")?;
        Ok(())
    })?;

    writeln!(w)?;
    writeln!(
        w,
        "public static {class_name} BincodeDeserialize(byte[] input)"
    )?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "if (input is null)")?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(
                w,
                "throw new DeserializationError(\"Cannot deserialize null array\");"
            )?;
            Ok(())
        })?;
        writeln!(w, "var deserializer = new BincodeDeserializer(input);")?;
        writeln!(w, "var value = Deserialize(deserializer);")?;
        writeln!(w, "if (deserializer.GetBufferOffset() < input.Length)")?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(
                w,
                "throw new DeserializationError(\"Some input bytes were not read\");"
            )?;
            Ok(())
        })?;
        writeln!(w, "return value;")?;
        Ok(())
    })?;

    Ok(())
}

/// Writes the bincode helpers for an `abstract record` type hierarchy:
///
/// - `public abstract void Serialize(ISerializer serializer);`
/// - Per-variant `private static Deserialize{Variant}` methods
/// - Per-variant `public sealed partial record {Variant}` with `Serialize` override
/// - `public static {Base} Deserialize(IDeserializer deserializer)` dispatch
/// - `BincodeSerialize` / `BincodeDeserialize` wrappers
fn write_record_bincode_helpers(
    w: &mut dyn IndentWrite,
    base_name: &str,
    variants: &[Named<VariantFormat>],
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    writeln!(w, "public abstract void Serialize(ISerializer serializer);")?;
    writeln!(w)?;

    for (index, variant) in variants.iter().enumerate() {
        let variant_name = variant.name.to_upper_camel_case();

        writeln!(
            w,
            "private static {base_name} Deserialize{variant_name}(IDeserializer deserializer)"
        )?;
        with_block(w, Newlines::BOTH, |w| {
            deserializer_variant_body(w, variant, c_style_enums)
        })?;
        writeln!(w)?;

        writeln!(w, "public sealed partial record {variant_name}")?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "public override void Serialize(ISerializer serializer)")?;
            with_block(w, Newlines::BOTH, |w| {
                writeln!(w, "serializer.IncreaseContainerDepth();")?;
                writeln!(w, "serializer.SerializeVariantIndex({index});")?;
                serializer_variant_body_write(w, variant, c_style_enums)?;
                writeln!(w, "serializer.DecreaseContainerDepth();")?;
                Ok(())
            })?;
            writeln!(w)?;
            Ok(())
        })?;
    }

    writeln!(
        w,
        "public static {base_name} Deserialize(IDeserializer deserializer)"
    )?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "var index = deserializer.DeserializeVariantIndex();")?;
        writeln!(w, "return index switch")?;
        with_block(w, Newlines::BOTH, |w| {
            for (index, variant) in variants.iter().enumerate() {
                let variant_name = variant.name.to_upper_camel_case();
                writeln!(w, "{index} => Deserialize{variant_name}(deserializer),")?;
            }
            writeln!(
                w,
                "_ => throw new DeserializationError(\"Unknown variant index for {base_name}: \" + index),"
            )?;
            Ok(())
        })?;
        writeln!(w, ";")?;
        Ok(())
    })?;

    writeln!(w)?;
    writeln!(w, "public byte[] BincodeSerialize()")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "var serializer = new BincodeSerializer();")?;
        writeln!(w, "Serialize(serializer);")?;
        writeln!(w, "return serializer.GetBytes();")?;
        Ok(())
    })?;

    writeln!(w)?;
    writeln!(
        w,
        "public static {base_name} BincodeDeserialize(byte[] input)"
    )?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "if (input is null)")?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(
                w,
                "throw new DeserializationError(\"Cannot deserialize null array\");"
            )?;
            Ok(())
        })?;
        writeln!(w, "var deserializer = new BincodeDeserializer(input);")?;
        writeln!(w, "var value = Deserialize(deserializer);")?;
        writeln!(w, "if (deserializer.GetBufferOffset() < input.Length)")?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(
                w,
                "throw new DeserializationError(\"Some input bytes were not read\");"
            )?;
            Ok(())
        })?;
        writeln!(w, "return value;")?;
        Ok(())
    })?;

    Ok(())
}

/// Writes the static `{EnumName}Bincode` helper class for a C-style (all-unit-variant) enum.
///
/// Because plain C# `enum` types cannot implement interfaces, all bincode serialization
/// logic is placed in a separate static class with `Serialize`, `Deserialize`,
/// `BincodeSerialize`, and `BincodeDeserialize` static methods.
fn write_enum_bincode_helpers(
    w: &mut dyn IndentWrite,
    enum_name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> io::Result<()> {
    writeln!(w, "/// <summary>")?;
    writeln!(
        w,
        "/// Bincode serialization helpers for <see cref=\"{enum_name}\"/>."
    )?;
    writeln!(w, "/// </summary>")?;
    write!(w, "public static class {enum_name}Bincode ")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(
            w,
            "public static void Serialize({enum_name} value, ISerializer serializer)"
        )?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "serializer.IncreaseContainerDepth();")?;
            writeln!(w, "serializer.SerializeVariantIndex((uint)value);")?;
            writeln!(w, "serializer.DecreaseContainerDepth();")?;
            Ok(())
        })?;

        writeln!(w)?;
        writeln!(
            w,
            "public static {enum_name} Deserialize(IDeserializer deserializer)"
        )?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "deserializer.IncreaseContainerDepth();")?;
            writeln!(w, "var index = deserializer.DeserializeVariantIndex();")?;
            writeln!(w, "deserializer.DecreaseContainerDepth();")?;
            writeln!(w, "return index switch")?;
            with_block(w, Newlines::BOTH, |w| {
                for (index, variant) in variants.values().enumerate() {
                    writeln!(
                        w,
                        "{} => {}.{},",
                        index,
                        enum_name,
                        variant.name.to_upper_camel_case()
                    )?;
                }
                writeln!(
                    w,
                    "_ => throw new DeserializationError(\"Unknown variant index for {enum_name}: \" + index),"
                )?;
                Ok(())
            })?;
            writeln!(w, ";")?;
            Ok(())
        })?;

        writeln!(w)?;
        writeln!(
            w,
            "public static byte[] BincodeSerialize({enum_name} value)"
        )?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "var serializer = new BincodeSerializer();")?;
            writeln!(w, "Serialize(value, serializer);")?;
            writeln!(w, "return serializer.GetBytes();")?;
            Ok(())
        })?;

        writeln!(w)?;
        writeln!(
            w,
            "public static {enum_name} BincodeDeserialize(byte[] input)"
        )?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "if (input is null)")?;
            with_block(w, Newlines::BOTH, |w| {
                writeln!(
                    w,
                    "throw new DeserializationError(\"Cannot deserialize null array\");"
                )?;
                Ok(())
            })?;
            writeln!(w, "var deserializer = new BincodeDeserializer(input);")?;
            writeln!(w, "var value = Deserialize(deserializer);")?;
            writeln!(w, "if (deserializer.GetBufferOffset() < input.Length)")?;
            with_block(w, Newlines::BOTH, |w| {
                writeln!(
                    w,
                    "throw new DeserializationError(\"Some input bytes were not read\");"
                )?;
                Ok(())
            })?;
            writeln!(w, "return value;")?;
            Ok(())
        })?;

        Ok(())
    })?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Variant body helpers
// ---------------------------------------------------------------------------

/// Dispatches on the variant format to write the serialization body statements inside
/// a variant's `Serialize` override (after `IncreaseContainerDepth` /
/// `SerializeVariantIndex` have already been written).
fn serializer_variant_body_write(
    w: &mut dyn IndentWrite,
    variant: &Named<VariantFormat>,
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    match &variant.value {
        VariantFormat::Unit => Ok(()),
        VariantFormat::NewType(format) => write_serialize_value(w, "Value", format, c_style_enums),
        VariantFormat::Tuple(formats) => {
            for (index, format) in formats.iter().enumerate() {
                write_serialize_value(w, &format!("Field{index}"), format, c_style_enums)?;
            }
            Ok(())
        }
        VariantFormat::Struct(fields) => {
            for field in fields {
                write_serialize_value(
                    w,
                    &field.name.to_upper_camel_case(),
                    &field.value,
                    c_style_enums,
                )?;
            }
            Ok(())
        }
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
    }
}

/// Dispatches on the variant format to write the full body of a
/// `private static Deserialize{Variant}` method (including the `return` statement).
fn deserializer_variant_body(
    w: &mut dyn IndentWrite,
    variant: &Named<VariantFormat>,
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    match &variant.value {
        VariantFormat::Unit => {
            writeln!(w, "return new {}();", variant.name.to_upper_camel_case())
        }
        VariantFormat::NewType(format) => {
            write_deserialize_binding(w, "value", format, c_style_enums)?;
            writeln!(
                w,
                "return new {}(value);",
                variant.name.to_upper_camel_case()
            )
        }
        VariantFormat::Tuple(formats) => {
            for (index, format) in formats.iter().enumerate() {
                write_deserialize_binding(w, &format!("field{index}"), format, c_style_enums)?;
            }
            let args = (0..formats.len())
                .map(|i| format!("field{i}"))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                w,
                "return new {}({});",
                variant.name.to_upper_camel_case(),
                args
            )
        }
        VariantFormat::Struct(fields) => {
            for field in fields {
                write_deserialize_binding(
                    w,
                    &field.name.to_lower_camel_case(),
                    &field.value,
                    c_style_enums,
                )?;
            }
            let args = fields
                .iter()
                .map(|field| field.name.to_lower_camel_case())
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                w,
                "return new {}({});",
                variant.name.to_upper_camel_case(),
                args
            )
        }
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
    }
}

// ---------------------------------------------------------------------------
// Serialize / deserialize expression writers
// ---------------------------------------------------------------------------

/// Writes a bare serialize expression (no semicolons, no trailing newline).
///
/// # Examples
///
/// - `I32` with val=`"x"`, ser=`"serializer"` → `serializer.SerializeI32(x)`
/// - `Seq(I32)` with val=`"items"` → `FacetHelpers.SerializeCollection(items, serializer, (item, s) => s.SerializeI32(item))`
/// - C-style enum field → `ColorBincode.Serialize(color, serializer)`
///
/// `Tuple` is not handled here — callers must expand tuples before calling this function.
fn write_serialize_expr(
    w: &mut dyn IndentWrite,
    val: &str,
    ser: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qtn) if c_style_enums.contains(&qtn.name) => {
            let type_name = format_qualified_type_name(qtn);
            write!(w, "{type_name}Bincode.Serialize({val}, {ser})")
        }
        Format::TypeName(_) => write!(w, "{val}.Serialize({ser})"),
        Format::Unit => write!(w, "{ser}.SerializeUnit({val})"),
        Format::Bool => write!(w, "{ser}.SerializeBool({val})"),
        Format::I8 => write!(w, "{ser}.SerializeI8({val})"),
        Format::I16 => write!(w, "{ser}.SerializeI16({val})"),
        Format::I32 => write!(w, "{ser}.SerializeI32({val})"),
        Format::I64 => write!(w, "{ser}.SerializeI64({val})"),
        Format::I128 => write!(w, "{ser}.SerializeI128({val})"),
        Format::U8 => write!(w, "{ser}.SerializeU8({val})"),
        Format::U16 => write!(w, "{ser}.SerializeU16({val})"),
        Format::U32 => write!(w, "{ser}.SerializeU32({val})"),
        Format::U64 => write!(w, "{ser}.SerializeU64({val})"),
        Format::U128 => write!(w, "{ser}.SerializeU128({val})"),
        Format::F32 => write!(w, "{ser}.SerializeF32({val})"),
        Format::F64 => write!(w, "{ser}.SerializeF64({val})"),
        Format::Char => write!(w, "{ser}.SerializeChar({val})"),
        Format::Str => write!(w, "{ser}.SerializeStr({val})"),
        Format::Bytes => write!(w, "{ser}.SerializeBytes({val})"),
        Format::Option(inner) => {
            let helper = option_serialize_helper(inner);
            write!(w, "FacetHelpers.{helper}({val}, {ser}, ")?;
            write_serialize_lambda(w, inner, c_style_enums)?;
            write!(w, ")")
        }
        Format::Seq(inner) | Format::Set(inner) => {
            write!(w, "FacetHelpers.SerializeCollection({val}, {ser}, ")?;
            write_serialize_lambda(w, inner, c_style_enums)?;
            write!(w, ")")
        }
        Format::Map { key, value } => {
            write!(w, "FacetHelpers.SerializeMap({val}, {ser}, ")?;
            write_serialize_lambda(w, key, c_style_enums)?;
            write!(w, ", ")?;
            write_serialize_lambda(w, value, c_style_enums)?;
            write!(w, ")")
        }
        Format::Tuple(_) => unreachable!("tuples are handled by callers"),
        Format::TupleArray { content, .. } => {
            write!(w, "FacetHelpers.SerializeArray({val}, {ser}, ")?;
            write_serialize_lambda(w, content, c_style_enums)?;
            write!(w, ")")
        }
    }
}

/// Writes a bare deserialize expression (no semicolons, no trailing newline).
///
/// # Examples
///
/// - `I32` with de=`"deserializer"` → `deserializer.DeserializeI32()`
/// - `Seq(I32)` → `FacetHelpers.DeserializeList(deserializer, d => d.DeserializeI32())`
/// - C-style enum field → `ColorBincode.Deserialize(deserializer)`
///
/// `Tuple` is not handled here — callers must expand tuples before calling this function.
fn write_deserialize_expr(
    w: &mut dyn IndentWrite,
    de: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qtn) if c_style_enums.contains(&qtn.name) => {
            let type_name = format_qualified_type_name(qtn);
            write!(w, "{type_name}Bincode.Deserialize({de})")
        }
        Format::TypeName(type_name) => write!(
            w,
            "{}.Deserialize({de})",
            csharp_type(&Format::TypeName(type_name.clone()))
        ),
        Format::Unit => write!(w, "{de}.DeserializeUnit()"),
        Format::Bool => write!(w, "{de}.DeserializeBool()"),
        Format::I8 => write!(w, "{de}.DeserializeI8()"),
        Format::I16 => write!(w, "{de}.DeserializeI16()"),
        Format::I32 => write!(w, "{de}.DeserializeI32()"),
        Format::I64 => write!(w, "{de}.DeserializeI64()"),
        Format::I128 => write!(w, "{de}.DeserializeI128()"),
        Format::U8 => write!(w, "{de}.DeserializeU8()"),
        Format::U16 => write!(w, "{de}.DeserializeU16()"),
        Format::U32 => write!(w, "{de}.DeserializeU32()"),
        Format::U64 => write!(w, "{de}.DeserializeU64()"),
        Format::U128 => write!(w, "{de}.DeserializeU128()"),
        Format::F32 => write!(w, "{de}.DeserializeF32()"),
        Format::F64 => write!(w, "{de}.DeserializeF64()"),
        Format::Char => write!(w, "{de}.DeserializeChar()"),
        Format::Str => write!(w, "{de}.DeserializeStr()"),
        Format::Bytes => write!(w, "{de}.DeserializeBytes()"),
        Format::Option(inner) => {
            let helper = option_deserialize_helper(inner);
            write!(w, "FacetHelpers.{helper}({de}, ")?;
            write_deserialize_lambda(w, inner, c_style_enums)?;
            write!(w, ")")
        }
        Format::Seq(inner) => {
            write!(w, "FacetHelpers.DeserializeList({de}, ")?;
            write_deserialize_lambda(w, inner, c_style_enums)?;
            write!(w, ")")
        }
        Format::Set(inner) => {
            write!(w, "FacetHelpers.DeserializeSet({de}, ")?;
            write_deserialize_lambda(w, inner, c_style_enums)?;
            write!(w, ")")
        }
        Format::Map { key, value } => {
            write!(w, "FacetHelpers.DeserializeMap({de}, ")?;
            write_deserialize_lambda(w, key, c_style_enums)?;
            write!(w, ", ")?;
            write_deserialize_lambda(w, value, c_style_enums)?;
            write!(w, ")")
        }
        Format::Tuple(_) => unreachable!("tuples are handled by callers"),
        Format::TupleArray { content, size } => {
            write!(w, "FacetHelpers.DeserializeArray({de}, {size}, ")?;
            write_deserialize_lambda(w, content, c_style_enums)?;
            write!(w, ")")
        }
    }
}

/// Writes a top-level serialize statement: `expr;\n`.
///
/// Tuples are expanded inline — each element becomes its own statement, accessing
/// `.Item1`, `.Item2`, etc. on the value expression.
fn write_serialize_value(
    w: &mut dyn IndentWrite,
    value_expr: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    if let Format::Tuple(formats) = format {
        for (index, inner) in formats.iter().enumerate() {
            write_serialize_value(
                w,
                &format!("{value_expr}.Item{}", index + 1),
                inner,
                c_style_enums,
            )?;
        }
        Ok(())
    } else {
        write_serialize_expr(w, value_expr, "serializer", format, c_style_enums)?;
        writeln!(w, ";")
    }
}

/// Writes a top-level deserialize binding: `var {name} = expr;\n`.
///
/// Tuples are expanded — each element is bound to `{name}_item1`, `{name}_item2`, etc.,
/// then combined into a C# value tuple literal.
fn write_deserialize_binding(
    w: &mut dyn IndentWrite,
    var_name: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    if let Format::Tuple(formats) = format {
        for (index, inner) in formats.iter().enumerate() {
            write_deserialize_binding(
                w,
                &format!("{var_name}_item{}", index + 1),
                inner,
                c_style_enums,
            )?;
        }
        if formats.is_empty() {
            writeln!(w, "var {var_name} = new Unit();")
        } else {
            let values = (0..formats.len())
                .map(|i| format!("{var_name}_item{}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(w, "var {var_name} = ({values});")
        }
    } else {
        write!(w, "var {var_name} = ")?;
        write_deserialize_expr(w, "deserializer", format, c_style_enums)?;
        writeln!(w, ";")
    }
}

/// Writes a C# serialize lambda: `(item, s) => expr`.
///
/// For tuples, emits a statement lambda:
/// `(item, s) => { s.SerializeI32(item.Item1); s.SerializeBool(item.Item2); }`
fn write_serialize_lambda(
    w: &mut dyn IndentWrite,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    match format {
        Format::Tuple(formats) if formats.is_empty() => {
            write!(w, "(item, s) => s.SerializeUnit(item)")
        }
        Format::Tuple(formats) => {
            write!(w, "(item, s) => {{ ")?;
            for (index, inner) in formats.iter().enumerate() {
                write_serialize_tuple_stmts(
                    w,
                    &format!("item.Item{}", index + 1),
                    "s",
                    inner,
                    c_style_enums,
                )?;
            }
            write!(w, "}}")
        }
        _ => {
            write!(w, "(item, s) => ")?;
            write_serialize_expr(w, "item", "s", format, c_style_enums)
        }
    }
}

/// Writes a C# deserialize lambda: `d => expr`.
///
/// For tuples, emits a statement lambda:
/// `d => { var item1 = d.DeserializeI32(); var item2 = ...; return (item1, item2); }`
fn write_deserialize_lambda(
    w: &mut dyn IndentWrite,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    match format {
        Format::Tuple(formats) if formats.is_empty() => {
            write!(w, "d => d.DeserializeUnit()")
        }
        Format::Tuple(formats) => {
            write!(w, "d => {{ ")?;
            for (index, inner) in formats.iter().enumerate() {
                write!(w, "var item{} = ", index + 1)?;
                write_deserialize_expr(w, "d", inner, c_style_enums)?;
                write!(w, "; ")?;
            }
            let values = (0..formats.len())
                .map(|i| format!("item{}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");
            write!(w, "return ({values}); }}")
        }
        _ => {
            write!(w, "d => ")?;
            write_deserialize_expr(w, "d", format, c_style_enums)
        }
    }
}

/// Writes inline serialize statements for tuple elements inside a statement lambda,
/// flattening nested tuples into their constituent parts.
///
/// Each non-tuple element becomes `expr; `. Nested tuples are recursively decomposed
/// via `.ItemN` access.
fn write_serialize_tuple_stmts(
    w: &mut dyn IndentWrite,
    val: &str,
    ser: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> io::Result<()> {
    if let Format::Tuple(formats) = format {
        for (index, inner) in formats.iter().enumerate() {
            write_serialize_tuple_stmts(
                w,
                &format!("{val}.Item{}", index + 1),
                ser,
                inner,
                c_style_enums,
            )?;
        }
        Ok(())
    } else {
        write_serialize_expr(w, val, ser, format, c_style_enums)?;
        write!(w, "; ")
    }
}

// ---------------------------------------------------------------------------
// Option helpers
// ---------------------------------------------------------------------------

/// Returns the `FacetHelpers` method name for serializing an `Option<T>`.
///
/// Value types use `SerializeOption`; reference types use `SerializeOptionRef`.
const fn option_serialize_helper(inner: &Format) -> &'static str {
    if is_csharp_value_type(inner) {
        "SerializeOption"
    } else {
        "SerializeOptionRef"
    }
}

/// Returns the `FacetHelpers` method name for deserializing an `Option<T>`.
///
/// Value types use `DeserializeOption`; reference types use `DeserializeOptionRef`.
const fn option_deserialize_helper(inner: &Format) -> &'static str {
    if is_csharp_value_type(inner) {
        "DeserializeOption"
    } else {
        "DeserializeOptionRef"
    }
}

// ---------------------------------------------------------------------------
// Type rendering helpers (duplicated from csharp/emitter/mod.rs)
// ---------------------------------------------------------------------------

/// Maps a [`Format`] to its C# type name.
fn csharp_type(format: &Format) -> String {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qualified_type_name) => format_qualified_type_name(qualified_type_name),
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
        Format::Option(inner) => format!("{}?", csharp_type(inner)),
        Format::Seq(inner) => format!("ObservableCollection<{}>", csharp_type(inner)),
        Format::Set(inner) => format!("HashSet<{}>", csharp_type(inner)),
        Format::Map { key, value } => {
            format!("Dictionary<{}, {}>", csharp_type(key), csharp_type(value))
        }
        Format::Tuple(formats) => {
            if formats.is_empty() {
                return "Unit".to_string();
            }
            let values = formats
                .iter()
                .map(csharp_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({values})")
        }
        Format::TupleArray { content, size: _ } => format!("{}[]", csharp_type(content)),
    }
}

/// Formats a [`QualifiedTypeName`] as a C# dotted name.
fn format_qualified_type_name(qualified_type_name: &QualifiedTypeName) -> String {
    match &qualified_type_name.namespace {
        Namespace::Root => qualified_type_name.name.to_upper_camel_case(),
        Namespace::Named(namespace) => {
            format!(
                "{}.{}",
                namespace_name(namespace),
                qualified_type_name.name.to_upper_camel_case()
            )
        }
    }
}

/// Converts a dotted namespace string into `PascalCase` C# namespace segments.
fn namespace_name(namespace: &str) -> String {
    namespace
        .split('.')
        .map(str::to_upper_camel_case)
        .collect::<Vec<_>>()
        .join(".")
}

/// Returns `true` for C# value types (structs, primitives, tuples) that use
/// `SerializeOption` / `DeserializeOption` rather than the `…Ref` variants.
const fn is_csharp_value_type(format: &Format) -> bool {
    matches!(
        format,
        Format::Unit
            | Format::Bool
            | Format::I8
            | Format::I16
            | Format::I32
            | Format::I64
            | Format::I128
            | Format::U8
            | Format::U16
            | Format::U32
            | Format::U64
            | Format::U128
            | Format::F32
            | Format::F64
            | Format::Char
            | Format::Tuple(_)
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{
        CodeGeneratorConfig, Container,
        bincode::BincodePlugin,
        indent::{IndentConfig, IndentedWriter},
        plugin::EmitContext,
    };
    use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

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
    fn imports_returns_bincode() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<CSharp>;
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let imports: Vec<String> = plugin.imports(&cfg);
        assert_eq!(imports, vec!["using Facet.Runtime.Bincode;"]);
    }

    // -------------------------------------------------------------------------
    // type_conformances
    // -------------------------------------------------------------------------

    #[test]
    fn type_conformances_struct() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<CSharp>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let name = QualifiedTypeName::root("MyStruct".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);

        let conformances = plugin.type_conformances(&ctx);
        assert!(
            conformances.iter().any(|c| c == "IFacetSerializable"),
            "{conformances:?}"
        );
        assert!(
            conformances
                .iter()
                .any(|c| c.contains("IFacetDeserializable")),
            "{conformances:?}"
        );
    }

    #[test]
    fn type_conformances_unit_enum_returns_empty() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<CSharp>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let mut variants = std::collections::BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "A".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);

        assert!(plugin.type_conformances(&ctx).is_empty());
    }

    // -------------------------------------------------------------------------
    // has_type_body
    // -------------------------------------------------------------------------

    #[test]
    fn has_type_body_true_for_struct() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<CSharp>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);

        assert!(plugin.has_type_body(&ctx));
    }

    #[test]
    fn has_type_body_false_for_unit_enum() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<CSharp>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let mut variants = std::collections::BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "A".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);

        assert!(!plugin.has_type_body(&ctx));
    }

    // -------------------------------------------------------------------------
    // type_body — struct
    // -------------------------------------------------------------------------

    #[test]
    fn type_body_unit_struct_emits_serialize_deserialize() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<CSharp>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let name = QualifiedTypeName::root("UnitStruct".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);

        let out = render(|w| plugin.type_body(w, &ctx));
        assert!(
            out.contains("public void Serialize(ISerializer serializer)"),
            "{out}"
        );
        assert!(
            out.contains("public static UnitStruct Deserialize(IDeserializer deserializer)"),
            "{out}"
        );
        assert!(out.contains("return new UnitStruct();"), "{out}");
        assert!(out.contains("public byte[] BincodeSerialize()"), "{out}");
        assert!(
            out.contains("public static UnitStruct BincodeDeserialize(byte[] input)"),
            "{out}"
        );
    }

    // -------------------------------------------------------------------------
    // after_type — unit enum
    // -------------------------------------------------------------------------

    #[test]
    fn after_type_unit_enum_emits_static_helper() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<CSharp>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let mut variants = std::collections::BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "Alpha".to_string()));
        variants.insert(1u32, Named::new(&VariantFormat::Unit, "Beta".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);

        let out = render(|w| plugin.after_type(w, &ctx));
        assert!(out.contains("public static class MyEnumBincode"), "{out}");
        assert!(
            out.contains("public static void Serialize(MyEnum value, ISerializer serializer)"),
            "{out}"
        );
        assert!(
            out.contains("public static MyEnum Deserialize(IDeserializer deserializer)"),
            "{out}"
        );
    }

    #[test]
    fn after_type_struct_emits_nothing() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<CSharp>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);

        let out = render(|w| plugin.after_type(w, &ctx));
        assert!(out.is_empty(), "expected empty output, got:\n{out}");
    }
}
