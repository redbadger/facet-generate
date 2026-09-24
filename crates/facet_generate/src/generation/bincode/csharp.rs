//! C#-specific bincode plugin for the `facet-generate` code generation pipeline.
//!
//! This module implements [`EmitterPlugin<CSharp>`](crate::generation::plugin::EmitterPlugin)
//! for [`BincodePlugin`], injecting bincode serialization and
//! deserialization methods into generated C# types.
//!
//! The plugin asks [`EmitContext::config`](crate::generation::plugin::EmitContext)
//! whether a type is an all-unit-variant enum
//! ([`is_unit_enum`](crate::generation::CodeGeneratorConfig::is_unit_enum)) at
//! call time.  C-style enums (all-unit-variant enums) are
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

use std::collections::BTreeMap;
use std::io;

use super::BincodePlugin;

use heck::{ToLowerCamelCase, ToUpperCamelCase};

use crate::generation::{
    CodeGeneratorConfig, Feature,
    csharp::{CSharp, escape_identifier, naming},
    indent::{IndentWrite, Newlines, with_block},
    naming::qualify_helper,
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
};
use crate::reflection::format::{
    ContainerFormat, Format, Named, Namespace, QualifiedTypeName, VariantFormat,
};

// ---------------------------------------------------------------------------
// Feature helper snippets
// ---------------------------------------------------------------------------

/// C# UUID serialization helper class.
///
/// Emitted once per module (via `module_helpers`) when `Feature::Uuid` is
/// active.  Uses the .NET 5+ `bigEndian: true` overloads of
/// `Guid.TryWriteBytes` / `new Guid(bytes, bigEndian: true)` to produce
/// RFC 4122 byte order, matching Rust's `uuid::Uuid::as_bytes()` wire
/// format.
const FEATURE_UUID: &str = r#"internal static class UuidSerde
{
    public static void Serialize(Guid value, ISerializer serializer)
    {
        // .NET 5+: bigEndian: true produces RFC 4122 byte order,
        // matching Rust's uuid::Uuid::as_bytes() wire format.
        Span<byte> bytes = stackalloc byte[16];
        value.TryWriteBytes(bytes, bigEndian: true, out _);
        serializer.SerializeBytes(bytes.ToArray());
    }

    public static Guid Deserialize(IDeserializer deserializer)
    {
        var bytes = deserializer.DeserializeBytes();
        if (bytes.Length != 16)
        {
            throw new DeserializationError($"UUID must be 16 bytes, got {bytes.Length}");
        }
        return new Guid(bytes, bigEndian: true);
    }
}
"#;

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

    /// Returns `using` directives needed for bincode support.
    ///
    /// Always includes `Facet.Runtime.Bincode`.  When `Feature::Uuid` is
    /// active, also adds `System` so that `Guid` resolves.
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let mut imports = vec!["using Facet.Runtime.Bincode;".to_string()];
        if config.features.contains(&Feature::Uuid) {
            imports.push("using System;".to_string());
        }
        imports
    }

    /// Emits the `UuidSerde` helper class when `Feature::Uuid` is active.
    ///
    /// C# puts collection-type helpers (`FacetHelpers`) into a shared runtime
    /// file, so only UUID needs a per-module snippet.
    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        if config.features.contains(&Feature::Uuid) {
            write!(
                w,
                "{}",
                qualify_helper(FEATURE_UUID, naming::QUALIFIED, |name| naming::shadows(
                    name, config
                ))
            )?;
            writeln!(w)?;
        }
        Ok(())
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
        if let ContainerFormat::Enum(variants_map, _, _) = ctx.container.format {
            if is_all_unit_enum(ctx.container.format) {
                return Ok(());
            }
            let variants: Vec<Named<VariantFormat>> = variants_map.values().cloned().collect();
            write_record_bincode_helpers(w, ctx.name(), &variants, ctx.config)
        } else {
            write_class_bincode_methods(
                w,
                &ctx.name().to_upper_camel_case(),
                &ctx.fields(),
                ctx.config,
            )
        }
    }

    /// Emits the `{EnumName}Bincode` static helper class after all-unit enum declarations.
    fn after_type(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        if let ContainerFormat::Enum(variants_map, _, _) = ctx.container.format
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
    if let ContainerFormat::Enum(variants, _, _) = format {
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
    cfg: &CodeGeneratorConfig,
) -> io::Result<()> {
    writeln!(w, "public void Serialize(ISerializer serializer)")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "serializer.IncreaseContainerDepth();")?;
        for field in fields {
            let field_name = field.name.to_upper_camel_case();
            write_serialize_statement(w, &field_name, &field.value, cfg)?;
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
            let lower_camel_name = field.name.to_lower_camel_case();
            let local_name = escape_identifier(&lower_camel_name);
            write_deserialize_binding(w, &local_name, &field.value, cfg)?;
        }
        writeln!(w, "deserializer.DecreaseContainerDepth();")?;
        if fields.is_empty() {
            writeln!(w, "return new {class_name}();")?;
        } else {
            write!(w, "return new {class_name} ")?;
            with_block(w, Newlines::OPEN, |w| {
                for field in fields {
                    let prop_name = field.name.to_upper_camel_case();
                    let lower_camel_name = field.name.to_lower_camel_case();
                    let local_name = escape_identifier(&lower_camel_name);
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
    cfg: &CodeGeneratorConfig,
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
            deserializer_variant_body(w, variant, cfg)
        })?;
        writeln!(w)?;

        writeln!(w, "public sealed partial record {variant_name}")?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "public override void Serialize(ISerializer serializer)")?;
            with_block(w, Newlines::BOTH, |w| {
                writeln!(w, "serializer.IncreaseContainerDepth();")?;
                writeln!(w, "serializer.SerializeVariantIndex({index});")?;
                serializer_variant_body_write(w, variant, cfg)?;
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
    cfg: &CodeGeneratorConfig,
) -> io::Result<()> {
    match &variant.value {
        VariantFormat::Unit => Ok(()),
        VariantFormat::NewType(format) => write_serialize_statement(w, "Value", format, cfg),
        VariantFormat::Tuple(formats) => {
            for (index, format) in formats.iter().enumerate() {
                write_serialize_statement(w, &format!("Field{index}"), format, cfg)?;
            }
            Ok(())
        }
        VariantFormat::Struct(fields) => {
            for field in fields {
                write_serialize_statement(w, &field.name.to_upper_camel_case(), &field.value, cfg)?;
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
    cfg: &CodeGeneratorConfig,
) -> io::Result<()> {
    match &variant.value {
        VariantFormat::Unit => {
            writeln!(w, "return new {}();", variant.name.to_upper_camel_case())
        }
        VariantFormat::NewType(format) => {
            write_deserialize_binding(w, "value", format, cfg)?;
            writeln!(
                w,
                "return new {}(value);",
                variant.name.to_upper_camel_case()
            )
        }
        VariantFormat::Tuple(formats) => {
            for (index, format) in formats.iter().enumerate() {
                write_deserialize_binding(w, &format!("field{index}"), format, cfg)?;
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
                let lower_camel_name = field.name.to_lower_camel_case();
                let local_name = escape_identifier(&lower_camel_name);
                write_deserialize_binding(w, &local_name, &field.value, cfg)?;
            }
            let args = fields
                .iter()
                .map(|field| escape_identifier(&field.name.to_lower_camel_case()).into_owned())
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
    cfg: &CodeGeneratorConfig,
) -> io::Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qtn) if cfg.is_unit_enum(qtn) => {
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
        Format::Uuid => write!(w, "UuidSerde.Serialize({val}, {ser})"),
        Format::Option(inner) => {
            let helper = option_serialize_helper(inner, cfg);
            write!(w, "FacetHelpers.{helper}({val}, {ser}, ")?;
            write_serialize_lambda(w, inner, cfg)?;
            write!(w, ")")
        }
        Format::Seq(inner) | Format::Set(inner) => {
            write!(w, "FacetHelpers.SerializeCollection({val}, {ser}, ")?;
            write_serialize_lambda(w, inner, cfg)?;
            write!(w, ")")
        }
        Format::Map { key, value } => {
            write!(w, "FacetHelpers.SerializeMap({val}, {ser}, ")?;
            write_serialize_lambda(w, key, cfg)?;
            write!(w, ", ")?;
            write_serialize_lambda(w, value, cfg)?;
            write!(w, ")")
        }
        Format::Tuple(_) => unreachable!("tuples are handled by callers"),
        Format::TupleArray { content, .. } => {
            write!(w, "FacetHelpers.SerializeArray({val}, {ser}, ")?;
            write_serialize_lambda(w, content, cfg)?;
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
    cfg: &CodeGeneratorConfig,
) -> io::Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qtn) if cfg.is_unit_enum(qtn) => {
            let type_name = format_qualified_type_name(qtn);
            write!(w, "{type_name}Bincode.Deserialize({de})")
        }
        Format::TypeName(type_name) => {
            write!(
                w,
                "{}.Deserialize({de})",
                format_qualified_type_name(type_name)
            )
        }
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
        Format::Uuid => write!(w, "UuidSerde.Deserialize({de})"),
        Format::Option(inner) => {
            let helper = option_deserialize_helper(inner, cfg);
            write!(w, "FacetHelpers.{helper}({de}, ")?;
            write_deserialize_lambda(w, inner, cfg)?;
            write!(w, ")")
        }
        Format::Seq(inner) => {
            write!(w, "FacetHelpers.DeserializeList({de}, ")?;
            write_deserialize_lambda(w, inner, cfg)?;
            write!(w, ")")
        }
        Format::Set(inner) => {
            write!(w, "FacetHelpers.DeserializeSet({de}, ")?;
            write_deserialize_lambda(w, inner, cfg)?;
            write!(w, ")")
        }
        Format::Map { key, value } => {
            write!(w, "FacetHelpers.DeserializeMap({de}, ")?;
            write_deserialize_lambda(w, key, cfg)?;
            write!(w, ", ")?;
            write_deserialize_lambda(w, value, cfg)?;
            write!(w, ")")
        }
        Format::Tuple(_) => unreachable!("tuples are handled by callers"),
        Format::TupleArray { content, size } => {
            write!(w, "FacetHelpers.DeserializeArray({de}, {size}, ")?;
            write_deserialize_lambda(w, content, cfg)?;
            write!(w, ")")
        }
    }
}

/// Write the bincode serialization statement(s) for `value_expr`, a C#
/// expression of the type described by `format`.
///
/// This is the same code the plugin emits for a property, exposed for plugins
/// that need to serialize a value of a type they looked up with
/// [`RegistryBuilder::format_of`](crate::reflection::RegistryBuilder::format_of).
///
/// The type names in `format` must be in the emitter's spelling. A format from
/// [`EmitContext`] already is, but one from `format_of` is in registry
/// spelling, so requalify it with
/// [`csharp::requalify_format`](crate::generation::csharp::requalify_format) first.
///
/// # Preconditions
///
/// A variable named `serializer`, of type `ISerializer`, must be in scope at
/// the point of the emitted code. Container depth is *not* managed here —
/// that is the caller's job, exactly as it is for the generated `Serialize`
/// methods. `config` decides how a named type is serialized: C-style enums
/// (all-unit-variant enums) become plain C# `enum`s and go through their
/// static `{Enum}Bincode` helper, and the emitter recognises them through
/// [`CodeGeneratorConfig::is_unit_enum`], so pass the config for the module
/// being generated.
///
/// # Errors
///
/// Returns an error if writing to `w` fails.
pub fn write_serialize_value(
    w: &mut dyn IndentWrite,
    value_expr: &str,
    format: &Format,
    config: &CodeGeneratorConfig,
) -> io::Result<()> {
    write_serialize_statement(w, value_expr, format, config)
}

/// Writes a top-level serialize statement: `expr;\n`.
///
/// Tuples are expanded inline — each element becomes its own statement, accessing
/// `.Item1`, `.Item2`, etc. on the value expression.
fn write_serialize_statement(
    w: &mut dyn IndentWrite,
    value_expr: &str,
    format: &Format,
    cfg: &CodeGeneratorConfig,
) -> io::Result<()> {
    if let Format::Tuple(formats) = format {
        for (index, inner) in formats.iter().enumerate() {
            write_serialize_statement(w, &format!("{value_expr}.Item{}", index + 1), inner, cfg)?;
        }
        Ok(())
    } else {
        write_serialize_expr(w, value_expr, "serializer", format, cfg)?;
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
    cfg: &CodeGeneratorConfig,
) -> io::Result<()> {
    if let Format::Tuple(formats) = format {
        for (index, inner) in formats.iter().enumerate() {
            write_deserialize_binding(w, &format!("{var_name}_item{}", index + 1), inner, cfg)?;
        }
        if formats.is_empty() {
            writeln!(
                w,
                "var {var_name} = new {}();",
                naming::builtin("Unit", cfg)
            )
        } else {
            let values = (0..formats.len())
                .map(|i| format!("{var_name}_item{}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(w, "var {var_name} = ({values});")
        }
    } else {
        write!(w, "var {var_name} = ")?;
        write_deserialize_expr(w, "deserializer", format, cfg)?;
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
    cfg: &CodeGeneratorConfig,
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
                    cfg,
                )?;
            }
            write!(w, "}}")
        }
        _ => {
            write!(w, "(item, s) => ")?;
            write_serialize_expr(w, "item", "s", format, cfg)
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
    cfg: &CodeGeneratorConfig,
) -> io::Result<()> {
    match format {
        Format::Tuple(formats) if formats.is_empty() => {
            write!(w, "d => d.DeserializeUnit()")
        }
        Format::Tuple(formats) => {
            write!(w, "d => {{ ")?;
            for (index, inner) in formats.iter().enumerate() {
                write!(w, "var item{} = ", index + 1)?;
                write_deserialize_expr(w, "d", inner, cfg)?;
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
            write_deserialize_expr(w, "d", format, cfg)
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
    cfg: &CodeGeneratorConfig,
) -> io::Result<()> {
    if let Format::Tuple(formats) = format {
        for (index, inner) in formats.iter().enumerate() {
            write_serialize_tuple_stmts(w, &format!("{val}.Item{}", index + 1), ser, inner, cfg)?;
        }
        Ok(())
    } else {
        write_serialize_expr(w, val, ser, format, cfg)?;
        write!(w, "; ")
    }
}

// ---------------------------------------------------------------------------
// Option helpers
// ---------------------------------------------------------------------------

/// Returns the `FacetHelpers` method name for serializing an `Option<T>`.
///
/// Value types (including C-style enums) use `SerializeOption`; reference types
/// use `SerializeOptionRef`.
fn option_serialize_helper(inner: &Format, cfg: &CodeGeneratorConfig) -> &'static str {
    if is_csharp_value_type(inner, cfg) {
        "SerializeOption"
    } else {
        "SerializeOptionRef"
    }
}

/// Returns the `FacetHelpers` method name for deserializing an `Option<T>`.
///
/// Value types (including C-style enums) use `DeserializeOption`; reference
/// types use `DeserializeOptionRef`.
fn option_deserialize_helper(inner: &Format, cfg: &CodeGeneratorConfig) -> &'static str {
    if is_csharp_value_type(inner, cfg) {
        "DeserializeOption"
    } else {
        "DeserializeOptionRef"
    }
}

// ---------------------------------------------------------------------------
// Type rendering helpers
// ---------------------------------------------------------------------------

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

/// Returns `true` for C# value types (structs, primitives, tuples, and C-style
/// enums) that use `SerializeOption` / `DeserializeOption` rather than the
/// `…Ref` variants.
fn is_csharp_value_type(format: &Format, cfg: &CodeGeneratorConfig) -> bool {
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
            | Format::Uuid
            | Format::Tuple(_)
    ) || matches!(
        format,
        Format::TypeName(qtn) if cfg.is_unit_enum(qtn)
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
    use crate::reflection::format::{ContainerFormat, Doc, EnumTagging, QualifiedTypeName};

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
        let format = ContainerFormat::Enum(variants, EnumTagging::External, Doc::default());
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
        let format = ContainerFormat::Enum(variants, EnumTagging::External, Doc::default());
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
        let format = ContainerFormat::Enum(variants, EnumTagging::External, Doc::default());
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

    // -------------------------------------------------------------------------
    // write_serialize_value — public helper for plugin authors
    // -------------------------------------------------------------------------

    #[test]
    fn write_serialize_value_emits_a_primitive_call() {
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let out = render(|w| write_serialize_value(w, "output", &Format::Str, &cfg));
        insta::assert_snapshot!(out, @"serializer.SerializeStr(output);");
    }

    #[test]
    fn write_serialize_value_emits_a_method_call_for_a_named_type() {
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let format = Format::TypeName(QualifiedTypeName::root("HttpResult".to_string()));
        let out = render(|w| write_serialize_value(w, "output", &format, &cfg));
        insta::assert_snapshot!(out, @"output.Serialize(serializer);");
    }

    #[test]
    fn write_serialize_value_routes_c_style_enums_through_their_helper() {
        let mut cfg = CodeGeneratorConfig::new("test".to_string());
        cfg.unit_variant_enums
            .insert(QualifiedTypeName::root("Flag".to_string()));
        let format = Format::TypeName(QualifiedTypeName::root("Flag".to_string()));
        let out = render(|w| write_serialize_value(w, "output", &format, &cfg));
        insta::assert_snapshot!(out, @"FlagBincode.Serialize(output, serializer);");
    }

    #[test]
    fn write_serialize_value_qualifies_the_helper_of_an_enum_in_another_namespace() {
        let mut cfg = CodeGeneratorConfig::new("test".to_string());
        let name = QualifiedTypeName::namespaced("Example.kit".to_string(), "Flag".to_string());
        cfg.unit_variant_enums.insert(name.clone());
        let format = Format::TypeName(name);
        let out = render(|w| write_serialize_value(w, "output", &format, &cfg));
        insta::assert_snapshot!(out, @"Example.Kit.FlagBincode.Serialize(output, serializer);");
    }
}
