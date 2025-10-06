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
    ($($ty:ident),* as $language:ident with $encoding:path) => {
        || -> anyhow::Result<String> {
            use $crate::generation::{Container, Encoding, WithEncoding, indent::{IndentConfig, IndentedWriter}};
            use std::io::Write as _;
            let mut out = Vec::new();
            let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let registry = $crate::reflect!($($ty),*);
            for (i, (name, format)) in registry.iter().enumerate() {
                if i > 0 {
                    writeln!(&mut w)?;
                }
                let container = WithEncoding {
                    encoding: $encoding,
                    value: Container { name, format },
                };
                <WithEncoding<Container> as Emitter<$language>>::write(&container, &mut w)?;
            }
            let out = String::from_utf8(out)?;

            Ok(out)
        }()
    };
}

#[macro_export]
macro_rules! emit_swift {
    ($($ty:ident),* as $encoding:path) => {
        || -> anyhow::Result<String> {
            use $crate::generation::{Encoding, indent::{IndentConfig, IndentedWriter}};
            let mut out = Vec::new();
            let w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let config = $crate::generation::CodeGeneratorConfig::new("com.example".to_string())
                .with_encoding($encoding);
            let generator = $crate::generation::swift::CodeGenerator::new(&config);
            let mut emitter = $crate::generation::swift::emitter::SwiftEmitter {
                out: w,
                generator: &generator,
                current_namespace: Vec::new(),
            };
            let registry = $crate::reflect!($($ty),*);
            for (name, format) in &registry {
                emitter.output_container(&name.name, format).unwrap();
            }
            let out = String::from_utf8(out)?;

            Ok(out)
        }()
    };
}

#[macro_export]
macro_rules! emit_java {
    ($($ty:ident),* as $encoding:path) => {
        || -> anyhow::Result<String> {
            use $crate::generation::{Encoding, indent::{IndentConfig, IndentedWriter}};
            let mut out = Vec::new();
            let w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let config = $crate::generation::CodeGeneratorConfig::new("com.example".to_string())
                .with_encoding($encoding);
            let generator = $crate::generation::java::CodeGenerator::new(&config);
            let mut emitter = $crate::generation::java::emitter::JavaEmitter {
                out: w,
                generator: &generator,
                current_namespace: Vec::new(),
                current_reserved_names: HashMap::new(),
            };
            let registry = $crate::reflect!($($ty),*);
            for (name, format) in &registry {
                emitter.output_container(&name.name, format).unwrap();
            }
            let out = String::from_utf8(out)?;

            Ok(out)
        }()
    };
}

#[macro_export]
macro_rules! reflect {
    ($($ty:ident),*) => {
        $crate::reflection::RegistryBuilder::new()
            $(.add_type::<$ty>())*
            .build()
    };
}
