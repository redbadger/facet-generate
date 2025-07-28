use std::collections::HashMap;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig,
        indent::{IndentConfig, IndentedWriter},
        java::emitter::JavaEmitter,
    },
    reflection::format::ContainerFormat,
};

/// Main configuration object for code-generation in Java.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Mapping from external type names to fully-qualified class names (e.g. "`MyClass`" -> "`com.my_org.my_package.MyClass`").
    /// Derived from `config.external_definitions`.
    pub(crate) external_qualified_names: HashMap<String, String>,
}

impl<'a> CodeGenerator<'a> {
    /// Create a Java code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        assert!(
            !config.c_style_enums,
            "Java does not support generating c-style enums"
        );
        let mut external_qualified_names = HashMap::new();
        let root_module = config.module_name();
        for (namespace, names) in &config.external_definitions {
            for name in names {
                external_qualified_names.insert(
                    name.to_string(),
                    format!("{root_module}.{namespace}.{name}"),
                );
            }
        }
        Self {
            config,
            external_qualified_names,
        }
    }

    /// Output class definitions for ` registry` in separate source files.
    /// Source files will be created in a subdirectory of `install_dir` corresponding to the given
    /// package name (if any, otherwise `install_dir` it self).
    pub fn write_source_files(
        &self,
        install_dir: std::path::PathBuf,
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
        if self.config.serialization.is_enabled() {
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
            out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
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
            out: IndentedWriter::new(&mut file, IndentConfig::Space(4)),
            generator: self,
            current_namespace,
            current_reserved_names: HashMap::new(),
        };

        emitter.output_preamble()?;
        emitter.output_trait_helpers(registry)
    }
}
