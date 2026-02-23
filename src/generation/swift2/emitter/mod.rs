#![allow(clippy::too_many_lines, dead_code)]
use std::{
    collections::BTreeMap,
    io::{Result, Write},
};

use heck::ToLowerCamelCase as _;
use indoc::writedoc;

use crate::{
    generation::{Container, Encoding, LanguageEmitter, WithEncoding, indent::IndentWrite},
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName, VariantFormat},
};

pub struct Swift;

enum Usage {
    Field,
    Parameter,
    Argument,
    Assignment,
    Serialize { receiver: String },
    Deserialize { receiver: String },
}

impl LanguageEmitter for Swift {
    fn write_container<W: IndentWrite>(
        container: &WithEncoding<Container<'_>>,
        w: &mut W,
    ) -> Result<()> {
        let WithEncoding {
            encoding,
            value:
                Container {
                    name: QualifiedTypeName { namespace: _, name },
                    format,
                },
        } = container;
        match format {
            ContainerFormat::UnitStruct(doc) => struct_(w, name, &[], doc, *encoding),
            ContainerFormat::NewTypeStruct(format, doc) => struct_(
                w,
                name,
                &[&Named::new(format, "value".to_string())],
                doc,
                *encoding,
            ),
            ContainerFormat::TupleStruct(formats, doc) => {
                let formats = named(formats, "field");
                struct_(w, name, &formats.iter().collect::<Vec<_>>(), doc, *encoding)
            }
            ContainerFormat::Struct(nameds, doc) => {
                struct_(w, name, &nameds.iter().collect::<Vec<_>>(), doc, *encoding)
            }
            ContainerFormat::Enum(variants, doc) => enum_(w, name, variants, doc, *encoding),
        }
    }

    fn write_format<W: IndentWrite>(format: &Format, w: &mut W) -> Result<()> {
        match format {
            Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Format::TypeName(qualified_type_name) => {
                write!(w, "{ty}", ty = qualified_type_name.name)
            }
            Format::Unit => write!(w, "()"),
            Format::Bool => write!(w, "Bool"),
            Format::I8 => write!(w, "Int8"),
            Format::I16 => write!(w, "Int16"),
            Format::I32 => write!(w, "Int32"),
            Format::I64 => write!(w, "Int64"),
            Format::I128 => write!(w, "Int128"),
            Format::U8 => write!(w, "UInt8"),
            Format::U16 => write!(w, "UInt16"),
            Format::U32 => write!(w, "UInt32"),
            Format::U64 => write!(w, "UInt64"),
            Format::U128 => write!(w, "UInt128"),
            Format::F32 => write!(w, "Float"),
            Format::F64 => write!(w, "Double"),
            Format::Char => write!(w, "Character"),
            Format::Str => write!(w, "String"),
            Format::Bytes => write!(w, "Array<UInt8>"),

            Format::Option(format) => {
                Self::write_format(format, w)?;
                write!(w, "?")
            }
            Format::Seq(format)
            | Format::TupleArray {
                content: format,
                size: _,
            } => {
                write!(w, "Array<")?;
                Self::write_format(format, w)?;
                write!(w, ">")
            }
            Format::Set(format) => {
                write!(w, "Set<")?;
                Self::write_format(format, w)?;
                write!(w, ">")
            }
            Format::Map { key, value } => {
                write!(w, "Dictionary<")?;
                Self::write_format(key, w)?;
                write!(w, ", ")?;
                Self::write_format(value, w)?;
                write!(w, ">")
            }
            Format::Tuple(formats) => {
                let len = formats.len();
                if len == 1 {
                    // A single-element tuple is just the element itself
                    Self::write_format(&formats[0], w)
                } else {
                    // Other tuples (including unit)
                    write!(w, "(")?;
                    for (i, format) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        Self::write_format(format, w)?;
                    }
                    write!(w, ")")
                }
            }
        }
    }

    fn write_doc<W: IndentWrite>(doc: &Doc, w: &mut W) -> Result<()> {
        for comment in doc.comments() {
            writeln!(w, "/// {comment}")?;
        }

        Ok(())
    }
}

fn write_named_format<W: IndentWrite>(
    field: &Named<Format>,
    usage: Usage,
    w: &mut W,
) -> Result<()> {
    let Named { name, doc, value } = field;
    let name = &name.to_lower_camel_case();

    match usage {
        Usage::Field => {
            Swift::write_doc(doc, w)?;
            write!(w, "public var {name}: ")?;
            Swift::write_format(value, w)?;
            writeln!(w)
        }
        Usage::Parameter => {
            write!(w, "{name}: ")?;
            Swift::write_format(value, w)
        }
        Usage::Argument => {
            write!(w, "{name}: {name}")
        }
        Usage::Assignment => writeln!(w, "self.{name} = {name}"),
        Usage::Serialize { receiver } => match value {
            Format::Variable(_) => unreachable!("placeholders should not get this far"),
            Format::TypeName(_) => {
                writeln!(w, "try {receiver}.{name}.serialize(serializer: serializer)")
            }
            Format::Unit => {
                writeln!(w, "try serializer.serialize_unit(value: {receiver}.{name})")
            }
            Format::Bool => {
                writeln!(w, "try serializer.serialize_bool(value: {receiver}.{name})")
            }
            Format::I8 => writeln!(w, "try serializer.serialize_i8(value: {receiver}.{name})"),
            Format::I16 => {
                writeln!(w, "try serializer.serialize_i16(value: {receiver}.{name})")
            }
            Format::I32 => {
                writeln!(w, "try serializer.serialize_i32(value: {receiver}.{name})")
            }
            Format::I64 => {
                writeln!(w, "try serializer.serialize_i64(value: {receiver}.{name})")
            }
            Format::I128 => {
                writeln!(w, "try serializer.serialize_i128(value: {receiver}.{name})")
            }
            Format::U8 => writeln!(w, "try serializer.serialize_u8(value: {receiver}.{name})"),
            Format::U16 => {
                writeln!(w, "try serializer.serialize_u16(value: {receiver}.{name})")
            }
            Format::U32 => {
                writeln!(w, "try serializer.serialize_u32(value: {receiver}.{name})")
            }
            Format::U64 => {
                writeln!(w, "try serializer.serialize_u64(value: {receiver}.{name})")
            }
            Format::U128 => {
                writeln!(w, "try serializer.serialize_u128(value: {receiver}.{name})")
            }
            Format::F32 => {
                writeln!(w, "try serializer.serialize_f32(value: {receiver}.{name})")
            }
            Format::F64 => {
                writeln!(w, "try serializer.serialize_f64(value: {receiver}.{name})")
            }
            Format::Char => {
                writeln!(w, "try serializer.serialize_char(value: {receiver}.{name})")
            }
            Format::Str => {
                writeln!(w, "try serializer.serialize_str(value: {receiver}.{name})")
            }
            Format::Bytes => writeln!(
                w,
                "try serializer.serialize_bytes(value: {receiver}.{name})"
            ),
            Format::Option(_format) => todo!(),
            Format::Seq(_format) => todo!(),
            Format::Set(_format) => todo!(),
            Format::Map { key: _, value: _ } => todo!(),
            Format::Tuple(formats) => {
                push_serializer(w)?;
                let formats = named(formats, "");
                for format in formats {
                    write_named_format(
                        &format,
                        Usage::Serialize {
                            receiver: name.clone(),
                        },
                        w,
                    )?;
                }
                pop_serializer(w)
            }
            Format::TupleArray {
                content: _,
                size: _,
            } => todo!(),
        },
        Usage::Deserialize { receiver: _ } => match value {
            Format::Variable(_) => unreachable!("placeholders should not get this far"),
            Format::TypeName(qualified_type_name) => {
                let type_name = &qualified_type_name.name;
                writeln!(
                    w,
                    "let {name} = try {type_name}.deserialize(deserializer: deserializer)"
                )
            }
            Format::Unit => writeln!(w, "let {name} = try deserializer.deserialize_unit()"),
            Format::Bool => writeln!(w, "let {name} = try deserializer.deserialize_bool()"),
            Format::I8 => writeln!(w, "let {name} = try deserializer.deserialize_i8()"),
            Format::I16 => writeln!(w, "let {name} = try deserializer.deserialize_i16()"),
            Format::I32 => writeln!(w, "let {name} = try deserializer.deserialize_i32()"),
            Format::I64 => writeln!(w, "let {name} = try deserializer.deserialize_i64()"),
            Format::I128 => writeln!(w, "let {name} = try deserializer.deserialize_i128()"),
            Format::U8 => writeln!(w, "let {name} = try deserializer.deserialize_u8()"),
            Format::U16 => writeln!(w, "let {name} = try deserializer.deserialize_u16()"),
            Format::U32 => writeln!(w, "let {name} = try deserializer.deserialize_u32()"),
            Format::U64 => writeln!(w, "let {name} = try deserializer.deserialize_u64()"),
            Format::U128 => writeln!(w, "let {name} = try deserializer.deserialize_u128()"),
            Format::F32 => writeln!(w, "let {name} = try deserializer.deserialize_f32()"),
            Format::F64 => writeln!(w, "let {name} = try deserializer.deserialize_f64()"),
            Format::Char => writeln!(w, "let {name} = try deserializer.deserialize_char()"),
            Format::Str => writeln!(w, "let {name} = try deserializer.deserialize_str()"),
            Format::Bytes => writeln!(w, "let {name} = try deserializer.deserialize_bytes()"),
            Format::Option(_format) => todo!(),
            Format::Seq(_format) => todo!(),
            Format::Set(_format) => todo!(),
            Format::Map { key: _, value: _ } => todo!(),
            Format::Tuple(formats) => {
                push_deserializer(w)?;
                let formats = named(formats, name);
                for (i, format) in formats.iter().enumerate() {
                    write_named_format(
                        format,
                        Usage::Deserialize {
                            receiver: i.to_string(),
                        },
                        w,
                    )?;
                }
                write!(w, "let {name} = (")?;
                for (i, format) in formats.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    write!(w, "{}", format.name)?;
                }
                writeln!(w, ")")?;
                pop_deserializer(w)
            }
            Format::TupleArray {
                content: _,
                size: _,
            } => todo!(),
        },
    }
}

fn write_variant<W: IndentWrite>(
    variant: &Named<VariantFormat>,
    usage: Usage,
    w: &mut W,
) -> Result<()> {
    let Named {
        name,
        doc,
        value: format,
    } = variant;
    let name = name.to_lower_camel_case();

    Swift::write_doc(doc, w)?;

    match usage {
        Usage::Field => match format {
            VariantFormat::Variable(_variable) => {
                unreachable!("placeholders should not get this far")
            }
            VariantFormat::Unit => {
                writeln!(w, "case {name}")
            }
            VariantFormat::NewType(format) => {
                write!(w, "case {name}(")?;
                Swift::write_format(format, w)?;
                writeln!(w, ")")
            }
            VariantFormat::Tuple(formats) => {
                write!(w, "case {name}(")?;
                for (i, format) in formats.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    Swift::write_format(format, w)?;
                }
                writeln!(w, ")")
            }
            VariantFormat::Struct(nameds) => {
                write!(w, "case {name}(")?;
                for (i, format) in nameds.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    write_named_format(format, Usage::Parameter, w)?;
                }
                writeln!(w, ")")
            }
        },
        Usage::Parameter | Usage::Argument | Usage::Assignment => Ok(()),
        Usage::Deserialize { receiver: index } => {
            writeln!(w, "case {index}:")?;
            w.indent();
            pop_deserializer(w)?;
            writeln!(w, "return .{name}")?;
            w.unindent();
            Ok(())
        }
        Usage::Serialize { receiver: index } => {
            writeln!(w, "case .{name}:")?;
            w.indent();
            writeln!(w, "try serializer.serialize_variant_index(value: {index})")?;
            w.unindent();
            Ok(())
        }
    }
}

fn struct_<W: IndentWrite>(
    w: &mut W,
    name: &str,
    fields: &[&Named<Format>],
    doc: &Doc,
    encoding: Encoding,
) -> Result<()> {
    Swift::write_doc(doc, w)?;

    write!(w, "public struct {name}: Hashable ")?;

    w.start_block()?;
    for field in fields {
        write_named_format(field, Usage::Field, w)?;
    }

    if !fields.is_empty() {
        writeln!(w)?;
    }

    write!(w, "public init(")?;
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(w, ", ")?;
        }
        write_named_format(field, Usage::Parameter, w)?;
    }
    write!(w, ") ")?;
    w.start_block()?;
    for field in fields {
        write_named_format(field, Usage::Assignment, w)?;
    }
    w.end_block()?;

    match encoding {
        Encoding::None => {}
        Encoding::Json => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            w.start_block()?;
            push_serializer(w)?;
            for field in fields {
                write_named_format(
                    field,
                    Usage::Serialize {
                        receiver: "self".to_string(),
                    },
                    w,
                )?;
            }
            pop_serializer(w)?;
            w.end_block()?;
            write_json_serialize(w)?;
            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            w.start_block()?;
            push_deserializer(w)?;
            for field in fields {
                write_named_format(
                    field,
                    Usage::Deserialize {
                        receiver: "self".to_string(),
                    },
                    w,
                )?;
            }
            pop_deserializer(w)?;
            write!(w, "return {name}(")?;
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write_named_format(field, Usage::Argument, w)?;
            }
            writeln!(w, ")")?;
            w.end_block()?;
            write_json_deserialize(w, name)?;
        }
        Encoding::Bincode => todo!(),
        Encoding::Bcs => todo!(),
    }

    w.end_block()?;

    Ok(())
}

fn enum_<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    encoding: Encoding,
) -> Result<()> {
    Swift::write_doc(doc, w)?;

    write!(w, "public enum {name}: Hashable ")?;

    w.start_block()?;

    let variants = variants.values().collect::<Vec<_>>();

    for format in &variants {
        write_variant(format, Usage::Field, w)?;
    }

    match encoding {
        Encoding::None => {}
        Encoding::Json => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            w.start_block()?;
            push_serializer(w)?;
            write!(w, "switch self ")?;
            w.start_block()?;
            w.unindent(); // in Swift, `case` is not indented
            for (i, variant) in variants.iter().enumerate() {
                write_variant(
                    &variant.without_docs(),
                    Usage::Serialize {
                        receiver: i.to_string(),
                    },
                    w,
                )?;
            }
            w.indent();
            w.end_block()?;
            pop_serializer(w)?;
            w.end_block()?;
            write_json_serialize(w)?;

            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            w.start_block()?;
            writeln!(
                w,
                "let index = try deserializer.deserialize_variant_index()"
            )?;
            push_deserializer(w)?;
            write!(w, "switch index ")?;
            w.start_block()?;
            w.unindent(); // in Swift, `case` is not indented
            for (i, variant) in variants.iter().enumerate() {
                write_variant(
                    &variant.without_docs(),
                    Usage::Deserialize {
                        receiver: i.to_string(),
                    },
                    w,
                )?;
            }
            writeln!(
                w,
                r#"default: throw DeserializationError.invalidInput(issue: "Unknown variant index for {name}: \(index)")"#
            )?;
            w.indent();
            w.end_block()?;
            w.end_block()?;
            write_json_deserialize(w, name)?;
        }
        Encoding::Bincode => todo!(),
        Encoding::Bcs => todo!(),
    }

    w.end_block()
}

fn write_json_serialize<W: IndentWrite>(w: &mut W) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r"
        public func jsonSerialize() throws -> Array<UInt8> {{
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }}
        "
    )
}

fn write_json_deserialize<W: IndentWrite>(w: &mut W, name: &str) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r#"
        public static func jsonDeserialize(input: Array<UInt8>) throws -> {name} {{
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {{
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }}
            return obj
        }}
        "#
    )
}

fn push_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try serializer.increase_container_depth()")
}

fn pop_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try serializer.decrease_container_depth()")
}

fn push_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try deserializer.increase_container_depth()")
}

fn pop_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try deserializer.decrease_container_depth()")
}

fn named<Format: Clone>(formats: &[Format], prefix: &str) -> Vec<Named<Format>> {
    formats
        .iter()
        .enumerate()
        .map(|(i, f)| Named::new(f, format!("{prefix}{i}")))
        .collect()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_json;
