use std::{
    collections::BTreeMap,
    io::{Result, Write},
    string::ToString,
};

use heck::ToLowerCamelCase;
use indoc::writedoc;

use crate::{
    generation::{
        CodeGeneratorConfig, Emitter, Encoding, Feature, PackageLocation, indent::IndentWrite,
        module::Module,
    },
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName, VariantFormat},
};

const FEATURE_BIGINT: &[u8] = include_bytes!("features/BigInt.kt");
const FEATURE_BUILD_LIST: &[u8] = include_bytes!("features/BuildList.kt");
const FEATURE_LIST_OF_T: &[u8] = include_bytes!("features/ListOfT.kt");
const FEATURE_MAP_OF_T: &[u8] = include_bytes!("features/MapOfT.kt");
const FEATURE_OPTION_OF_T: &[u8] = include_bytes!("features/OptionOfT.kt");
const FEATURE_SET_OF_T: &[u8] = include_bytes!("features/SetOfT.kt");

pub struct Kotlin;

impl Emitter<Kotlin> for Module {
    #[allow(clippy::too_many_lines)]
    fn write<W: Write>(&self, w: &mut W) -> Result<()> {
        let CodeGeneratorConfig {
            module_name,
            encoding,
            features,
            ..
        } = self.config();

        writeln!(w, "package {module_name}")?;
        writeln!(w)?;

        let mut imports = match encoding {
            Encoding::Json => vec![
                "import kotlinx.serialization.Serializable",
                "import kotlinx.serialization.SerialName",
            ],
            Encoding::Bincode => vec![
                "import com.novi.bincode.BincodeDeserializer",
                "import com.novi.bincode.BincodeSerializer",
                "import com.novi.serde.Bytes",
                "import com.novi.serde.DeserializationError",
                "import com.novi.serde.Deserializer",
                "import com.novi.serde.Serializer",
                "import com.novi.serde.Unsigned",
            ],
            _ => vec![],
        };

        let mut features_out = vec![];
        for feature in features {
            match feature {
                Feature::BigInt => {
                    imports.push("import java.math.BigInteger");
                    match encoding {
                        Encoding::Json => {
                            imports.extend_from_slice(&[
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
                            features_out.write_all(FEATURE_BIGINT)?;
                            writeln!(features_out)?;
                        }
                        Encoding::Bincode => {
                            imports.push("import com.novi.serde.Int128");
                        }
                        Encoding::None | Encoding::Bcs => (),
                    }
                }
                Feature::ListOfT => {
                    if encoding == &Encoding::Bincode {
                        features_out.write_all(FEATURE_LIST_OF_T)?;
                        writeln!(features_out)?;
                    }
                }
                Feature::OptionOfT => {
                    if encoding == &Encoding::Bincode {
                        features_out.write_all(FEATURE_OPTION_OF_T)?;
                        writeln!(features_out)?;
                    }
                }
                Feature::SetOfT => {
                    if encoding == &Encoding::Bincode {
                        features_out.write_all(FEATURE_SET_OF_T)?;
                        writeln!(features_out)?;
                    }
                }
                Feature::MapOfT => {
                    if encoding == &Encoding::Bincode {
                        features_out.write_all(FEATURE_MAP_OF_T)?;
                        writeln!(features_out)?;
                    }
                }
                Feature::BuildList => {
                    features_out.write_all(FEATURE_BUILD_LIST)?;
                    writeln!(features_out)?;
                }
            }
        }

        let mut imports = imports
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        for (ns, names) in &self.config().external_definitions {
            if let Some(external_package) = self.config().external_packages.get(ns) {
                if let PackageLocation::Path(ns) = &external_package.location {
                    for name in names {
                        imports.push(format!("import {ns}.{name}"));
                    }
                }
            }
        }

        imports.sort_unstable();
        if !imports.is_empty() {
            for import in imports {
                writeln!(w, "{import}")?;
            }
            writeln!(w)?;
        }

        w.write_all(&features_out)?;

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

        let name = &self.name.to_lower_camel_case();
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

impl Format {
    fn is_native(&self) -> bool {
        match self {
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
            | Format::Str
            | Format::Bytes => true,
            Format::Variable(..)
            | Format::TypeName(..)
            | Format::Option(..)
            | Format::Seq(..)
            | Format::Set(..)
            | Format::Map { .. }
            | Format::Tuple(..)
            | Format::TupleArray { .. } => false,
        }
    }

    fn is_leaf(&self) -> bool {
        self.is_native() || matches!(self, Format::TypeName(..))
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
        w.start_block()?;
        if variant_index.is_some() {
            write!(w, "override ")?;
        }
        write!(w, "fun serialize(serializer: Serializer) ")?;
        if let Some(index) = variant_index {
            w.start_block()?;
            push_serializer(w)?;
            writeln!(w, "serializer.serialize_variant_index({index})")?;
            pop_serializer(w)?;
            w.end_block()?;
        } else {
            w.empty_block()?;
        }
        writeln!(w)?;

        if variant_index.is_none() {
            write_bincode_serialize(w)?;
            writeln!(w)?;
        }
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        w.start_block()?;
        writeln!(w, "return {name}")?;
        w.end_block()?;
        if variant_index.is_none() {
            writeln!(w)?;
            write_bincode_deserialize(w, name)?;
        }
        w.end_block()?;
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
        w.start_block()?;
        if variant_index.is_some() {
            write!(w, "override ")?;
        }
        write!(w, "fun serialize(serializer: Serializer) ")?;
        if fields.is_empty() {
            w.empty_block()?;
        } else {
            w.start_block()?;
            push_serializer(w)?;
            if let Some(index) = variant_index {
                writeln!(w, "serializer.serialize_variant_index({index})")?;
            }
            for field in fields {
                write_serialize(w, &field.name.to_lower_camel_case(), &field.value, 0)?;
            }
            pop_serializer(w)?;
            w.end_block()?;
        }
        writeln!(w)?;

        if variant_index.is_none() {
            write_bincode_serialize(w)?;
            writeln!(w)?;
        }
        write!(w, "companion object ")?;
        w.start_block()?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        w.start_block()?;
        if fields.is_empty() {
            writeln!(w, "return {name}()")?;
        } else {
            push_deserializer(w)?;
            for field in fields {
                write_deserialize(
                    w,
                    Some(&field.name.to_lower_camel_case()),
                    &field.value,
                    true,
                )?;
            }
            pop_deserializer(w)?;
            write!(w, "return {name}(")?;
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write!(w, "{}", field.name.to_lower_camel_case())?;
            }
            writeln!(w, ")")?;
        }
        w.end_block()?;
        if variant_index.is_none() {
            writeln!(w)?;
            write_bincode_deserialize(w, name)?;
        }
        w.end_block()?;
        w.end_block()?;
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
            w.start_block()?;
            push_serializer(w)?;
            writeln!(w, "serializer.serialize_variant_index(ordinal)")?;
            pop_serializer(w)?;
            w.end_block()?;
            writeln!(w)?;

            write_bincode_serialize(w)?;
            writeln!(w)?;

            write!(w, "companion object ")?;
            w.start_block()?;

            writeln!(w, "@Throws(DeserializationError::class)")?;
            write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
            w.start_block()?;
            push_deserializer(w)?;
            writeln!(w, "val index = deserializer.deserialize_variant_index()")?;
            pop_deserializer(w)?;
            write!(w, "return when (index) ")?;
            w.start_block()?;
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
            w.end_block()?;
            w.end_block()?;
            writeln!(w)?;

            write_bincode_deserialize(w, name)?;
            w.end_block()?;
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
    w.start_block()?;

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
        write!(w, "companion object ")?;
        w.start_block()?;
        writeln!(w, "@Throws(DeserializationError::class)")?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        w.start_block()?;
        writeln!(w, "val index = deserializer.deserialize_variant_index()")?;
        write!(w, "return when (index) ")?;
        w.start_block()?;
        for (i, variant) in variants.iter().enumerate() {
            let name = &variant.name;
            writeln!(w, "{i} -> {name}.deserialize(deserializer)")?;
        }
        writeln!(
            w,
            r#"else -> throw DeserializationError("Unknown variant index for {name}: $index")"#
        )?;
        w.end_block()?;
        w.end_block()?;
        writeln!(w)?;
        write_bincode_deserialize(w, name)?;
        w.end_block()?;
    }
    w.end_block()
}

fn named<Format: Clone>(formats: &[Format]) -> Vec<Named<Format>> {
    formats
        .iter()
        .enumerate()
        .map(|(i, f)| Named {
            name: format!("field{i}"),
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

fn write_serialize<W: IndentWrite>(
    w: &mut W,
    field_name: &str,
    format: &Format,
    level: usize,
) -> Result<()> {
    match format {
        Format::Unit => writeln!(w, "serializer.serialize_unit({field_name})"),
        Format::Bool => writeln!(w, "serializer.serialize_bool({field_name})"),
        Format::I8 => writeln!(w, "serializer.serialize_i8({field_name})"),
        Format::I16 => writeln!(w, "serializer.serialize_i16({field_name})"),
        Format::I32 => writeln!(w, "serializer.serialize_i32({field_name})"),
        Format::I64 => writeln!(w, "serializer.serialize_i64({field_name})"),
        Format::I128 => writeln!(w, "serializer.serialize_i128(@Int128 {field_name})"),
        Format::U8 => writeln!(
            w,
            "serializer.serialize_u8(@Unsigned {field_name}.toByte())"
        ),
        Format::U16 => writeln!(
            w,
            "serializer.serialize_u16(@Unsigned {field_name}.toShort())"
        ),
        Format::U32 => writeln!(
            w,
            "serializer.serialize_u32(@Unsigned {field_name}.toInt())"
        ),
        Format::U64 => writeln!(
            w,
            "serializer.serialize_u64(@Unsigned {field_name}.toLong())"
        ),
        Format::U128 => writeln!(
            w,
            "serializer.serialize_u128(@Unsigned @Int128 {field_name})"
        ),
        Format::F32 => writeln!(w, "serializer.serialize_f32({field_name})"),
        Format::F64 => writeln!(w, "serializer.serialize_f64({field_name})"),
        Format::Char => writeln!(w, "serializer.serialize_char({field_name})"),
        Format::Str => writeln!(w, "serializer.serialize_str({field_name})"),
        Format::Bytes => writeln!(w, "serializer.serialize_bytes(Bytes.valueOf({field_name}))"),

        // Container types - these generate method calls with lambdas
        Format::Option(inner_format) => {
            write!(w, "{field_name}.serializeOptionOf(serializer) ")?;
            write_serialize_lambda(w, inner_format, level)?;
            Ok(())
        }

        Format::Seq(inner_format) | Format::Set(inner_format) => {
            write!(w, "{field_name}.serialize(serializer) ")?;
            write_serialize_lambda(w, inner_format, level)?;
            Ok(())
        }

        Format::Map { key, value } => {
            write!(w, "{field_name}.serialize(serializer) ")?;
            write_map_serialize_lambda(w, key, value, level)?;
            Ok(())
        }

        Format::TypeName(..) | Format::Tuple(..) | Format::TupleArray { .. } => {
            writeln!(w, "{field_name}.serialize(serializer)")
        }
        Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
    }
}

fn write_serialize_lambda<W: IndentWrite>(w: &mut W, format: &Format, level: usize) -> Result<()> {
    if format.is_leaf() {
        w.start_block()?;
        write_serialize(w, "it", format, level + 1)?;
    } else {
        let param_name = format!("level{}", level + 1);
        w.start_block_no_newline()?;
        writeln!(w, " {param_name} ->")?;
        write_serialize(w, &param_name, format, level + 1)?;
    }
    w.end_block()
}

fn write_map_serialize_lambda<W: IndentWrite>(
    w: &mut W,
    key_format: &Format,
    value_format: &Format,
    level: usize,
) -> Result<()> {
    w.start_block_no_newline()?;
    writeln!(w, " key, value ->")?;

    if key_format.is_leaf() {
        write_serialize(w, "key", key_format, level + 1)?;
    } else {
        write!(w, "key.serialize(serializer) ")?;
        write_serialize_lambda(w, key_format, level + 1)?;
    }

    if value_format.is_leaf() {
        write_serialize(w, "value", value_format, level + 1)?;
    } else {
        write!(w, "value.serialize(serializer) ")?;
        write_serialize_lambda(w, value_format, level + 1)?;
    }

    w.end_block()
}

fn write_deserialize<W: IndentWrite>(
    w: &mut W,
    field_name: Option<&str>,
    format: &Format,
    newline: bool,
) -> Result<()> {
    let mut indented = false;
    if let Some(field_name) = field_name {
        write!(w, "val {field_name} =")?;
        if matches!(
            format,
            Format::Seq(..) | Format::Option(..) | Format::Set(..) | Format::Map { .. }
        ) {
            writeln!(w)?;
            w.indent();
            indented = true;
        } else {
            write!(w, " ")?;
        }
    }
    match format {
        Format::TypeName(qualified_name) => {
            let name = &qualified_name.name;
            write!(w, "{name}.deserialize(deserializer)")
        }
        Format::Unit => write!(w, "deserializer.deserialize_unit()"),
        Format::Bool => write!(w, "deserializer.deserialize_bool()"),
        Format::I8 => write!(w, "deserializer.deserialize_i8()"),
        Format::I16 => write!(w, "deserializer.deserialize_i16()"),
        Format::I32 => write!(w, "deserializer.deserialize_i32()"),
        Format::I64 => write!(w, "deserializer.deserialize_i64()"),
        Format::I128 => write!(w, "deserializer.deserialize_i128()"),
        Format::U8 => write!(w, "deserializer.deserialize_u8().toUByte()"),
        Format::U16 => write!(w, "deserializer.deserialize_u16().toUShort()"),
        Format::U32 => write!(w, "deserializer.deserialize_u32().toUInt()"),
        Format::U64 => write!(w, "deserializer.deserialize_u64().toULong()"),
        Format::U128 => write!(w, "deserializer.deserialize_u128()"),
        Format::F32 => write!(w, "deserializer.deserialize_f32()"),
        Format::F64 => write!(w, "deserializer.deserialize_f64()"),
        Format::Char => write!(w, "deserializer.deserialize_char()"),
        Format::Str => write!(w, "deserializer.deserialize_str()"),
        Format::Bytes => write!(w, "deserializer.deserialize_bytes().content()"),
        Format::Seq(format) => {
            write!(w, "deserializer.deserializeListOf ")?;
            write_deserialize_lambda(w, format)
        }
        Format::Option(format) => {
            write!(w, "deserializer.deserializeOptionOf ")?;
            write_deserialize_lambda(w, format)
        }
        Format::Set(format) => {
            write!(w, "deserializer.deserializeSetOf ")?;
            write_deserialize_lambda(w, format)
        }
        Format::Map { key, value } => {
            write!(w, "deserializer.deserializeMapOf ")?;
            write_map_deserialize_lambda(w, key, value)
        }
        Format::Tuple(formats) => {
            let len = formats.len();
            let typename = match len {
                0 => return Ok(()),
                1 => {
                    push_deserializer(w)?;
                    write_deserialize(w, Some("value"), &formats[0], true)?;
                    pop_deserializer(w)?;
                    return Ok(());
                }
                2 => "Pair".to_string(),
                3 => "Triple".to_string(),
                _ => format!("NTuple{len}"),
            };
            write!(w, "{typename}<")?;
            for (i, format) in formats.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                format.write(w)?;
            }
            writeln!(w, ">.deserialize(deserializer)")
        }
        Format::TupleArray { content, size } => {
            write!(w, "buildList({size}) {{ repeat({size}) {{ add(")?;
            write_deserialize(w, None, content, false)?;
            write!(w, ") }} }}")
        }
        Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
    }?;

    if newline
        && !matches!(
            format,
            Format::Seq(..)
                | Format::Option(..)
                | Format::Set(..)
                | Format::Map { .. }
                | Format::Tuple(..)
        )
    {
        writeln!(w)?;
    }

    if indented {
        w.unindent();
    }

    Ok(())
}

fn write_deserialize_lambda<W: IndentWrite>(w: &mut W, format: &Format) -> Result<()> {
    w.start_block()?;
    write_deserialize(w, None, format, true)?;
    w.end_block()
}

fn write_map_deserialize_lambda<W: IndentWrite>(
    w: &mut W,
    key_format: &Format,
    value_format: &Format,
) -> Result<()> {
    w.start_block()?;
    write!(w, "val key =")?;
    if key_format.is_leaf() {
        write!(w, " ")?;
        write_deserialize(w, None, key_format, true)?;
    } else {
        writeln!(w)?;
        w.indent();
        write_deserialize(w, None, key_format, true)?;
        w.unindent();
    }
    write!(w, "val value =")?;
    if value_format.is_leaf() {
        write!(w, " ")?;
        write_deserialize(w, None, value_format, true)?;
    } else {
        writeln!(w)?;
        w.indent();
        write_deserialize(w, None, value_format, true)?;
        w.unindent();
    }
    writeln!(w, "Pair(key, value)")?;
    w.end_block()
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

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bincode;
#[cfg(test)]
mod tests_json;
