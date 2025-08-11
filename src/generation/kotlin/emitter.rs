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
            (_name, ContainerFormat::NewTypeStruct(_format, _doc)) => {}
            (_name, ContainerFormat::TupleStruct(_formats, _doc)) => {}
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

        Ok(())
    }
}

impl Emitter<Kotlin> for Named<Format> {
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()> {
        let name = &self.name;
        for comment in self.doc.comments() {
            writeln!(writer, "/// {comment}")?;
        }
        write!(writer, "val {name}: ")?;
        match &self.value {
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
            Format::Tuple(_formats) => todo!(),
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

impl Emitter<Kotlin> for Named<VariantFormat> {
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()> {
        let name = &self.name;
        let comments = self.doc.comments();
        if !comments.is_empty() {
            writeln!(writer)?;
            for comment in comments {
                writeln!(writer, "/// {comment}")?;
            }
        }

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
                let name = &qualified_type_name.name;
                write!(writer, "{name}")
            }
            Format::Unit => todo!(),
            Format::Bool => todo!(),
            Format::I8 => todo!(),
            Format::I16 => todo!(),
            Format::I32 => write!(writer, "Int"),
            Format::I64 => todo!(),
            Format::I128 => todo!(),
            Format::U8 => todo!(),
            Format::U16 => todo!(),
            Format::U32 => todo!(),
            Format::U64 => todo!(),
            Format::U128 => todo!(),
            Format::F32 => todo!(),
            Format::F64 => todo!(),
            Format::Char => todo!(),
            Format::Str => write!(writer, "String"),
            Format::Bytes => write!(writer, "ByteArray"),
            Format::Option(_format) => todo!(),
            Format::Seq(_format) => todo!(),
            Format::Map { key, value } => {
                write!(writer, "Map<")?;
                key.write(writer)?;
                write!(writer, ", ")?;
                value.write(writer)?;
                write!(writer, ">")
            }
            Format::Tuple(_formats) => todo!(),
            Format::TupleArray {
                content: _,
                size: _,
            } => todo!(),
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
    )?;
    writeln!(writer)
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
