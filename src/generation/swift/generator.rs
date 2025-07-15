use std::{
    collections::HashMap,
    io::{Result, Write},
};

use heck::{AsUpperCamelCase, ToUpperCamelCase};

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig,
        indent::{IndentConfig, IndentedWriter},
        swift::emitter::SwiftEmitter,
    },
};

/// Main configuration object for code-generation in Swift.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    pub config: &'a CodeGeneratorConfig,
    /// Mapping from external type names to fully-qualified class names (e.g. "`MyClass`" ->`com.my_org.my_package.MyClass`").
    /// Derived from `config.external_definitions`.
    pub external_qualified_names: HashMap<String, String>,
}

impl<'a> CodeGenerator<'a> {
    /// Create a Swift code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        assert!(
            !config.c_style_enums,
            "Swift does not support generating c-style enums"
        );
        let mut external_qualified_names = HashMap::new();
        for (namespace, names) in &config.external_definitions {
            let module = {
                let path = namespace.rsplitn(2, '/').collect::<Vec<_>>();
                if path.len() <= 1 { namespace } else { path[0] }
            }
            .to_upper_camel_case();
            for name in names {
                external_qualified_names.insert(name.to_string(), format!("{module}.{name}"));
            }
        }
        Self {
            config,
            external_qualified_names,
        }
    }

    /// Output class definitions for `registry`.
    pub fn output(&self, out: &mut dyn Write, registry: &Registry) -> Result<()> {
        let current_namespace = self
            .config
            .module_name
            .split('.')
            .map(AsUpperCamelCase)
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let mut emitter = SwiftEmitter {
            out: IndentedWriter::new(out, IndentConfig::Space(4)),
            generator: self,
            current_namespace,
        };

        emitter.output_preamble()?;

        for (name, format) in registry {
            emitter.output_container(&name.name, format)?;
        }

        if self.config.serialization {
            writeln!(emitter.out)?;
            emitter.output_trait_helpers(registry)?;
        }

        Ok(())
    }
}
