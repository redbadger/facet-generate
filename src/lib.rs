#![allow(clippy::missing_panics_doc)]

pub mod error;
pub mod generation;
pub mod reflection;

#[cfg(all(test, feature = "generate"))]
mod tests;

use std::collections::BTreeMap;

use crate::{
    error::Error,
    reflection::format::{ContainerFormat, QualifiedTypeName},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub type Registry = BTreeMap<QualifiedTypeName, ContainerFormat>;

#[cfg(test)]
#[macro_export]
macro_rules! emit {
    ($($ty:ident),*) => {
        {
            use $crate::generation::indent::{IndentConfig, IndentedWriter};
            let mut out = Vec::new();
            let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let registry = $crate::reflection::RegistryBuilder::new()
                $(.add_type::<$ty>())*
                .build();
            for item in &registry {
                item.write(&mut w).unwrap();
            }
            String::from_utf8(out).unwrap()
        }
    };
}
