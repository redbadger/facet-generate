use std::{collections::HashMap, io::Write};

use heck::ToUpperCamelCase as _;

use crate::{
    Registry,
    generation::{
        CodeGen, CodeGeneratorConfig,
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
    /// Whether to generate extensionless imports (for React/Node.js compatibility)
    pub(crate) target: InstallTarget,
}

impl<'a> CodeGen<'a> for CodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        CodeGenerator::new(config, InstallTarget::Node)
    }

    fn write_output<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        registry: &Registry,
    ) -> std::io::Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> CodeGenerator<'a> {
    /// Create a TypeScript code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig, target: InstallTarget) -> Self {
        let mut external_qualified_names = HashMap::new();
        for (namespace, names) in &config.external_definitions {
            for name in names {
                external_qualified_names.insert(
                    name.clone(),
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
                    // For local packages, use path
                    path.clone()
                }
            };
            external_import_paths.insert(namespace.to_lowercase(), import_path);
        }

        Self {
            config,
            external_qualified_names,
            target,
        }
    }

    /// Output class definitions for `registry` in a single source file.
    pub fn output<B: Write>(&self, out: &mut B, registry: &Registry) -> std::io::Result<()> {
        let mut emitter = TypeScriptEmitter::new(self);

        let mut body = Vec::new();

        let mut writer = IndentedWriter::new(&mut body, IndentConfig::Space(2));
        for (name, format) in registry {
            emitter.output_container(&mut writer, &name.name, format)?;
        }

        if self.config.has_encoding() {
            emitter.output_helpers(&mut writer, registry)?;
        }

        let mut preamble = Vec::new();

        let mut writer = IndentedWriter::new(&mut preamble, IndentConfig::Space(2));
        emitter.output_preamble(&mut writer)?;

        out.write_all(&preamble)?;
        out.write_all(&body)?;

        Ok(())
    }
}
