use std::{collections::HashMap, io::Write};

use heck::ToUpperCamelCase as _;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig,
        indent::{IndentConfig, IndentedWriter},
        typescript::{InstallTarget, emitter::TypeScriptEmitter},
    },
};

/// Main configuration object for code-generation in TypeScript, powered by
/// the Deno runtime.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Mapping from external type names to fully-qualified class names (e.g. "`MyClass`" -> "`com.my_org.my_package.MyClass`").
    /// Derived from `config.external_definitions`.
    pub(crate) external_qualified_names: HashMap<String, String>,
    /// vector of namespaces to import
    pub(crate) namespaces_to_import: Vec<String>,
    /// Mapping from namespace to import path for external packages
    pub(crate) external_import_paths: HashMap<String, String>,
    /// Whether to generate extensionless imports (for React/Node.js compatibility)
    pub(crate) target: InstallTarget,
}

impl<'a> CodeGenerator<'a> {
    /// Create a TypeScript code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig, target: InstallTarget) -> Self {
        assert!(
            !config.c_style_enums,
            "TypeScript does not support generating c-style enums"
        );
        let mut external_qualified_names = HashMap::new();
        for (namespace, names) in &config.external_definitions {
            for name in names {
                external_qualified_names.insert(
                    name.to_string(),
                    format!("{}.{}", namespace.to_upper_camel_case(), name),
                );
            }
        }
        let mut external_import_paths = HashMap::new();
        for (namespace, external_package) in &config.external_packages {
            let import_path = match &external_package.location {
                crate::generation::PackageLocation::Url(url) => {
                    // Extract package name from URL for npm packages
                    if let Some(package_name) = url.split('/').next_back() {
                        package_name.to_string()
                    } else {
                        namespace.clone()
                    }
                }
                crate::generation::PackageLocation::Path(path) => {
                    // For local packages, use relative path
                    format!("./{}", path.trim_start_matches("../"))
                }
            };
            external_import_paths.insert(namespace.clone(), import_path);
        }

        Self {
            config,
            external_qualified_names,
            namespaces_to_import: config
                .external_definitions
                .keys()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>(),
            external_import_paths,
            target,
        }
    }

    /// Output class definitions for `registry` in a single source file.
    pub fn output(&self, out: &mut dyn Write, registry: &Registry) -> std::io::Result<()> {
        let mut emitter = TypeScriptEmitter {
            out: IndentedWriter::new(out, IndentConfig::Space(2)),
            generator: self,
        };

        emitter.output_preamble()?;

        for (name, format) in registry {
            emitter.output_container(&name.name, format)?;
        }

        if self.config.serialization {
            emitter.output_helpers(registry)?;
        }

        Ok(())
    }
}
