use std::io::Result;

use crate::{
    generation::{Emitter, Encoding, indent::IndentWrite},
    reflection::format::{ContainerFormat, QualifiedTypeName},
};

pub struct Swift;

impl Emitter<Swift> for (Encoding, (&QualifiedTypeName, &ContainerFormat)) {
    fn write<W: IndentWrite>(&self, _w: &mut W) -> Result<()> {
        Ok(())
    }
}
