use std::collections::BTreeSet;
use std::io::{Result, Write};

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

#[derive(Debug, Clone, Copy)]
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
    fn write<W: Write>(&self, w: &mut W, lang: CSharp) -> Result<()> {
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
    fn write<W: IndentWrite>(&self, w: &mut W, lang: CSharp) -> Result<()> {
        let Container {
            name: QualifiedTypeName { namespace: _, name },
            format,
        } = self;

        match format {
            ContainerFormat::UnitStruct(doc) => write_sealed_record(w, name, doc, lang),
            ContainerFormat::NewTypeStruct(format, doc) => write_class(
                w,
                name,
                &[Named::new(format, "value".to_string())],
                doc,
                lang,
            ),
            ContainerFormat::TupleStruct(formats, doc) => {
                write_class(w, name, &named(formats), doc, lang)
            }
            ContainerFormat::Struct(fields, doc) => {
                if fields.is_empty() {
                    write_sealed_record(w, name, doc, lang)
                } else {
                    write_class(w, name, fields, doc, lang)
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
                    write_variant_record_hierarchy(w, name, &variant_list, doc, lang)
                }
            }
        }
    }
}

impl Emitter<CSharp> for Named<Format> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: CSharp) -> Result<()> {
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
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: CSharp) -> Result<()> {
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
    lang: CSharp,
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
            write_class_bincode_methods(&mut w, &record_name, &[])?;
        }
    }

    Ok(())
}

fn write_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    fields: &[Named<Format>],
    doc: &Doc,
    lang: CSharp,
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
        write_class_bincode_methods(&mut w, &class_name, fields)?;
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
) -> Result<()> {
    writeln!(w, "public void Serialize(ISerializer serializer)")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "serializer.IncreaseContainerDepth();")?;
        for field in fields {
            let field_name = field.name.to_upper_camel_case();
            write_serialize_value(&mut w, &field_name, &field.value)?;
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
            write_deserialize_binding(&mut w, &local_name, &field.value)?;
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
    lang: CSharp,
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
    lang: CSharp,
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
        write_record_bincode_helpers(&mut w, &base_name, variants)?;
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
            deserializer_variant_body(&mut w, variant)?;
        }
        writeln!(w)?;

        writeln!(w, "public sealed partial record {variant_name}")?;
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "public override void Serialize(ISerializer serializer)")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, "serializer.IncreaseContainerDepth();")?;
            writeln!(w, "serializer.SerializeVariantIndex({index});")?;
            serializer_variant_body_write(&mut w, variant)?;
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
) -> Result<()> {
    match &variant.value {
        VariantFormat::Unit => Ok(()),
        VariantFormat::NewType(format) => write_serialize_value(w, "Value", format),
        VariantFormat::Tuple(formats) => {
            for (index, format) in formats.iter().enumerate() {
                write_serialize_value(w, &format!("Field{index}"), format)?;
            }
            Ok(())
        }
        VariantFormat::Struct(fields) => {
            for field in fields {
                write_serialize_value(w, &field.name.to_upper_camel_case(), &field.value)?;
            }
            Ok(())
        }
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
    }
}

fn deserializer_variant_body<W: IndentWrite>(
    w: &mut W,
    variant: &Named<VariantFormat>,
) -> Result<()> {
    match &variant.value {
        VariantFormat::Unit => writeln!(w, "return new {}();", variant.name.to_upper_camel_case()),
        VariantFormat::NewType(format) => {
            write_deserialize_binding(w, "value", format)?;
            writeln!(
                w,
                "return new {}(value);",
                variant.name.to_upper_camel_case()
            )
        }
        VariantFormat::Tuple(formats) => {
            for (index, format) in formats.iter().enumerate() {
                write_deserialize_binding(w, &format!("field{index}"), format)?;
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
                write_deserialize_binding(w, &field.name.to_lower_camel_case(), &field.value)?;
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

fn write_serialize_value<W: IndentWrite>(
    w: &mut W,
    value_expr: &str,
    format: &Format,
) -> Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(_) => writeln!(w, "{value_expr}.Serialize(serializer);"),
        Format::Unit => writeln!(w, "serializer.SerializeUnit({value_expr});"),
        Format::Bool => writeln!(w, "serializer.SerializeBool({value_expr});"),
        Format::I8 => writeln!(w, "serializer.SerializeI8({value_expr});"),
        Format::I16 => writeln!(w, "serializer.SerializeI16({value_expr});"),
        Format::I32 => writeln!(w, "serializer.SerializeI32({value_expr});"),
        Format::I64 => writeln!(w, "serializer.SerializeI64({value_expr});"),
        Format::I128 => writeln!(w, "serializer.SerializeI128({value_expr});"),
        Format::U8 => writeln!(w, "serializer.SerializeU8({value_expr});"),
        Format::U16 => writeln!(w, "serializer.SerializeU16({value_expr});"),
        Format::U32 => writeln!(w, "serializer.SerializeU32({value_expr});"),
        Format::U64 => writeln!(w, "serializer.SerializeU64({value_expr});"),
        Format::U128 => writeln!(w, "serializer.SerializeU128({value_expr});"),
        Format::F32 => writeln!(w, "serializer.SerializeF32({value_expr});"),
        Format::F64 => writeln!(w, "serializer.SerializeF64({value_expr});"),
        Format::Char => writeln!(w, "serializer.SerializeChar({value_expr});"),
        Format::Str => writeln!(w, "serializer.SerializeStr({value_expr});"),
        Format::Bytes => writeln!(w, "serializer.SerializeBytes({value_expr});"),
        Format::Option(inner) => {
            writeln!(w, "if ({value_expr} is not null)")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                writeln!(w, "serializer.SerializeOptionTag(true);")?;
                let unwrapped = if is_csharp_value_type(inner) {
                    format!("{value_expr}.Value")
                } else {
                    value_expr.to_string()
                };
                write_serialize_value(&mut w, &unwrapped, inner)?;
            }
            writeln!(w, "else")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                writeln!(w, "serializer.SerializeOptionTag(false);")?;
            }
            Ok(())
        }
        Format::Seq(inner) | Format::Set(inner) => {
            writeln!(w, "serializer.SerializeLen((ulong){value_expr}.Count);")?;
            writeln!(w, "foreach (var item in {value_expr})")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                write_serialize_value(&mut w, "item", inner)?;
            }
            Ok(())
        }
        Format::Map { key, value } => {
            writeln!(w, "serializer.SerializeLen((ulong){value_expr}.Count);")?;
            writeln!(w, "foreach (var entry in {value_expr})")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                write_serialize_value(&mut w, "entry.Key", key)?;
                write_serialize_value(&mut w, "entry.Value", value)?;
            }
            Ok(())
        }
        Format::Tuple(formats) => {
            for (index, inner) in formats.iter().enumerate() {
                write_serialize_value(w, &format!("{}.Item{}", value_expr, index + 1), inner)?;
            }
            Ok(())
        }
        Format::TupleArray { content, .. } => {
            writeln!(w, "serializer.SerializeLen((ulong){value_expr}.Length);")?;
            writeln!(w, "foreach (var item in {value_expr})")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                write_serialize_value(&mut w, "item", content)?;
            }
            Ok(())
        }
    }
}

fn write_deserialize_binding<W: IndentWrite>(
    w: &mut W,
    var_name: &str,
    format: &Format,
) -> Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(type_name) => {
            writeln!(
                w,
                "var {} = {}.Deserialize(deserializer);",
                var_name,
                csharp_type(&Format::TypeName(type_name.clone()))
            )
        }
        Format::Unit => writeln!(w, "var {var_name} = deserializer.DeserializeUnit();"),
        Format::Bool => writeln!(w, "var {var_name} = deserializer.DeserializeBool();"),
        Format::I8 => writeln!(w, "var {var_name} = deserializer.DeserializeI8();"),
        Format::I16 => writeln!(w, "var {var_name} = deserializer.DeserializeI16();"),
        Format::I32 => writeln!(w, "var {var_name} = deserializer.DeserializeI32();"),
        Format::I64 => writeln!(w, "var {var_name} = deserializer.DeserializeI64();"),
        Format::I128 => writeln!(w, "var {var_name} = deserializer.DeserializeI128();"),
        Format::U8 => writeln!(w, "var {var_name} = deserializer.DeserializeU8();"),
        Format::U16 => writeln!(w, "var {var_name} = deserializer.DeserializeU16();"),
        Format::U32 => writeln!(w, "var {var_name} = deserializer.DeserializeU32();"),
        Format::U64 => writeln!(w, "var {var_name} = deserializer.DeserializeU64();"),
        Format::U128 => writeln!(w, "var {var_name} = deserializer.DeserializeU128();"),
        Format::F32 => writeln!(w, "var {var_name} = deserializer.DeserializeF32();"),
        Format::F64 => writeln!(w, "var {var_name} = deserializer.DeserializeF64();"),
        Format::Char => writeln!(w, "var {var_name} = deserializer.DeserializeChar();"),
        Format::Str => writeln!(w, "var {var_name} = deserializer.DeserializeStr();"),
        Format::Bytes => writeln!(w, "var {var_name} = deserializer.DeserializeBytes();"),
        Format::Option(inner) => write_deserialize_option(w, var_name, inner),
        Format::Seq(inner) => {
            let collection_type = format!("ObservableCollection<{}>", csharp_type(inner));
            write_deserialize_collection(w, var_name, inner, &collection_type)
        }
        Format::Set(inner) => {
            let collection_type = format!("HashSet<{}>", csharp_type(inner));
            write_deserialize_collection(w, var_name, inner, &collection_type)
        }
        Format::Map { key, value } => write_deserialize_map(w, var_name, key, value),
        Format::Tuple(formats) => {
            for (index, inner) in formats.iter().enumerate() {
                write_deserialize_binding(w, &format!("{}_item{}", var_name, index + 1), inner)?;
            }
            if formats.is_empty() {
                writeln!(w, "var {var_name} = new Unit();")
            } else {
                let values = (0..formats.len())
                    .map(|i| format!("{}_item{}", var_name, i + 1))
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(w, "var {var_name} = ({values});")
            }
        }
        Format::TupleArray { content, .. } => {
            writeln!(w, "var {var_name}_len = deserializer.DeserializeLen();")?;
            writeln!(
                w,
                "var {}_list = new List<{}>();",
                var_name,
                csharp_type(content)
            )?;
            writeln!(w, "for (ulong i = 0; i < {var_name}_len; i++)")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                write_deserialize_binding(&mut w, "item", content)?;
                writeln!(w, "{var_name}_list.Add(item);")?;
            }
            writeln!(w, "var {var_name} = {var_name}_list.ToArray();")
        }
    }
}

fn write_deserialize_option<W: IndentWrite>(
    w: &mut W,
    var_name: &str,
    inner: &Format,
) -> Result<()> {
    let inner_type = csharp_type(inner);
    writeln!(w, "{inner_type}? {var_name};")?;
    writeln!(w, "if (deserializer.DeserializeOptionTag())")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        let temp_name = format!("{var_name}_value");
        write_deserialize_binding(&mut w, &temp_name, inner)?;
        writeln!(w, "{var_name} = {temp_name};")?;
    }
    writeln!(w, "else")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "{var_name} = null;")?;
    }
    Ok(())
}

fn write_deserialize_collection<W: IndentWrite>(
    w: &mut W,
    var_name: &str,
    inner: &Format,
    collection_type: &str,
) -> Result<()> {
    let idx = format!("{var_name}_idx");
    let item = format!("{var_name}_item");
    writeln!(w, "var {var_name}_len = deserializer.DeserializeLen();")?;
    writeln!(w, "var {var_name} = new {collection_type}();")?;
    writeln!(w, "for (ulong {idx} = 0; {idx} < {var_name}_len; {idx}++)")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        write_deserialize_binding(&mut w, &item, inner)?;
        writeln!(w, "{var_name}.Add({item});")?;
    }
    Ok(())
}

fn write_deserialize_map<W: IndentWrite>(
    w: &mut W,
    var_name: &str,
    key: &Format,
    value: &Format,
) -> Result<()> {
    let idx = format!("{var_name}_idx");
    let key_var = format!("{var_name}_key");
    let val_var = format!("{var_name}_val");
    writeln!(w, "var {var_name}_len = deserializer.DeserializeLen();")?;
    writeln!(
        w,
        "var {} = new Dictionary<{}, {}>();",
        var_name,
        csharp_type(key),
        csharp_type(value)
    )?;
    writeln!(w, "for (ulong {idx} = 0; {idx} < {var_name}_len; {idx}++)")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        write_deserialize_binding(&mut w, &key_var, key)?;
        write_deserialize_binding(&mut w, &val_var, value)?;
        writeln!(w, "{var_name}.Add({key_var}, {val_var});")?;
    }
    Ok(())
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
