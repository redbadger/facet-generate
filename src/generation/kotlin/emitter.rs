use std::{
    collections::BTreeMap,
    io::{Result, Write},
};

use indoc::writedoc;

use crate::{
    generation::{Emitter, indent::IndentWrite, module::Module},
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName, VariantFormat},
};

pub struct Kotlin;

impl Emitter<Kotlin> for Module {
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()> {
        let name = &self.config().module_name;
        writedoc!(
            writer,
            "
                package {name}

                import kotlinx.serialization.*
                import kotlinx.serialization.builtins.*
                import kotlinx.serialization.descriptors.*
                import kotlinx.serialization.encoding.*
                import kotlinx.serialization.json.*
                import kotlinx.serialization.modules.*

            "
        )?;

        Ok(())
    }
}

impl Emitter<Kotlin> for (&QualifiedTypeName, &ContainerFormat) {
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()> {
        match self {
            (name, ContainerFormat::UnitStruct(doc)) => {
                let name = &name.name;
                data_object(writer, name, doc)?;
            }
            (name, ContainerFormat::NewTypeStruct(format, doc)) => {
                let name = &name.name;
                type_alias(writer, name, format, doc)?;
            }
            (name, ContainerFormat::TupleStruct(formats, doc)) => {
                let name = &name.name;
                let nameds = formats
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Named {
                        name: format!("field_{i}"),
                        doc: Doc::new(),
                        value: f.clone(),
                    })
                    .collect::<Vec<_>>();
                data_class(writer, name, &nameds, doc)?;
            }
            (name, ContainerFormat::Struct(nameds, doc)) => {
                let name = &name.name;
                if nameds.is_empty() {
                    data_object(writer, name, doc)?;
                } else {
                    data_class(writer, name, nameds, doc)?;
                }
            }
            (name, ContainerFormat::Enum(btree_map, doc)) => {
                let name = &name.name;
                enum_class(writer, name, btree_map, doc)?;
            }
        }

        writeln!(writer)?;

        Ok(())
    }
}

impl Emitter<Kotlin> for Named<Format> {
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()> {
        for comment in self.doc.comments() {
            writeln!(writer, "/// {comment}")?;
        }

        let name = &self.name;
        write!(writer, "val {name}: ")?;

        self.value.write(writer)?;

        Ok(())
    }
}

impl Emitter<Kotlin> for Named<VariantFormat> {
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()> {
        for comment in self.doc.comments() {
            writeln!(writer, "/// {comment}")?;
        }

        let name = &self.name;
        match &self.value {
            VariantFormat::Variable(_variable) => {
                unreachable!("placeholders should not get this far")
            }
            VariantFormat::Unit => {
                let name_upper = name.to_uppercase();
                write!(writer, r#"@SerialName("{name}") {name_upper}"#)?;
            }
            VariantFormat::NewType(_format) => todo!(),
            VariantFormat::Tuple(_formats) => todo!(),
            VariantFormat::Struct(_nameds) => todo!(),
        }

        Ok(())
    }
}

impl Emitter<Kotlin> for Format {
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()> {
        match &self {
            Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Format::TypeName(qualified_type_name) => {
                write!(writer, "{ty}", ty = qualified_type_name.name)
            }
            Format::Unit => write!(writer, "Unit"),
            Format::Bool => write!(writer, "Boolean"),
            Format::I8 => write!(writer, "Byte"),
            Format::I16 => write!(writer, "Short"),
            Format::I32 => write!(writer, "Int"),
            Format::I64 => write!(writer, "Long"),
            Format::I128 => write!(writer, "java.math.@com.novi.serde.Int128 BigInteger"),
            Format::U8 => write!(writer, "UByte"),
            Format::U16 => write!(writer, "UShort"),
            Format::U32 => write!(writer, "UInt"),
            Format::U64 => write!(writer, "ULong"),
            Format::U128 => write!(
                writer,
                "java.math.@com.novi.serde.Unsigned @com.novi.serde.Int128 BigInteger"
            ),
            Format::F32 => write!(writer, "Float"),
            Format::F64 => write!(writer, "Double"),
            Format::Char | Format::Str => write!(writer, "String"),
            Format::Bytes => todo!(),
            Format::Option(format) => {
                format.write(writer)?;
                write!(writer, "? = null")
            }
            Format::Seq(format) => {
                write!(writer, "List<")?;
                format.write(writer)?;
                write!(writer, ">")
            }
            Format::Map { key, value } => {
                write!(writer, "Map<")?;
                key.write(writer)?;
                write!(writer, ", ")?;
                value.write(writer)?;
                write!(writer, ">")
            }
            Format::Tuple(formats) => {
                let len = formats.len();
                match len {
                    1 => return formats[0].write(writer),
                    2 => write!(writer, "Pair<")?,
                    3 => write!(writer, "Triple<")?,
                    _ => write!(writer, "NTuple{len}<")?,
                }
                for (i, format) in formats.iter().enumerate() {
                    if i > 0 {
                        write!(writer, ", ")?;
                    }
                    format.write(writer)?;
                }
                write!(writer, ">")
            }
            Format::TupleArray {
                content: format,
                size: _,
            } => {
                write!(writer, "List<")?;
                format.write(writer)?;
                write!(writer, ">")
            }
        }
    }
}

fn data_object<W: Write>(writer: &mut W, name: &String, doc: &Doc) -> Result<()> {
    for comment in doc.comments() {
        writeln!(writer, "/// {comment}")?;
    }

    writedoc!(
        writer,
        "
            @Serializable
            data object {name}
        "
    )
}

fn data_class<W: IndentWrite>(
    writer: &mut W,
    name: &String,
    nameds: &[Named<Format>],
    doc: &Doc,
) -> Result<()> {
    for comment in doc.comments() {
        writeln!(writer, "/// {comment}")?;
    }

    writedoc!(
        writer,
        "
            @Serializable
            data class {name} (
        "
    )?;

    writer.indent();
    for (i, named) in nameds.iter().enumerate() {
        if i > 0 {
            writeln!(writer, ",")?;
        }
        named.write(writer)?;
    }
    writer.unindent();

    writeln!(writer)?;
    writeln!(writer, ")")?;

    Ok(())
}

fn enum_class<W: IndentWrite>(
    writer: &mut W,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
) -> Result<()> {
    for comment in doc.comments() {
        writeln!(writer, "/// {comment}")?;
    }

    writedoc!(
        writer,
        "
            @Serializable
            enum class {name} {{
        "
    )?;

    writer.indent();
    for (i, named) in variants {
        if *i > 0 {
            writeln!(writer, ",")?;
        }
        named.write(writer)?;
    }
    writeln!(writer, ";")?;
    writeln!(writer)?;

    writedoc!(
        writer,
        "
        val serialName: String
            get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
    "
    )?;
    writer.unindent();

    writeln!(writer)?;
    writeln!(writer, "}}")?;

    Ok(())
}

fn type_alias<W: IndentWrite>(
    writer: &mut W,
    name: &str,
    format: &Format,
    doc: &Doc,
) -> Result<()> {
    for comment in doc.comments() {
        writeln!(writer, "/// {comment}")?;
    }

    write!(writer, "typealias {name} = ")?;
    format.write(writer)?;
    writeln!(writer)?;

    Ok(())
}

#[cfg(test)]
#[path = "emitter_tests.rs"]
mod tests;
