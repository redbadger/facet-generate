use std::{
    collections::BTreeMap,
    io::{Result, Write},
};

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

        let encoding = self.config().encoding;
        let for_json = encoding.is_json();
        let has_bigint = self.config().has_bigint;

        let mut imports = match encoding {
            Encoding::Json => vec![
                "import kotlinx.serialization.Serializable",
                "import kotlinx.serialization.SerialName",
            ],
            Encoding::Bincode => vec![
                "import com.novi.bincode.BincodeDeserializer",
                "import com.novi.bincode.BincodeSerializer",
                "import com.novi.serde.DeserializationError",
                "import com.novi.serde.Deserializer",
                "import com.novi.serde.Serializer",
            ],
            _ => vec![],
        };
        if has_bigint {
            if let Encoding::Json = encoding {
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
        }
        imports.sort_unstable();
        if !imports.is_empty() {
            writeln!(w, "{}", imports.join("\n"))?;
            writeln!(w)?;
        }

        if has_bigint {
            if for_json {
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
                data_object(w, &name.name, None, doc, *encoding, None)?;
            }
            ContainerFormat::NewTypeStruct(format, doc) => {
                data_class(
                    w,
                    &name.name,
                    None,
                    &[Named {
                        name: "value".to_string(),
                        doc: Doc::new(),
                        value: Format::clone(format),
                    }],
                    doc,
                    *encoding,
                    None,
                )?;
            }
            ContainerFormat::TupleStruct(formats, doc) => {
                data_class(w, &name.name, None, &named(formats), doc, *encoding, None)?;
            }
            ContainerFormat::Struct(fields, doc) => {
                if fields.is_empty() {
                    data_object(w, &name.name, None, doc, *encoding, None)?;
                } else {
                    data_class(w, &name.name, None, fields, doc, *encoding, None)?;
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

        writeln!(w, ",")
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
    SealedInterface(String, usize),
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
            (VariantFormat::Unit, VariantContext::SealedInterface(interface_name, index)) => {
                data_object(
                    w,
                    name,
                    Some(interface_name),
                    &variant.doc,
                    encoding,
                    Some(*index),
                )?;
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
            (
                VariantFormat::NewType(format),
                VariantContext::SealedInterface(interface_name, index),
            ) => {
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
                    Some(*index),
                )?;
            }
            (VariantFormat::NewType(_format), VariantContext::EnumClass) => {
                unreachable!("NewType variants are not supported in enum classes")
            }
            (
                VariantFormat::Tuple(formats),
                VariantContext::SealedInterface(interface_name, index),
            ) => {
                data_class(
                    w,
                    name,
                    Some(interface_name),
                    &named(formats),
                    &variant.doc,
                    encoding,
                    Some(*index),
                )?;
            }
            (VariantFormat::Tuple(_formats), VariantContext::EnumClass) => {
                unreachable!("Tuple variants are not supported in enum classes")
            }
            (
                VariantFormat::Struct(fields),
                VariantContext::SealedInterface(interface_name, index),
            ) => {
                data_class(
                    w,
                    name,
                    Some(interface_name),
                    fields,
                    &variant.doc,
                    encoding,
                    Some(*index),
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
    variant_index: Option<usize>,
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

    if encoding.is_bincode() {
        write!(w, " ")?;
        start_block(w)?;
        if variant_index.is_some() {
            write!(w, "override ")?;
        }
        write!(w, "fun serialize(serializer: Serializer) ")?;
        if let Some(index) = variant_index {
            start_block(w)?;
            push_serializer(w)?;
            writeln!(w, "serializer.serialize_variant_index({index})")?;
            pop_serializer(w)?;
            end_block(w)?;
        } else {
            empty_block(w)?;
        }
        writeln!(w)?;

        if variant_index.is_none() {
            write_bincode_serialize(w)?;
            writeln!(w)?;
        }
        write!(w, "companion object ")?;
        start_block(w)?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        start_block(w)?;
        writeln!(w, "return {name}()")?;
        end_block(w)?;
        if variant_index.is_none() {
            writeln!(w)?;
            write_bincode_deserialize(w, name)?;
        }
        end_block(w)?;
        end_block(w)?;
    } else {
        writeln!(w)?;
    }

    Ok(())
}

fn data_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    interface: Option<&str>,
    fields: &[Named<Format>],
    doc: &Doc,
    encoding: Encoding,
    variant_index: Option<usize>,
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

    if encoding.is_bincode() {
        write!(w, " ")?;
        start_block(w)?;
        if variant_index.is_some() {
            write!(w, "override ")?;
        }
        write!(w, "fun serialize(serializer: Serializer) ")?;
        if fields.is_empty() {
            empty_block(w)?;
        } else {
            start_block(w)?;
            push_serializer(w)?;
            if let Some(index) = variant_index {
                writeln!(w, "serializer.serialize_variant_index({index})")?;
            }
            for field in fields {
                write_serialize(w, &field.name, &field.value)?;
            }
            pop_serializer(w)?;
            end_block(w)?;
        }
        writeln!(w)?;

        if variant_index.is_none() {
            write_bincode_serialize(w)?;
            writeln!(w)?;
        }
        write!(w, "companion object ")?;
        start_block(w)?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        start_block(w)?;
        if fields.is_empty() {
            writeln!(w, "return {name}()")?;
        } else {
            push_deserializer(w)?;
            for field in fields {
                write_deserialize(w, &field.name, &field.value)?;
            }
            pop_deserializer(w)?;
            write!(w, "return {name}(")?;
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write!(w, "{}", field.name)?;
            }
            writeln!(w, ")")?;
        }
        end_block(w)?;
        if variant_index.is_none() {
            writeln!(w)?;
            write_bincode_deserialize(w, name)?;
        }
        end_block(w)?;
        end_block(w)?;
    } else {
        writeln!(w)?;
    }

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

    match encoding {
        Encoding::Json => {
            writedoc!(
                w,
                "
                val serialName: String
                    get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
                "
            )?;
        }
        Encoding::Bincode => {
            write!(w, "fun serialize(serializer: Serializer) ")?;
            start_block(w)?;
            push_serializer(w)?;
            writeln!(w, "serializer.serialize_variant_index(ordinal)")?;
            pop_serializer(w)?;
            end_block(w)?;
            writeln!(w)?;

            write_bincode_serialize(w)?;
            writeln!(w)?;

            write!(w, "companion object ")?;
            start_block(w)?;

            writeln!(w, "@Throws(DeserializationError::class)")?;
            write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
            start_block(w)?;
            push_deserializer(w)?;
            writeln!(w, "val index = deserializer.deserialize_variant_index()")?;
            pop_deserializer(w)?;
            write!(w, "return when (index) ")?;
            start_block(w)?;
            for (i, variant) in variants {
                write!(w, "{i} -> ")?;
                let variant = Named {
                    name: variant.name.to_string(),
                    doc: Doc::new(), // remove comments for this printing
                    value: variant.value.clone(),
                };
                (&variant, &VariantContext::EnumClass, encoding).write(w)?;
                writeln!(w)?;
            }
            writeln!(
                w,
                r#"else -> throw DeserializationError("Unknown variant index for {name}: $index")"#
            )?;
            end_block(w)?;
            end_block(w)?;
            writeln!(w)?;

            write_bincode_deserialize(w, name)?;
            end_block(w)?;
        }
        _ => (),
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
    write!(w, "sealed interface {name} ")?;
    start_block(w)?;

    if encoding.is_bincode() {
        writeln!(w, "fun serialize(serializer: Serializer)")?;
        writeln!(w)?;
        write_bincode_serialize(w)?;
        writeln!(w)?;
    }

    for (index, variant) in variants.iter().enumerate() {
        if index > 0 {
            writeln!(w)?;
        }
        let ctx = VariantContext::SealedInterface(name.to_string(), index);
        (variant, &ctx, encoding).write(w)?;
    }

    if encoding.is_bincode() {
        writeln!(w)?;
        write!(w, "companion object")?;
        start_block(w)?;
        writeln!(w, "@Throws(DeserializationError::class)")?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        start_block(w)?;
        writeln!(w, "val index = deserializer.serialize_variant_index()")?;
        write!(w, "return when (index) ")?;
        start_block(w)?;
        for (i, variant) in variants.iter().enumerate() {
            let name = &variant.name;
            writeln!(w, "{i} -> {name}.deserialize(deserializer)")?;
        }
        writeln!(
            w,
            r#"else -> throw DeserializationError("Unknown variant index for {name}: $index")"#
        )?;
        end_block(w)?;
        writeln!(w)?;
        write_bincode_deserialize(w, name)?;
        end_block(w)?;
    }
    end_block(w)
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

fn write_bincode_serialize<W: Write>(w: &mut W) -> Result<()> {
    writedoc!(
        w,
        r"
        fun bincodeSerialize(): ByteArray {{
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer.get_bytes()
        }}
        "
    )
}

fn write_bincode_deserialize<W: Write>(w: &mut W, name: &str) -> Result<()> {
    writedoc!(
        w,
        r#"
        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): {name} {{
            if (input == null) {{
                throw DeserializationError("Cannot deserialize null array")
            }}
            val deserializer = BincodeDeserializer(input)
            val value = deserialize(deserializer)
            if (deserializer.get_buffer_offset() < input.size) {{
                throw DeserializationError("Some input bytes were not read")
            }}
            return value
        }}
        "#
    )
}

fn write_serialize<W: IndentWrite>(w: &mut W, value: &str, format: &Format) -> Result<()> {
    match format {
        Format::Unit => writeln!(w, "serializer.serialize_unit({value})"),
        Format::Bool => writeln!(w, "serializer.serialize_bool({value})"),
        Format::I8 => writeln!(w, "serializer.serialize_i8({value})"),
        Format::I16 => writeln!(w, "serializer.serialize_i16({value})"),
        Format::I32 => writeln!(w, "serializer.serialize_i32({value})"),
        Format::I64 => writeln!(w, "serializer.serialize_i64({value})"),
        Format::I128 => writeln!(w, "serializer.serialize_i128({value})"),
        Format::U8 => writeln!(w, "serializer.serialize_u8({value})"),
        Format::U16 => writeln!(w, "serializer.serialize_u16({value})"),
        Format::U32 => writeln!(w, "serializer.serialize_u32({value})"),
        Format::U64 => writeln!(w, "serializer.serialize_u64({value})"),
        Format::U128 => writeln!(w, "serializer.serialize_u128({value})"),
        Format::F32 => writeln!(w, "serializer.serialize_f32({value})"),
        Format::F64 => writeln!(w, "serializer.serialize_f64({value})"),
        Format::Char => writeln!(w, "serializer.serialize_char({value})"),
        Format::Str => writeln!(w, "serializer.serialize_str({value})"),
        Format::Bytes => writeln!(w, "serializer.serialize_bytes({value})"),
        Format::TypeName(..)
        | Format::Option(..)
        | Format::Seq(..)
        | Format::Set(..)
        | Format::Map { .. }
        | Format::Tuple(..)
        | Format::TupleArray { .. } => writeln!(w, "{value}.serialize(serializer)"),
        Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
    }
}

fn write_deserialize<W: IndentWrite>(w: &mut W, value: &str, format: &Format) -> Result<()> {
    match format {
        Format::TypeName(qualified_name) => {
            let name = &qualified_name.name;
            writeln!(w, "val {value} = {name}.deserialize(deserializer)")
        }
        Format::Unit => writeln!(w, "val {value} = deserializer.deserialize_unit()"),
        Format::Bool => writeln!(w, "val {value} = deserializer.deserialize_bool()"),
        Format::I8 => writeln!(w, "val {value} = deserializer.deserialize_i8()"),
        Format::I16 => writeln!(w, "val {value} = deserializer.deserialize_i16()"),
        Format::I32 => writeln!(w, "val {value} = deserializer.deserialize_i32()"),
        Format::I64 => writeln!(w, "val {value} = deserializer.deserialize_i64()"),
        Format::I128 => writeln!(w, "val {value} = deserializer.deserialize_i128()"),
        Format::U8 => writeln!(w, "val {value} = deserializer.deserialize_u8()"),
        Format::U16 => writeln!(w, "val {value} = deserializer.deserialize_u16()"),
        Format::U32 => writeln!(w, "val {value} = deserializer.deserialize_u32()"),
        Format::U64 => writeln!(w, "val {value} = deserializer.deserialize_u64()"),
        Format::U128 => writeln!(w, "val {value} = deserializer.deserialize_u128()"),
        Format::F32 => writeln!(w, "val {value} = deserializer.deserialize_f32()"),
        Format::F64 => writeln!(w, "val {value} = deserializer.deserialize_f64()"),
        Format::Char => writeln!(w, "val {value} = deserializer.deserialize_char()"),
        Format::Str => writeln!(w, "val {value} = deserializer.deserialize_str()"),
        Format::Bytes => writeln!(w, "val {value} = deserializer.deserialize_bytes()"),
        Format::Option(_format) => todo!(),
        Format::Seq(_format) => todo!(),
        Format::Set(_format) => todo!(),
        Format::Map { key: _, value: _ } => todo!(),
        Format::Tuple(formats) => {
            let len = formats.len();
            let typename = match len {
                0 => return Ok(()),
                1 => {
                    push_deserializer(w)?;
                    write_deserialize(w, "value", &formats[0])?;
                    pop_deserializer(w)?;
                    return Ok(());
                }
                2 => "Pair".to_string(),
                3 => "Triple".to_string(),
                _ => format!("NTuple{len}"),
            };
            write!(w, "val {value} = {typename}<")?;
            for (i, format) in formats.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                format.write(w)?;
            }
            writeln!(w, ">.deserialize(deserializer)")
        }
        Format::TupleArray {
            content: _,
            size: _,
        } => todo!(),
        Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
    }
}

fn push_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "serializer.increase_container_depth()")
}

fn pop_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "serializer.decrease_container_depth()")
}

fn push_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "deserializer.increase_container_depth()")
}

fn pop_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "deserializer.decrease_container_depth()")
}

fn start_block<W: IndentWrite>(w: &mut W) -> Result<()> {
    writeln!(w, "{{")?;
    w.indent();
    Ok(())
}

fn end_block<W: IndentWrite>(w: &mut W) -> Result<()> {
    w.unindent();
    writeln!(w, "}}")
}

fn empty_block<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "{{}}")
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bincode;
#[cfg(test)]
mod tests_json;
