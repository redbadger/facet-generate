use std::{collections::HashMap, path::PathBuf};

use crate::{
    Registry,
    generation::{
        CodeGenerator, CodeGeneratorConfig, Encoding, indent::IndentedWriter,
        java::emitter::JavaEmitter,
    },
    reflection::format::ContainerFormat,
};

/// Main configuration object for code-generation in Java.
#[deprecated(
    since = "0.16.0",
    note = "The Java generator is deprecated. Use the Kotlin generator instead."
)]
pub struct JavaCodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Which serialization encoding to generate code for.
    pub(crate) encoding: Encoding,
    /// Mapping from external type names to fully-qualified class names (e.g. "`MyClass`" -> "`com.my_org.my_package.MyClass`").
    /// Derived from `config.external_definitions`.
    pub(crate) external_qualified_names: HashMap<String, String>,
}

impl<'a> CodeGenerator<'a> for JavaCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        JavaCodeGenerator::new(config)
    }

    fn write_output<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        registry: &Registry,
    ) -> std::io::Result<()> {
        let current_namespace = self
            .config
            .module_name
            .split('.')
            .map(String::from)
            .collect::<Vec<_>>();

        let mut emitter = JavaEmitter {
            out: IndentedWriter::new(writer, self.config.indent),
            generator: self,
            current_namespace,
            current_reserved_names: HashMap::new(),
        };

        emitter.output_preamble()?;

        for (name, format) in registry {
            emitter.output_container(&name.name, format)?;
        }

        if !self.encoding.is_none() {
            emitter.output_trait_helpers(registry)?;
        }

        Ok(())
    }
}

impl<'a> JavaCodeGenerator<'a> {
    /// Create a Java code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        let mut external_qualified_names = HashMap::new();
        for (namespace, names) in &config.external_definitions {
            // Get the external package for this namespace to determine the correct package location
            let package_name = config.external_packages.get(namespace).map_or_else(
                || format!("{}.{}", config.module_name(), namespace),
                |external_package| match &external_package.location {
                    crate::generation::PackageLocation::Path(path) => path.clone(),
                    crate::generation::PackageLocation::Url(_) => {
                        // Fallback to using the current module name + namespace for URLs
                        format!("{}.{}", config.module_name(), namespace)
                    }
                },
            );

            for name in names {
                external_qualified_names.insert(name.clone(), format!("{package_name}.{name}"));
            }
        }
        Self {
            config,
            encoding: Encoding::None,
            external_qualified_names,
        }
    }

    /// Set the encoding, returning the modified generator.
    #[must_use]
    pub const fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Output class definitions for ` registry` in separate source files.
    /// Source files will be created in a subdirectory of `install_dir` corresponding to the given
    /// package name (if any, otherwise `install_dir` it self).
    pub fn write_source_files(
        &self,
        install_dir: PathBuf,
        registry: &Registry,
    ) -> std::io::Result<()> {
        let current_namespace = self
            .config
            .module_name
            .split('.')
            .map(String::from)
            .collect::<Vec<_>>();

        let mut dir_path = install_dir;
        for part in &current_namespace {
            dir_path = dir_path.join(part);
        }
        std::fs::create_dir_all(&dir_path)?;

        for (name, format) in registry {
            self.write_container_class(&dir_path, current_namespace.clone(), &name.name, format)?;
        }

        if !self.encoding.is_none() {
            self.write_helper_class(&dir_path, current_namespace, registry)?;
        }

        Ok(())
    }

    fn write_container_class(
        &self,
        dir_path: &std::path::Path,
        current_namespace: Vec<String>,
        name: &str,
        format: &ContainerFormat,
    ) -> std::io::Result<()> {
        let mut file = std::fs::File::create(dir_path.join(name.to_string() + ".java"))?;
        let mut emitter = JavaEmitter {
            out: IndentedWriter::new(&mut file, self.config.indent),
            generator: self,
            current_namespace,
            current_reserved_names: HashMap::new(),
        };

        emitter.output_preamble()?;
        emitter.output_container(name, format)
    }

    fn write_helper_class(
        &self,
        dir_path: &std::path::Path,
        current_namespace: Vec<String>,
        registry: &Registry,
    ) -> std::io::Result<()> {
        let mut file = std::fs::File::create(dir_path.join("TraitHelpers.java"))?;
        let mut emitter = JavaEmitter {
            out: IndentedWriter::new(&mut file, self.config.indent),
            generator: self,
            current_namespace,
            current_reserved_names: HashMap::new(),
        };

        emitter.output_preamble()?;
        emitter.output_trait_helpers(registry)
    }
}
