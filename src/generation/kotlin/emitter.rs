use std::io::{Result, Write};

use indoc::writedoc;

use crate::{
    generation::{Emitter, module::Module},
    reflection::format::{ContainerFormat, QualifiedTypeName},
};

pub struct Kotlin;

impl Emitter<Kotlin> for Module {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let name = &self.config().module_name;
        writedoc!(
            writer,
            r"
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
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            (name, ContainerFormat::UnitStruct) => {
                let name = &name.name;
                writeln!(writer, "@Serializable data object {name}")?;
            }
            (_name, ContainerFormat::NewTypeStruct(_format)) => {}
            (_name, ContainerFormat::TupleStruct(_formats)) => {}
            (_name, ContainerFormat::Struct(_nameds)) => {}
            (_name, ContainerFormat::Enum(_btree_map)) => {}
        }

        Ok(())
    }
}
