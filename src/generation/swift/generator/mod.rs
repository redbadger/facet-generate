use std::io::{Result, Write};

use crate::{
    Registry,
    generation::{
        CodeGen, CodeGeneratorConfig, Container, Emitter,
        indent::{IndentConfig, IndentedWriter},
        module::Module,
        swift::emitter::Swift,
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for code-generation in Swift.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
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
    /// Create a Swift code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self { config }
    }

    /// Output class definitions for `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, IndentConfig::Space(4));

        let mut config = self.config.clone();
        config.update_from(registry);

        let lang = Swift::new(config.encoding);

        Module::new(&config).write(w, lang)?;

        let updated_registry = Self::update_qualified_names(&config, registry);
        for container in updated_registry.iter().map(Container::from) {
            writeln!(w)?;
            container.write(w, lang)?;
        }

        Ok(())
    }

    /// Updates `QualifiedTypeName` instances so external types include their namespace prefix.
    /// For example, a type `Tree` in namespace `foo` becomes `Foo.Tree` in the output.
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            let _ = container_format.visit_mut(&mut |format| {
                if let Format::TypeName(qualified_name) = format {
                    if let Namespace::Named(namespace) = &qualified_name.namespace {
                        if config.external_definitions.contains_key(namespace) {
                            *qualified_name = QualifiedTypeName::namespaced(
                                namespace.clone(),
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
