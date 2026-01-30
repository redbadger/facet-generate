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
            let registry = $crate::reflect!($($ty),*)?;
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
macro_rules! emit_swift {
    ($($ty:ident),* as $encoding:path) => {
        || -> anyhow::Result<String> {
            use $crate::generation::indent::{IndentConfig, IndentedWriter};
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
            let registry = $crate::reflect!($($ty),*)?;
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
            use $crate::generation::indent::{IndentConfig, IndentedWriter};
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
            let registry = $crate::reflect!($($ty),*)?;
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
        || -> anyhow::Result<std::collections::BTreeMap<$crate::reflection::format::QualifiedTypeName, $crate::reflection::format::ContainerFormat>> {
            let registry = $crate::reflection::RegistryBuilder::new()
                $(.add_type::<$ty>().map_err(|e| anyhow::anyhow!("failed to add type {}: {}", stringify!($ty), e))?)*
                .build()
                .map_err(|e| anyhow::anyhow!("failed to build registry: {e}"))?;
            Ok(registry)
        }()
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! emit_two_modules {
    ($generator:ty, $facet:ident, $root:expr) => {{
        use $crate::generation::CodeGen;
        use $crate::generation::module::{self, Module};
        use $crate::{Registry, reflect};

        fn emit_module<'a, G: CodeGen<'a>>(module: &'a Module, registry: &Registry) -> String {
            let mut out = Vec::new();
            let mut generator = G::new(module.config());
            generator.write_output(&mut out, registry).unwrap();
            String::from_utf8(out).unwrap()
        }

        let registry = reflect!($facet).unwrap();
        let mut modules: Vec<_> = module::split($root, &registry).into_iter().collect();
        modules.sort_by(|a, b| a.0.config().module_name.cmp(&b.0.config().module_name));

        let modules: [(Module, Registry); 2] = modules.try_into().expect("Two modules expected");
        let [(other_module, other_registry), (root_module, root_registry)] = modules;

        let module_1 = emit_module::<$generator>(&other_module, &other_registry);
        let module_2 = emit_module::<$generator>(&root_module, &root_registry);
        (module_1, module_2)
    }};
}
