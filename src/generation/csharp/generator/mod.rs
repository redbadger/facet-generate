//! Top-level orchestrator for C# code generation.
//!
//! [`CodeGenerator`] implements [`CodeGen`](super::super::CodeGen) to produce a
//! complete C# source file from a [`Registry`](crate::Registry). It resolves
//! qualified type names using the dotted-namespace convention, then delegates
//! AST-to-source rendering to the [emitter](super::emitter) layer.

use std::io::{Result, Write};

use crate::{
    Registry,
    generation::{
        CodeGen, CodeGeneratorConfig, Container, Emitter,
        csharp::emitter::CSharp,
        indent::{IndentConfig, IndentedWriter},
        module::Module,
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for C# code generation.
///
/// Wraps a [`CodeGeneratorConfig`] and implements [`CodeGen`] to provide the
/// entry point for producing C# source from a registry.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
}

impl<'a> CodeGen<'a> for CodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        CodeGenerator::new(config)
    }

    fn write_output<W: Write>(&mut self, writer: &mut W, registry: &Registry) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> CodeGenerator<'a> {
    /// Create a C# code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self { config }
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

        let lang = CSharp::new(config.encoding);

        Module::new(&config).write(w, lang)?;

        let updated_registry = Self::update_qualified_names(&config, registry);
        for (index, container) in updated_registry.iter().map(Container::from).enumerate() {
            if index > 0 {
                writeln!(w)?;
            }
            container.write(w, lang)?;
        }

        Ok(())
    }

    /// Update [`QualifiedTypeName`] instances for C#'s dotted-namespace rules.
    ///
    /// 1. **Same leaf namespace** — a `Named("Users")` reference inside module
    ///    `Company.Models.Users` is stripped to `Root` (bare name).
    /// 2. **External namespace** — a `Named("Payments")` reference inside module
    ///    `Company.Models` becomes `Named("Company.Models.Payments")` (rooted
    ///    under the configured module name).
    /// 3. **Root with dotted module** — a `Root` reference inside module
    ///    `Company.Models` is promoted to `Named("Company.Models")`.
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            let _ = container_format.visit_mut(&mut |format| {
                if let Format::TypeName(qualified_name) = format {
                    match &qualified_name.namespace {
                        Namespace::Named(namespace) => {
                            let namespace = namespace.clone();
                            let current_leaf_namespace = config
                                .module_name()
                                .rsplit_once('.')
                                .map_or(config.module_name(), |(_, leaf)| leaf);

                            if namespace == current_leaf_namespace {
                                *qualified_name =
                                    QualifiedTypeName::root(qualified_name.name.clone());
                            } else {
                                *qualified_name = QualifiedTypeName::namespaced(
                                    format!("{}.{}", config.module_name(), namespace),
                                    qualified_name.name.clone(),
                                );
                            }
                        }
                        Namespace::Root => {
                            if config.module_name().contains('.') {
                                *qualified_name = QualifiedTypeName::namespaced(
                                    config.module_name().to_string(),
                                    qualified_name.name.clone(),
                                );
                            }
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
