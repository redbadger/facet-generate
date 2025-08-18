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

#[macro_export]
macro_rules! emit {
    ($($ty:ident),* as $encoding:path) => {
        || -> anyhow::Result<String> {
            use $crate::generation::indent::{IndentConfig, IndentedWriter};
            use std::io::Write as _;
            let mut out = Vec::new();
            let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let registry = $crate::reflect!($($ty),*);
            for (i, item) in registry.iter().enumerate() {
                if i > 0 {
                    writeln!(&mut w)?;
                }
                ($encoding, item).write(&mut w)?;
            }
            let out = String::from_utf8(out)?;

            Ok(out)
        }()
    };
}

#[macro_export]
macro_rules! reflect {
    ($($ty:ident),*) => {{
        $crate::reflection::RegistryBuilder::new()
            $(.add_type::<$ty>())*
            .build()
    }};
}
