//! AST-to-C# source rendering.
//!
//! This module implements [`Emitter<CSharp>`](super::super::Emitter) for each
//! node type in the format AST, turning abstract type descriptions into
//! idiomatic C# code.
//!
//! # Emitter implementations
//!
//! | AST node | C# output |
//! |---|---|
//! | [`Module`] | `using` directives, file-scoped `namespace` declaration |
//! | [`Container`] | `sealed record`, `partial class : ObservableObject`, `public enum`, or `abstract record` + `sealed record` variant hierarchy |
//! | [`Named<Format>`](Named) | `[ObservableProperty]` private field (+ `[JsonPropertyName]` for JSON) |
//! | [`Format`] | Inline type expression (`int`, `string`, `ObservableCollection<T>`, …) |
//! | [`Doc`] | `///` XML doc comments |
//!
//! # C# type mapping
//!
//! The [`Format`] emitter maps Rust/reflection types to C# equivalents:
//! `I32` → `int`, `I64` → `long`, `U8` → `byte`, `Str` → `string`,
//! `Bytes` → `byte[]`, `Seq(T)` → `ObservableCollection<T>`,
//! `Set(T)` → `HashSet<T>`, `Map(K,V)` → `Dictionary<K, V>`,
//! `Option(T)` → `T?`, tuples → `(T1, T2, …)`, `TupleArray` → `T[]`,
//! `Unit` → `Unit` (custom readonly record struct).
//!
//! # Encoding-dependent output
//!
//! The [`CSharp`] language tag carries the active [`Encoding`]. When encoding
//! is `Json`, types get `System.Text.Json` annotations (`[JsonPropertyName]`,
//! `[JsonPolymorphic]`, `[JsonDerivedType]`, `[JsonConverter]`) plus
//! `JsonSerde` static helper methods. When encoding is `Bincode`, types
//! implement `IFacetSerializable`/`IFacetDeserializable<T>` interfaces with
//! hand-written `Serialize`/`Deserialize` methods and convenience
//! `BincodeSerialize`/`BincodeDeserialize` wrappers. When encoding is `None`,
//! only plain MVVM type declarations are emitted.
//!
//! # Feature helpers via `FacetHelpers.cs`
//!
//! Like Kotlin, Swift, and TypeScript, C# uses reusable helper functions for
//! serializing/deserializing generic container types (collections, maps,
//! options, arrays). Instead of per-module snippet files (as the other
//! languages use), C# places these helpers in a single shared runtime file
//! (`Facet/Runtime/Bincode/FacetHelpers.cs`). This works because C#
//! file-scoped namespaces and `using` directives make a helper class in
//! `Facet.Runtime.Bincode` accessible from any generated namespace without
//! duplication. The generated code calls helpers with lambdas rather than
//! emitting inline loops, e.g.:
//!
//! ```csharp
//! FacetHelpers.SerializeCollection(Items, serializer, (item, s) => s.SerializeStr(item));
//! var items = FacetHelpers.DeserializeList(deserializer, d => d.DeserializeStr());
//! ```

use std::collections::BTreeSet;
use std::io::{Result, Write};

use crate::Registry;

use heck::{ToLowerCamelCase, ToUpperCamelCase};

use crate::{
    generation::{
        CodeGeneratorConfig, Container, Emitter, Encoding,
        indent::{IndentWrite, Newlines},
        module::Module,
    },
    reflection::format::{
        ContainerFormat, Doc, Format, Named, Namespace, QualifiedTypeName, VariantFormat,
    },
};

/// Language tag for C#, carrying the active [`Encoding`].
///
/// Passed to every [`Emitter`](super::super::Emitter) implementation so
/// that encoding-specific code (JSON annotations, Bincode methods) can be
/// conditionally emitted.
#[derive(Debug, Clone)]
pub struct CSharp {
    pub encoding: Encoding,
}

impl CSharp {
    #[must_use]
    pub fn new(encoding: Encoding) -> Self {
        Self { encoding }
    }
}

impl Emitter<CSharp> for Module {
    fn write<W: Write>(&self, w: &mut W, lang: &CSharp) -> Result<()> {
        let CodeGeneratorConfig { module_name, .. } = self.config();
        writeln!(w, "using CommunityToolkit.Mvvm.ComponentModel;")?;
        writeln!(w, "using Facet.Runtime.Serde;")?;
        writeln!(w, "using System.Collections.Generic;")?;
        writeln!(w, "using System.Collections.ObjectModel;")?;
        match lang.encoding {
            Encoding::None => {}
            Encoding::Json => {
                writeln!(w, "using Facet.Runtime.Json;")?;
                writeln!(w, "using System.Text.Json.Serialization;")?;
            }
            Encoding::Bincode => {
                writeln!(w, "using Facet.Runtime.Bincode;")?;
            }
        }
        writeln!(w)?;
        writeln!(w, "namespace {};", namespace_name(module_name))?;
        writeln!(w)
    }
}

impl Emitter<CSharp> for Container<'_> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &CSharp) -> Result<()> {
        let Container {
            name: QualifiedTypeName { namespace: _, name },
            format,
            ..
        } = self;

        let c_style_enums = if lang.encoding == Encoding::Bincode {
            collect_c_style_enums(self.registry)
        } else {
            BTreeSet::new()
        };

        match format {
            ContainerFormat::UnitStruct(doc) => {
                write_sealed_record(w, name, doc, lang, &c_style_enums)
            }
            ContainerFormat::NewTypeStruct(format, doc) => write_class(
                w,
                name,
                &[Named::new(format, "value".to_string())],
                doc,
                lang,
                &c_style_enums,
            ),
            ContainerFormat::TupleStruct(formats, doc) => {
                write_class(w, name, &named(formats), doc, lang, &c_style_enums)
            }
            ContainerFormat::Struct(fields, doc) => {
                if fields.is_empty() {
                    write_sealed_record(w, name, doc, lang, &c_style_enums)
                } else {
                    write_class(w, name, fields, doc, lang, &c_style_enums)
                }
            }
            ContainerFormat::Enum(variants, doc) => {
                let all_unit_variants = variants
                    .values()
                    .all(|variant| matches!(variant.value, VariantFormat::Unit));
                if all_unit_variants {
                    write_enum(w, name, variants, doc, lang)
                } else {
                    let variant_list: Vec<_> = variants.values().cloned().collect();
                    write_variant_record_hierarchy(
                        w,
                        name,
                        &variant_list,
                        doc,
                        lang,
                        &c_style_enums,
                    )
                }
            }
        }
    }
}

/// Scan the registry (if available) and collect the **names** of every enum
/// whose variants are all [`VariantFormat::Unit`] (C-style enums).
///
/// These enums are emitted as plain C# `enum` types rather than class
/// hierarchies, so their bincode serialization must go through a static
/// `{Enum}Bincode` helper class instead of instance methods.
///
/// We collect bare type names (`String`) rather than full
/// [`QualifiedTypeName`]s because [`update_qualified_names`] rewrites the
/// namespace on `Format::TypeName` references without touching registry keys.
/// Within a single module the bare name is unambiguous (types are grouped by
/// namespace via [`module::split`]).
fn collect_c_style_enums(registry: Option<&Registry>) -> BTreeSet<String> {
    registry.map_or_else(BTreeSet::new, |r| {
        r.iter()
            .filter_map(|(name, format)| {
                if let ContainerFormat::Enum(variants, _) = format
                    && variants
                        .values()
                        .all(|v| matches!(v.value, VariantFormat::Unit))
                {
                    return Some(name.name.clone());
                }
                None
            })
            .collect()
    })
}

impl Emitter<CSharp> for Named<Format> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &CSharp) -> Result<()> {
        self.doc.write(w, lang)?;
        if lang.encoding == Encoding::Json {
            writeln!(
                w,
                "[JsonPropertyName(\"{}\")]",
                self.name.to_lower_camel_case()
            )?;
        }
        writeln!(w, "[ObservableProperty]")?;
        writeln!(
            w,
            "private {} _{};",
            csharp_type(&self.value),
            self.name.to_lower_camel_case()
        )
    }
}

impl Emitter<CSharp> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: &CSharp) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }
        Ok(())
    }
}

fn write_sealed_record<W: IndentWrite>(
    w: &mut W,
    name: &str,
    doc: &Doc,
    lang: &CSharp,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
    doc.write(w, lang)?;

    let record_name = name.to_upper_camel_case();
    let has_json = lang.encoding == Encoding::Json;
    let has_bincode = lang.encoding == Encoding::Bincode;

    if !has_json && !has_bincode {
        writeln!(w, "public sealed record {record_name};")?;
        return Ok(());
    }

    let bincode_interfaces = if has_bincode {
        format!(" : IFacetSerializable, IFacetDeserializable<{record_name}>")
    } else {
        String::new()
    };

    write!(w, "public sealed record {record_name}{bincode_interfaces} ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;

        if has_json {
            write_json_helpers(&mut w, &record_name)?;
        }

        if has_bincode {
            if has_json {
                writeln!(w)?;
            }
            write_class_bincode_methods(&mut w, &record_name, &[], c_style_enums)?;
        }
    }

    Ok(())
}

fn write_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    fields: &[Named<Format>],
    doc: &Doc,
    lang: &CSharp,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
    doc.write(w, lang)?;

    let class_name = name.to_upper_camel_case();
    let has_json = lang.encoding == Encoding::Json;
    let has_bincode = lang.encoding == Encoding::Bincode;
    let bincode_interfaces = if has_bincode {
        format!(", IFacetSerializable, IFacetDeserializable<{class_name}>")
    } else {
        String::new()
    };

    write!(
        w,
        "public partial class {class_name} : ObservableObject{bincode_interfaces} "
    )?;

    if fields.is_empty() && !has_json && !has_bincode {
        let _ = w.block(Newlines::CLOSE)?;
        return Ok(());
    }

    let mut w = w.block(Newlines::BOTH)?;
    for field in fields {
        field.write(&mut w, lang)?;
    }

    if has_json {
        if !fields.is_empty() {
            writeln!(w)?;
        }
        write_json_helpers(&mut w, &class_name)?;
    }

    if has_bincode {
        if !fields.is_empty() || has_json {
            writeln!(w)?;
        }
        write_class_bincode_methods(&mut w, &class_name, fields, c_style_enums)?;
    }

    Ok(())
}

fn write_json_helpers<W: IndentWrite>(w: &mut W, type_name: &str) -> Result<()> {
    writeln!(w, "public string JsonSerialize()")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "return JsonSerde.Serialize(this);")?;
    }
    writeln!(w)?;
    writeln!(w, "public static {type_name} JsonDeserialize(string input)")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "return JsonSerde.Deserialize<{type_name}>(input);")?;
    }
    Ok(())
}

fn write_class_bincode_methods<W: IndentWrite>(
    w: &mut W,
    class_name: &str,
    fields: &[Named<Format>],
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
    writeln!(w, "public void Serialize(ISerializer serializer)")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "serializer.IncreaseContainerDepth();")?;
        for field in fields {
            let field_name = field.name.to_upper_camel_case();
            write_serialize_value(&mut w, &field_name, &field.value, c_style_enums)?;
        }
        writeln!(w, "serializer.DecreaseContainerDepth();")?;
    }

    writeln!(w)?;
    writeln!(
        w,
        "public static {class_name} Deserialize(IDeserializer deserializer)"
    )?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "deserializer.IncreaseContainerDepth();")?;
        for field in fields {
            let local_name = field.name.to_lower_camel_case();
            write_deserialize_binding(&mut w, &local_name, &field.value, c_style_enums)?;
        }
        writeln!(w, "deserializer.DecreaseContainerDepth();")?;
        if fields.is_empty() {
            writeln!(w, "return new {class_name}();")?;
        } else {
            write!(w, "return new {class_name} ")?;
            {
                let mut w = w.block(Newlines::OPEN)?;
                for field in fields {
                    let prop_name = field.name.to_upper_camel_case();
                    let local_name = field.name.to_lower_camel_case();
                    writeln!(w, "{prop_name} = {local_name},")?;
                }
            }
            writeln!(w, ";")?;
        }
    }

    writeln!(w)?;
    writeln!(w, "public byte[] BincodeSerialize()")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "var serializer = new BincodeSerializer();")?;
        writeln!(w, "Serialize(serializer);")?;
        writeln!(w, "return serializer.GetBytes();")?;
    }

    writeln!(w)?;
    writeln!(
        w,
        "public static {class_name} BincodeDeserialize(byte[] input)"
    )?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "if (input is null)")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(
                w,
                "throw new DeserializationError(\"Cannot deserialize null array\");"
            )?;
        }
        writeln!(w, "var deserializer = new BincodeDeserializer(input);")?;
        writeln!(w, "var value = Deserialize(deserializer);")?;
        writeln!(w, "if (deserializer.GetBufferOffset() < input.Length)")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(
                w,
                "throw new DeserializationError(\"Some input bytes were not read\");"
            )?;
        }
        writeln!(w, "return value;")?;
    }

    Ok(())
}

fn write_enum<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &std::collections::BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    lang: &CSharp,
) -> Result<()> {
    let enum_name = name.to_upper_camel_case();

    doc.write(w, lang)?;
    if lang.encoding == Encoding::Json {
        writeln!(w, "[JsonConverter(typeof(JsonStringEnumConverter))]")?;
    }
    write!(w, "public enum {enum_name} ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;

        let len = variants.len();
        for (index, variant) in variants.values().enumerate() {
            variant.doc.write(&mut w, lang)?;
            write!(w, "{}", variant.name.to_upper_camel_case())?;
            if index + 1 < len {
                writeln!(w, ",")?;
            } else {
                writeln!(w)?;
            }
        }
    }

    if lang.encoding == Encoding::Bincode {
        writeln!(w)?;
        write_enum_bincode_helpers(w, &enum_name, variants)?;
    }

    Ok(())
}

fn write_enum_bincode_helpers<W: IndentWrite>(
    w: &mut W,
    enum_name: &str,
    variants: &std::collections::BTreeMap<u32, Named<VariantFormat>>,
) -> Result<()> {
    writeln!(w, "/// <summary>")?;
    writeln!(
        w,
        "/// Bincode serialization helpers for <see cref=\"{enum_name}\"/>."
    )?;
    writeln!(w, "/// </summary>")?;
    write!(w, "public static class {enum_name}Bincode ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;

        writeln!(
            w,
            "public static void Serialize({enum_name} value, ISerializer serializer)"
        )?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, "serializer.IncreaseContainerDepth();")?;
            writeln!(w, "serializer.SerializeVariantIndex((uint)value);")?;
            writeln!(w, "serializer.DecreaseContainerDepth();")?;
        }

        writeln!(w)?;
        writeln!(
            w,
            "public static {enum_name} Deserialize(IDeserializer deserializer)"
        )?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, "deserializer.IncreaseContainerDepth();")?;
            writeln!(w, "var index = deserializer.DeserializeVariantIndex();")?;
            writeln!(w, "deserializer.DecreaseContainerDepth();")?;
            writeln!(w, "return index switch")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
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
            }
            writeln!(w, ";")?;
        }

        writeln!(w)?;
        writeln!(
            w,
            "public static byte[] BincodeSerialize({enum_name} value)"
        )?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, "var serializer = new BincodeSerializer();")?;
            writeln!(w, "Serialize(value, serializer);")?;
            writeln!(w, "return serializer.GetBytes();")?;
        }

        writeln!(w)?;
        writeln!(
            w,
            "public static {enum_name} BincodeDeserialize(byte[] input)"
        )?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, "if (input is null)")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                writeln!(
                    w,
                    "throw new DeserializationError(\"Cannot deserialize null array\");"
                )?;
            }
            writeln!(w, "var deserializer = new BincodeDeserializer(input);")?;
            writeln!(w, "var value = Deserialize(deserializer);")?;
            writeln!(w, "if (deserializer.GetBufferOffset() < input.Length)")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                writeln!(
                    w,
                    "throw new DeserializationError(\"Some input bytes were not read\");"
                )?;
            }
            writeln!(w, "return value;")?;
        }
    }

    Ok(())
}

fn write_variant_record_hierarchy<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &[Named<VariantFormat>],
    doc: &Doc,
    lang: &CSharp,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
    let base_name = name.to_upper_camel_case();
    let base_interfaces = if lang.encoding == Encoding::Bincode {
        format!(" : IFacetSerializable, IFacetDeserializable<{base_name}>")
    } else {
        String::new()
    };

    doc.write(w, lang)?;
    if lang.encoding == Encoding::Json {
        writeln!(
            w,
            "[JsonPolymorphic(TypeDiscriminatorPropertyName = \"type\")]"
        )?;
        for variant in variants {
            let variant_name = variant.name.to_upper_camel_case();
            writeln!(
                w,
                "[JsonDerivedType(typeof({variant_name}), \"{}\")]",
                variant.name
            )?;
        }
    }
    write!(w, "public abstract record {base_name}{base_interfaces} ")?;
    let mut w = w.block(Newlines::BOTH)?;

    // Bincode serialization is emitted as a separate partial record declaration,
    // so the primary declaration must also be partial to satisfy the C# compiler.
    let partial = if lang.encoding == Encoding::Bincode {
        " partial"
    } else {
        ""
    };
    for variant in variants {
        variant.doc.write(&mut w, lang)?;
        let variant_name = variant.name.to_upper_camel_case();
        write!(w, "public sealed{partial} record {variant_name}")?;
        match &variant.value {
            VariantFormat::Unit => {
                writeln!(w, "() : {base_name};")?;
            }
            VariantFormat::NewType(inner) => {
                writeln!(w, "({} Value) : {};", csharp_type(inner), base_name)?;
            }
            VariantFormat::Tuple(values) => {
                write!(w, "(")?;
                for (index, format) in values.iter().enumerate() {
                    if index > 0 {
                        write!(w, ", ")?;
                    }
                    write!(w, "{} Field{}", csharp_type(format), index)?;
                }
                writeln!(w, ") : {base_name};")?;
            }
            VariantFormat::Struct(fields) => {
                write!(w, "(")?;
                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(w, ", ")?;
                    }
                    write!(
                        w,
                        "{} {}",
                        csharp_type(&field.value),
                        field.name.to_upper_camel_case()
                    )?;
                }
                writeln!(w, ") : {base_name};")?;
            }
            VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
        }
        writeln!(w)?;
    }

    if lang.encoding == Encoding::Json {
        write_record_json_helpers(&mut w, &base_name)?;
    }

    if lang.encoding == Encoding::Bincode {
        if lang.encoding == Encoding::Json {
            writeln!(w)?;
        }
        write_record_bincode_helpers(&mut w, &base_name, variants, c_style_enums)?;
    }

    Ok(())
}

fn write_record_json_helpers<W: IndentWrite>(w: &mut W, base_name: &str) -> Result<()> {
    writeln!(w, "public string JsonSerialize()")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "return JsonSerde.Serialize(this);")?;
    }
    writeln!(w)?;
    writeln!(w, "public static {base_name} JsonDeserialize(string input)")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "return JsonSerde.Deserialize<{base_name}>(input);")?;
    }
    Ok(())
}

fn write_record_bincode_helpers<W: IndentWrite>(
    w: &mut W,
    base_name: &str,
    variants: &[Named<VariantFormat>],
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
    writeln!(w, "public abstract void Serialize(ISerializer serializer);")?;
    writeln!(w)?;

    for (index, variant) in variants.iter().enumerate() {
        let variant_name = variant.name.to_upper_camel_case();
        writeln!(
            w,
            "private static {base_name} Deserialize{variant_name}(IDeserializer deserializer)"
        )?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            deserializer_variant_body(&mut w, variant, c_style_enums)?;
        }
        writeln!(w)?;

        writeln!(w, "public sealed partial record {variant_name}")?;
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "public override void Serialize(ISerializer serializer)")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, "serializer.IncreaseContainerDepth();")?;
            writeln!(w, "serializer.SerializeVariantIndex({index});")?;
            serializer_variant_body_write(&mut w, variant, c_style_enums)?;
            writeln!(w, "serializer.DecreaseContainerDepth();")?;
        }
        writeln!(w)?;
    }

    writeln!(
        w,
        "public static {base_name} Deserialize(IDeserializer deserializer)"
    )?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "var index = deserializer.DeserializeVariantIndex();")?;
        writeln!(w, "return index switch")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            for (index, variant) in variants.iter().enumerate() {
                let variant_name = variant.name.to_upper_camel_case();
                writeln!(w, "{index} => Deserialize{variant_name}(deserializer),")?;
            }
            writeln!(
                w,
                "_ => throw new DeserializationError(\"Unknown variant index for {base_name}: \" + index),"
            )?;
        }
        writeln!(w, ";")?;
    }

    writeln!(w)?;
    writeln!(w, "public byte[] BincodeSerialize()")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "var serializer = new BincodeSerializer();")?;
        writeln!(w, "Serialize(serializer);")?;
        writeln!(w, "return serializer.GetBytes();")?;
    }

    writeln!(w)?;
    writeln!(
        w,
        "public static {base_name} BincodeDeserialize(byte[] input)"
    )?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "if (input is null)")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(
                w,
                "throw new DeserializationError(\"Cannot deserialize null array\");"
            )?;
        }
        writeln!(w, "var deserializer = new BincodeDeserializer(input);")?;
        writeln!(w, "var value = Deserialize(deserializer);")?;
        writeln!(w, "if (deserializer.GetBufferOffset() < input.Length)")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(
                w,
                "throw new DeserializationError(\"Some input bytes were not read\");"
            )?;
        }
        writeln!(w, "return value;")?;
    }

    Ok(())
}

fn serializer_variant_body_write<W: IndentWrite>(
    w: &mut W,
    variant: &Named<VariantFormat>,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
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

fn deserializer_variant_body<W: IndentWrite>(
    w: &mut W,
    variant: &Named<VariantFormat>,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
    match &variant.value {
        VariantFormat::Unit => writeln!(w, "return new {}();", variant.name.to_upper_camel_case()),
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

/// Writes a bare serialize expression (no semicolons, no newlines), parameterized
/// by variable names for the value and serializer.
///
/// # Examples
///
/// - `I32` with val=`"x"`, ser=`"serializer"` → `serializer.SerializeI32(x)`
/// - `Seq(I32)` with val=`"x"`, ser=`"serializer"` →
///   `FacetHelpers.SerializeCollection(x, serializer, (item, s) => s.SerializeI32(item))`
/// - `Option(F32)` with val=`"x"`, ser=`"s"` →
///   `FacetHelpers.SerializeOption(x, s, (item, s) => s.SerializeF32(item))`
///
/// Tuple is not handled here — callers handle tuples specially since they
/// expand to multiple statements.
fn write_serialize_expr<W: Write>(
    w: &mut W,
    val: &str,
    ser: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
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

/// Writes a bare deserialize expression (no semicolons, no newlines), parameterized
/// by the deserializer variable name.
///
/// # Examples
///
/// - `I32` with de=`"deserializer"` → `deserializer.DeserializeI32()`
/// - `Seq(I32)` with de=`"deserializer"` →
///   `FacetHelpers.DeserializeList(deserializer, d => d.DeserializeI32())`
/// - `TypeName("Foo")` with de=`"d"` → `Foo.Deserialize(d)`
///
/// Tuple is not handled here — callers handle tuples specially since they
/// expand to multiple statements.
fn write_deserialize_expr<W: Write>(
    w: &mut W,
    de: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
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

/// Writes a top-level serialize statement: `write_serialize_expr(...);\n`.
///
/// Tuples are expanded inline — each element becomes its own statement, accessing
/// `.Item1`, `.Item2`, etc. on the value expression.
///
/// # Examples
///
/// - `I32` with expr=`"value"` → `serializer.SerializeI32(value);\n`
/// - `Tuple([I32, Bool])` with expr=`"value"` →
///   `serializer.SerializeI32(value.Item1);\n`
///   `serializer.SerializeBool(value.Item2);\n`
fn write_serialize_value<W: IndentWrite>(
    w: &mut W,
    value_expr: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
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

/// Writes a top-level deserialize binding: `var {name} = write_deserialize_expr(...);\n`.
///
/// Tuples are expanded — each element is bound to `{name}_item1`, `{name}_item2`, etc.,
/// then combined into a C# value tuple.
///
/// # Examples
///
/// - `I32` with `var_name`=`"x"` → `var x = deserializer.DeserializeI32();\n`
/// - `Tuple([I32, Bool])` with `var_name`=`"x"` →
///   `var x_item1 = deserializer.DeserializeI32();\n`
///   `var x_item2 = deserializer.DeserializeBool();\n`
///   `var x = (x_item1, x_item2);\n`
fn write_deserialize_binding<W: IndentWrite>(
    w: &mut W,
    var_name: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
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

/// Writes a C# serialize lambda: `(item, s) => write_serialize_expr(item, s, ...)`.
///
/// For tuples, emits a statement lambda that serializes each element:
/// `(item, s) => { s.SerializeI32(item.Item1); s.SerializeBool(item.Item2); }`
///
/// # Examples
///
/// - `I32` → `(item, s) => s.SerializeI32(item)`
/// - `TypeName("Foo")` → `(item, s) => item.Serialize(s)`
/// - `Seq(I32)` → `(item, s) => FacetHelpers.SerializeCollection(item, s, (item, s) => s.SerializeI32(item))`
fn write_serialize_lambda<W: Write>(
    w: &mut W,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
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

/// Writes a C# deserialize lambda: `d => write_deserialize_expr(d, ...)`.
///
/// For tuples, emits a statement lambda that deserializes each element and returns
/// a value tuple: `d => { var item1 = d.DeserializeI32(); var item2 = ...; return (item1, item2); }`
///
/// # Examples
///
/// - `I32` → `d => d.DeserializeI32()`
/// - `TypeName("Foo")` → `d => Foo.Deserialize(d)`
/// - `Seq(I32)` → `d => FacetHelpers.DeserializeList(d, d => d.DeserializeI32())`
fn write_deserialize_lambda<W: Write>(
    w: &mut W,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
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
/// Each non-tuple element becomes `write_serialize_expr(...); `. Nested tuples
/// are recursively decomposed via `.ItemN` access.
fn write_serialize_tuple_stmts<W: Write>(
    w: &mut W,
    val: &str,
    ser: &str,
    format: &Format,
    c_style_enums: &BTreeSet<String>,
) -> Result<()> {
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

fn option_serialize_helper(inner: &Format) -> &'static str {
    if is_csharp_value_type(inner) {
        "SerializeOption"
    } else {
        "SerializeOptionRef"
    }
}

fn option_deserialize_helper(inner: &Format) -> &'static str {
    if is_csharp_value_type(inner) {
        "DeserializeOption"
    } else {
        "DeserializeOptionRef"
    }
}

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

fn namespace_name(namespace: &str) -> String {
    namespace
        .split('.')
        .map(str::to_upper_camel_case)
        .collect::<Vec<_>>()
        .join(".")
}

fn is_csharp_value_type(format: &Format) -> bool {
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

fn named<Format: Clone>(formats: &[Format]) -> Vec<Named<Format>> {
    formats
        .iter()
        .enumerate()
        .map(|(i, f)| Named::new(f, format!("field{i}")))
        .collect()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bincode;
#[cfg(test)]
mod tests_json;
