use std::io::{Result, Write};

use indoc::writedoc;

use crate::{
    generation::{Emitter, indent::IndentWrite, module::Module},
    reflection::format::{ContainerFormat, Format, Named, QualifiedTypeName},
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
            (name, ContainerFormat::UnitStruct) => {
                let name = &name.name;
                data_object(writer, name)?;
            }
            (_name, ContainerFormat::NewTypeStruct(_format)) => {}
            (_name, ContainerFormat::TupleStruct(_formats)) => {}
            (name, ContainerFormat::Struct(nameds)) => {
                let name = &name.name;
                if nameds.is_empty() {
                    data_object(writer, name)?;
                } else {
                    data_class(writer, name, nameds)?;
                }
            }
            (_name, ContainerFormat::Enum(_btree_map)) => {}
        }

        Ok(())
    }
}

impl Emitter<Kotlin> for Named<Format> {
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()> {
        let name = &self.name;
        match &self.value {
            Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Format::TypeName(qualified_type_name) => {
                write!(writer, "val {name}: {ty}", ty = qualified_type_name.name)
            }
            Format::Unit => todo!(),
            Format::Bool => todo!(),
            Format::I8 => todo!(),
            Format::I16 => todo!(),
            Format::I32 => todo!(),
            Format::I64 => todo!(),
            Format::I128 => todo!(),
            Format::U8 => write!(writer, "val {name}: UByte"),
            Format::U16 => todo!(),
            Format::U32 => todo!(),
            Format::U64 => todo!(),
            Format::U128 => todo!(),
            Format::F32 => todo!(),
            Format::F64 => todo!(),
            Format::Char => todo!(),
            Format::Str => write!(writer, "/// This is another comment\nval {name}: String"),
            Format::Bytes => todo!(),
            Format::Option(format) => {
                write!(writer, "val {name}: ")?;
                format.write(writer)?;
                write!(writer, "? = null")
            }
            Format::Seq(format) => {
                write!(writer, "val {name}: List<")?;
                format.write(writer)?;
                write!(writer, ">")
            }
            Format::Map { key: _, value: _ } => todo!(),
            Format::Tuple(_formats) => todo!(),
            Format::TupleArray {
                content: _,
                size: _,
            } => todo!(),
        }
    }
}

impl Emitter<Kotlin> for Format {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match &self {
            Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Format::TypeName(_qualified_type_name) => todo!(),
            Format::Unit => todo!(),
            Format::Bool => todo!(),
            Format::I8 => todo!(),
            Format::I16 => todo!(),
            Format::I32 => todo!(),
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
            Format::Bytes => todo!(),
            Format::Option(_format) => todo!(),
            Format::Seq(_format) => todo!(),
            Format::Map { key: _, value: _ } => todo!(),
            Format::Tuple(_formats) => todo!(),
            Format::TupleArray {
                content: _,
                size: _,
            } => todo!(),
        }
    }
}

fn data_object<W: Write>(writer: &mut W, name: &String) -> Result<()> {
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
) -> Result<()> {
    writedoc!(
        writer,
        "
            /// This is a comment.
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
