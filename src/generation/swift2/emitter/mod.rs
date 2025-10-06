use std::{collections::BTreeMap, io::Result};

use heck::ToLowerCamelCase as _;

use crate::{
    generation::{Container, Emitter, Encoding, WithEncoding, indent::IndentWrite},
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName, VariantFormat},
};

pub struct Swift;

impl Emitter<Swift> for WithEncoding<Container<'_>> {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        let WithEncoding {
            encoding,
            value:
                Container {
                    name: QualifiedTypeName { namespace: _, name },
                    format,
                },
        } = self;
        match format {
            ContainerFormat::UnitStruct(doc) => struct_(w, name, &[], doc, *encoding),
            ContainerFormat::NewTypeStruct(format, doc) => struct_(
                w,
                name,
                &[Named {
                    name: "value".to_string(),
                    doc: Doc::new(),
                    value: Format::clone(format),
                }],
                doc,
                *encoding,
            ),
            ContainerFormat::TupleStruct(formats, doc) => {
                struct_(w, name, &named(formats), doc, *encoding)
            }
            ContainerFormat::Struct(nameds, doc) => struct_(w, name, nameds, doc, *encoding),
            ContainerFormat::Enum(variants, doc) => enum_(w, name, variants, doc, *encoding),
        }
    }
}

enum Usage {
    Field,
    Parameter,
    Assignment,
}

impl Emitter<Swift> for (&Named<Format>, Usage) {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        let (Named { name, doc, value }, usage) = self;
        let name = &name.to_lower_camel_case();

        match usage {
            Usage::Field => {
                <Doc as Emitter<Swift>>::write(doc, w)?;
                write!(w, "public var {name}: ")?;
                <Format as Emitter<Swift>>::write(value, w)?;
                writeln!(w)
            }
            Usage::Parameter => {
                write!(w, "{name}: ")?;
                <Format as Emitter<Swift>>::write(value, w)
            }
            Usage::Assignment => writeln!(w, "self.{name} = {name}"),
        }
    }
}

impl Emitter<Swift> for Format {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        let write = <Format as Emitter<Swift>>::write::<W>;
        match &self {
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
            Format::I128 => write!(w, "BigInt"),
            Format::U8 => write!(w, "UInt8"),
            Format::U16 => write!(w, "UInt16"),
            Format::U32 => write!(w, "UInt32"),
            Format::U64 => write!(w, "UInt64"),
            Format::U128 => write!(w, "BigUInt"),
            Format::F32 => write!(w, "Float"),
            Format::F64 => write!(w, "Double"),
            Format::Char => write!(w, "Character"),
            Format::Str => write!(w, "String"),
            Format::Bytes => write!(w, "Array<UInt8>"),

            Format::Option(format) => {
                write(format, w)?;
                write!(w, "?")
            }
            Format::Seq(format)
            | Format::TupleArray {
                content: format,
                size: _,
            } => {
                write!(w, "Array<")?;
                write(format, w)?;
                write!(w, ">")
            }
            Format::Set(format) => {
                write!(w, "Set<")?;
                write(format, w)?;
                write!(w, ">")
            }
            Format::Map { key, value } => {
                write!(w, "Dictionary<")?;
                write(key, w)?;
                write!(w, ", ")?;
                write(value, w)?;
                write!(w, ">")
            }
            Format::Tuple(formats) => {
                let len = formats.len();
                if len == 1 {
                    // A single-element tuple is just the element itself
                    write(&formats[0], w)
                } else {
                    // Other tuples (including unit)
                    write!(w, "(")?;
                    for (i, format) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write(format, w)?;
                    }
                    write!(w, ")")
                }
            }
        }
    }
}

impl Emitter<Swift> for WithEncoding<&Named<VariantFormat>> {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        let WithEncoding {
            encoding: _,
            value:
                Named {
                    name,
                    doc,
                    value: format,
                },
        } = self;
        let name = name.to_lower_camel_case();

        <Doc as Emitter<Swift>>::write(doc, w)?;

        match format {
            VariantFormat::Variable(_variable) => {
                unreachable!("placeholders should not get this far")
            }
            VariantFormat::Unit => {
                writeln!(w, "case {name}")
            }
            VariantFormat::NewType(format) => {
                write!(w, "case {name}(")?;
                <Format as Emitter<Swift>>::write(format, w)?;
                writeln!(w, ")")
            }
            VariantFormat::Tuple(formats) => {
                write!(w, "case {name}(")?;
                for (i, format) in formats.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    <Format as Emitter<Swift>>::write(format, w)?;
                }
                writeln!(w, ")")
            }
            VariantFormat::Struct(nameds) => {
                write!(w, "case {name}(")?;
                for (i, format) in nameds.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    <(&Named<Format>, Usage) as Emitter<Swift>>::write(
                        &(format, Usage::Parameter),
                        w,
                    )?;
                }
                writeln!(w, ")")
            }
        }
    }
}

impl Emitter<Swift> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }

        Ok(())
    }
}

fn struct_<W: IndentWrite>(
    w: &mut W,
    name: &str,
    fields: &[Named<Format>],
    doc: &Doc,
    _encoding: Encoding,
) -> Result<()> {
    <Doc as Emitter<Swift>>::write(doc, w)?;

    write!(w, "public struct {name}: Hashable ")?;

    w.start_block()?;
    for field in fields {
        <(&Named<Format>, Usage) as Emitter<Swift>>::write(&(field, Usage::Field), w)?;
    }

    if !fields.is_empty() {
        writeln!(w)?;
    }

    write!(w, "public init(")?;
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(w, ", ")?;
        }
        <(&Named<Format>, Usage) as Emitter<Swift>>::write(&(field, Usage::Parameter), w)?;
    }
    write!(w, ") ")?;
    w.start_block()?;
    for field in fields {
        <(&Named<Format>, Usage) as Emitter<Swift>>::write(&(field, Usage::Assignment), w)?;
    }
    w.end_block()?;
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
    <Doc as Emitter<Swift>>::write(doc, w)?;

    write!(w, "public enum {name}: Hashable ")?;

    w.start_block()?;

    for format in variants.values() {
        let variant = WithEncoding {
            encoding,
            value: format,
        };
        variant.write(w)?;
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

#[cfg(test)]
mod tests;
