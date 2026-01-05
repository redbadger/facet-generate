use std::io::{Result, Write};

use crate::{
    Registry,
    generation::{
        CodeGen, CodeGeneratorConfig, Emitter,
        config::PackageLocation,
        indent::{IndentConfig, IndentedWriter},
        module::Module,
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for code-generation in Kotlin
pub struct CodeGenerator<'a> {
    /// Language-independent configuration
    pub(crate) config: &'a CodeGeneratorConfig,
}

impl<'a> CodeGen<'a> for CodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        CodeGenerator::new(config)
    }

    fn write_output<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        registry: &Registry,
    ) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> CodeGenerator<'a> {
    /// Create a Kotlin code generator for the given config
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self { config }
    }

    /// Output type definitions for `registry`.
    /// # Errors
    /// This function may fail if the writer encounters an error while writing the generated code.
    pub fn output(&self, out: &mut dyn Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, IndentConfig::Space(4));

        let mut config = self.config.clone();
        config.update_from(registry);

        // Update qualified type names to use fully qualified paths
        let updated_registry = Self::update_qualified_names(&config, registry);

        let module = Module::new(&config);
        module.write(w)?;

        for (i, container) in updated_registry.iter().enumerate() {
            if i > 0 {
                writeln!(w)?;
            }
            (config.encoding, container).write(w)?;
        }

        Ok(())
    }

    /// Updates all `QualifiedTypeName` instances in the registry to use fully qualified paths
    /// based on the provided configuration for external type references.
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            let _ = container_format.visit_mut(&mut |format| {
                if let Format::TypeName(qualified_name) = format {
                    match &qualified_name.namespace {
                        Namespace::Named(namespace) => {
                            let namespace = namespace.clone();
                            // First check if this namespace has an external package configuration with a Path
                            let external_package_handled = if let Some(external_package) =
                                config.external_packages.get(&namespace)
                            {
                                if let PackageLocation::Path(path) = &external_package.location {
                                    let full_namespace = format!("{path}.{namespace}");
                                    *qualified_name = QualifiedTypeName::namespaced(
                                        full_namespace,
                                        qualified_name.name.clone(),
                                    );
                                    true
                                } else {
                                    // PackageLocation::Url is ignored for Kotlin generation - fall through
                                    false
                                }
                            } else {
                                false
                            };

                            if !external_package_handled {
                                // Check if this type's namespace matches the current module's namespace
                                let current_leaf_namespace = config
                                    .module_name()
                                    .rsplit_once('.')
                                    .map_or(config.module_name(), |(_, leaf)| leaf);

                                if config.external_definitions.contains_key(&namespace)
                                    && namespace != current_leaf_namespace
                                {
                                    // For external types, build full path: current_module.namespace
                                    let full_namespace =
                                        format!("{}.{namespace}", config.module_name());
                                    *qualified_name = QualifiedTypeName::namespaced(
                                        full_namespace,
                                        qualified_name.name.clone(),
                                    );
                                } else if namespace == current_leaf_namespace {
                                    // For same-module types, use current module name only
                                    *qualified_name = QualifiedTypeName::namespaced(
                                        config.module_name().to_string(),
                                        qualified_name.name.clone(),
                                    );
                                } else {
                                    // For other local types with named namespace, preserve the namespace
                                    let full_namespace =
                                        format!("{}.{namespace}", config.module_name());
                                    *qualified_name = QualifiedTypeName::namespaced(
                                        full_namespace,
                                        qualified_name.name.clone(),
                                    );
                                }
                            }
                        }
                        Namespace::Root => {
                            // Root namespace types get current module name
                            *qualified_name = QualifiedTypeName::namespaced(
                                config.module_name().to_string(),
                                qualified_name.name.clone(),
                            );
                        }
                    }
                }
                Ok(())
            });
        }

        updated_registry
    }
}

#[cfg(test)]
mod tests;
