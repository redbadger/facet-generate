use std::io::{Result, Write};

use crate::{
    Registry,
    generation::{
        CodeGen, CodeGeneratorConfig, Container, Emitter,
        indent::{IndentConfig, IndentedWriter},
        module::Module,
        typescript::{InstallTarget, emitter::TypeScript},
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for code-generation in TypeScript.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Installation target (Node or Deno).
    pub(crate) target: InstallTarget,
}

impl<'a> CodeGen<'a> for CodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        CodeGenerator::new(config, InstallTarget::Node)
    }

    fn write_output<W: Write>(&mut self, writer: &mut W, registry: &Registry) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> CodeGenerator<'a> {
    /// Create a TypeScript code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig, target: InstallTarget) -> Self {
        Self { config, target }
    }

    /// Output type definitions for `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, IndentConfig::Space(4));

        let mut config = self.config.clone();
        config.update_from(registry);

        let lang = TypeScript::new(config.encoding).with_target(self.target);

        Module::new(&config).write(w, lang)?;

        let updated_registry = Self::update_qualified_names(&config, registry);
        for container in updated_registry.iter().map(Container::from) {
            container.write(w, lang)?;
        }

        Ok(())
    }

    /// Updates `QualifiedTypeName` instances so external types include their namespace prefix
    /// and same-module types are stripped to root (bare name).
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            let _ = container_format.visit_mut(&mut |format| {
                if let Format::TypeName(qualified_name) = format
                    && let Namespace::Named(namespace) = &qualified_name.namespace
                    && namespace == config.module_name()
                {
                    // Same-module type: strip namespace so it renders as a bare name
                    *qualified_name = QualifiedTypeName::root(qualified_name.name.clone());
                }
                Ok(())
            });
        }

        updated_registry
    }
}

#[cfg(test)]
mod tests;
