use std::{collections::BTreeMap, io::Result};

use indoc::writedoc;

use crate::{
    generation::{Emitter, Encoding, indent::IndentWrite, module::Module},
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName, VariantFormat},
};

pub struct Kotlin;

impl Emitter<Kotlin> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        let name = &self.config().module_name;

        writeln!(w, "package {name}")?;
        writeln!(w)?;

        let mut imports = vec![];

        if self.config().encoding.is_json() {
            imports.extend_from_slice(&[
                "import kotlinx.serialization.Serializable",
                "import kotlinx.serialization.SerialName",
            ]);
        }

        if self.config().has_bigint {
            imports.extend_from_slice(&[
                "import java.math.BigInteger",
                "import kotlinx.serialization.KSerializer",
                "import kotlinx.serialization.descriptors.PrimitiveKind",
                "import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor",
                "import kotlinx.serialization.encoding.Decoder",
                "import kotlinx.serialization.encoding.Encoder",
                "import kotlinx.serialization.json.JsonDecoder",
                "import kotlinx.serialization.json.JsonEncoder",
                "import kotlinx.serialization.json.JsonUnquotedLiteral",
                "import kotlinx.serialization.json.jsonPrimitive",
            ]);
        }

        imports.sort_unstable();

        if !imports.is_empty() {
            writeln!(w, "{}", imports.join("\n"))?;
            writeln!(w)?;
        }

        if self.config().has_bigint {
            if self.config().encoding.is_json() {
                emit_bigint_serializer(w)?;
            } else {
                writeln!(w, "typealias BigInteger = java.math.BigInteger")?;
            }
        }

        Ok(())
    }
}

impl Emitter<Kotlin> for (Encoding, (&QualifiedTypeName, &ContainerFormat)) {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        let (encoding, (name, format)) = self;
        match format {
            ContainerFormat::UnitStruct(doc) => {
                data_object(w, &name.name, None, doc, *encoding)?;
            }
            ContainerFormat::NewTypeStruct(format, doc) => {
                type_alias(w, &name.name, format, doc)?;
            }
            ContainerFormat::TupleStruct(formats, doc) => {
                data_class(w, &name.name, None, &named(formats), doc, *encoding)?;
            }
            ContainerFormat::Struct(fields, doc) => {
                if fields.is_empty() {
                    data_object(w, &name.name, None, doc, *encoding)?;
                } else {
                    data_class(w, &name.name, None, fields, doc, *encoding)?;
                }
            }
            ContainerFormat::Enum(variants, doc) => {
                let variant_list: Vec<_> = variants.values().cloned().collect();

                let all_unit_variants = variants
                    .values()
                    .all(|variant| matches!(variant.value, VariantFormat::Unit));

                if all_unit_variants {
                    enum_class(w, &name.name, variants, doc, *encoding)?;
                } else {
                    sealed_interface(w, &name.name, &variant_list, doc, *encoding)?;
                }
            }
        }

        Ok(())
    }
}

impl Emitter<Kotlin> for Named<Format> {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        self.doc.write(w)?;

        let name = &self.name;
        write!(w, "val {name}: ")?;

        self.value.write(w)?;

        // Add = null default only for top-level Option types
        if matches!(self.value, Format::Option(_)) {
            write!(w, " = null")?;
        }

        writeln!(w, ",")?;

        Ok(())
    }
}

impl Emitter<Kotlin> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub enum VariantContext {
    SealedInterface(String),
    EnumClass,
}

impl Emitter<Kotlin> for (&Named<VariantFormat>, &VariantContext, Encoding) {
    #[allow(clippy::too_many_lines)]
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        let (variant, context, encoding) = *self;

        let name = &variant.name;
        match (&variant.value, context) {
            (VariantFormat::Variable(_), _) => {
                unreachable!("placeholders should not get this far")
            }
            (VariantFormat::Unit, VariantContext::SealedInterface(interface_name)) => {
                data_object(w, name, Some(interface_name), &variant.doc, encoding)?;
            }
            (VariantFormat::Unit, VariantContext::EnumClass) => {
                variant.doc.write(w)?;
                let name_upper = variant.name.to_uppercase();
                if encoding.is_json() {
                    let name = &variant.name;
                    write!(w, r#"@SerialName("{name}") {name_upper}"#)?;
                } else {
                    write!(w, "{name_upper}")?;
                }
            }
            (VariantFormat::NewType(format), VariantContext::SealedInterface(interface_name)) => {
                data_class(
                    w,
                    name,
                    Some(interface_name),
                    &[Named {
                        name: "value".to_string(),
                        doc: Doc::new(),
                        value: Format::clone(format),
                    }],
                    &variant.doc,
                    encoding,
                )?;
            }
            (VariantFormat::NewType(_format), VariantContext::EnumClass) => {
                unreachable!("NewType variants are not supported in enum classes")
            }
            (VariantFormat::Tuple(formats), VariantContext::SealedInterface(interface_name)) => {
                data_class(
                    w,
                    name,
                    Some(interface_name),
                    &named(formats),
                    &variant.doc,
                    encoding,
                )?;
            }
            (VariantFormat::Tuple(_formats), VariantContext::EnumClass) => {
                unreachable!("Tuple variants are not supported in enum classes")
            }
            (VariantFormat::Struct(fields), VariantContext::SealedInterface(interface_name)) => {
                data_class(
                    w,
                    name,
                    Some(interface_name),
                    fields,
                    &variant.doc,
                    encoding,
                )?;
            }
            (VariantFormat::Struct(_fields), VariantContext::EnumClass) => {
                unreachable!("Struct variants are not supported in enum classes")
            }
        }

        Ok(())
    }
}

impl Emitter<Kotlin> for Format {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        match &self {
            Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Format::TypeName(qualified_type_name) => {
                write!(w, "{ty}", ty = qualified_type_name.name)
            }
            Format::Unit => write!(w, "Unit"),
            Format::Bool => write!(w, "Boolean"),
            Format::I8 => write!(w, "Byte"),
            Format::I16 => write!(w, "Short"),
            Format::I32 => write!(w, "Int"),
            Format::I64 => write!(w, "Long"),
            Format::U8 => write!(w, "UByte"),
            Format::U16 => write!(w, "UShort"),
            Format::U32 => write!(w, "UInt"),
            Format::U64 => write!(w, "ULong"),
            Format::I128 | Format::U128 => write!(w, "BigInteger"),
            Format::F32 => write!(w, "Float"),
            Format::F64 => write!(w, "Double"),
            Format::Char | Format::Str => write!(w, "String"),
            Format::Bytes => write!(w, "ByteArray"),

            Format::Option(format) => {
                format.write(w)?;
                write!(w, "?")
            }
            Format::Seq(format) => {
                write!(w, "List<")?;
                format.write(w)?;
                write!(w, ">")
            }
            Format::Set(format) => {
                write!(w, "Set<")?;
                format.write(w)?;
                write!(w, ">")
            }
            Format::Map { key, value } => {
                write!(w, "Map<")?;
                key.write(w)?;
                write!(w, ", ")?;
                value.write(w)?;
                write!(w, ">")
            }
            Format::Tuple(formats) => {
                let len = formats.len();
                match len {
                    0 => write!(w, "Unit"),
                    1 => {
                        // A single-element tuple is just the element itself
                        formats[0].write(w)
                    }
                    2 => {
                        write!(w, "Pair<")?;
                        formats[0].write(w)?;
                        write!(w, ", ")?;
                        formats[1].write(w)?;
                        write!(w, ">")
                    }
                    3 => {
                        write!(w, "Triple<")?;
                        formats[0].write(w)?;
                        write!(w, ", ")?;
                        formats[1].write(w)?;
                        write!(w, ", ")?;
                        formats[2].write(w)?;
                        write!(w, ">")
                    }
                    _ => {
                        // For larger tuples, we'll use a data class NTupleN
                        write!(w, "NTuple{len}<")?;
                        for (i, format) in formats.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            format.write(w)?;
                        }
                        write!(w, ">")
                    }
                }
            }
            Format::TupleArray { content, size: _ } => {
                write!(w, "List<")?;
                content.write(w)?;
                write!(w, ">")
            }
        }
    }
}

fn data_object<W: IndentWrite>(
    w: &mut W,
    name: &str,
    interface: Option<&str>,
    doc: &Doc,
    encoding: Encoding,
) -> Result<()> {
    doc.write(w)?;

    if encoding.is_json() {
        writeln!(w, "@Serializable")?;
        writeln!(w, r#"@SerialName("{name}")"#)?;
    }

    write!(w, "data object {name}")?;

    if let Some(interface) = interface {
        write!(w, ": {interface}")?;
    }

    writeln!(w)
}

fn type_alias<W: IndentWrite>(w: &mut W, name: &str, format: &Format, doc: &Doc) -> Result<()> {
    doc.write(w)?;

    write!(w, "typealias {name} = ")?;
    format.write(w)?;
    writeln!(w)
}

fn data_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    interface: Option<&str>,
    fields: &[Named<Format>],
    doc: &Doc,
    encoding: Encoding,
) -> Result<()> {
    doc.write(w)?;

    if encoding.is_json() {
        writeln!(w, "@Serializable")?;
        writeln!(w, r#"@SerialName("{name}")"#)?;
    }

    writeln!(w, "data class {name}(")?;

    w.indent();
    for field in fields {
        field.write(w)?;
    }
    w.unindent();

    write!(w, ")")?;

    if let Some(interface) = interface {
        write!(w, " : {interface}")?;
    }

    writeln!(w)?;

    Ok(())
}

fn enum_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    encoding: Encoding,
) -> Result<()> {
    doc.write(w)?;

    if encoding.is_json() {
        writeln!(w, "@Serializable")?;
        writeln!(w, r#"@SerialName("{name}")"#)?;
    }

    writeln!(w, "enum class {name} {{")?;

    w.indent();

    for (i, variant) in variants {
        if *i > 0 {
            writeln!(w, ",")?;
        }

        (variant, &VariantContext::EnumClass, encoding).write(w)?;
    }
    writeln!(w, ";")?;
    writeln!(w)?;

    if encoding.is_json() {
        writedoc!(
            w,
            "
            val serialName: String
                get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
        "
        )?;
    }

    w.unindent();

    writeln!(w, "}}")
}

fn sealed_interface<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &[Named<VariantFormat>],
    doc: &Doc,
    encoding: Encoding,
) -> Result<()> {
    doc.write(w)?;

    if encoding.is_json() {
        writeln!(w, "@Serializable")?;
        writeln!(w, r#"@SerialName("{name}")"#)?;
    }
    writeln!(w, "sealed interface {name} {{")?;

    w.indent();

    for (index, variant) in variants.iter().enumerate() {
        if index > 0 {
            writeln!(w)?;
        }
        let ctx = VariantContext::SealedInterface(name.to_string());
        (variant, &ctx, encoding).write(w)?;
    }
    w.unindent();

    writeln!(w, "}}")
}

fn emit_bigint_serializer<W: IndentWrite>(w: &mut W) -> Result<()> {
    writedoc!(
        w,
        r#"
        typealias BigInteger = @Serializable(with = BigIntegerSerializer::class) BigInteger

        private object BigIntegerSerializer : KSerializer<BigInteger> {{
            override val descriptor = PrimitiveSerialDescriptor("java.math.BigInteger", PrimitiveKind.STRING)

            override fun deserialize(decoder: Decoder): BigInteger =
                when (decoder) {{
                    is JsonDecoder -> decoder.decodeJsonElement().jsonPrimitive.content.toBigInteger()
                    else -> decoder.decodeString().toBigInteger()
                }}

            override fun serialize(encoder: Encoder, value: BigInteger) =
                when (encoder) {{
                    is JsonEncoder -> encoder.encodeJsonElement(JsonUnquotedLiteral(value.toString()))
                    else -> encoder.encodeString(value.toString())
                }}
        }}
        "#
    )
}

fn named<Format: Clone>(formats: &[Format]) -> Vec<Named<Format>> {
    formats
        .iter()
        .enumerate()
        .map(|(i, f)| Named {
            name: format!("field_{i}"),
            doc: Doc::new(),
            value: f.clone(),
        })
        .collect()
}

#[cfg(test)]
#[path = "emitter_tests.rs"]
mod tests;
