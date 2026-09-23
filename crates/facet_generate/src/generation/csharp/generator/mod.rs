//! Top-level orchestrator for C# code generation.
//!
//! [`CSharpCodeGenerator`] implements [`CodeGenerator`](super::super::CodeGenerator) to produce a
//! complete C# source file from a [`Registry`](crate::Registry). It resolves
//! qualified type names using the dotted-namespace convention, then delegates
//! AST-to-source rendering to the [emitter](super::emitter) layer.

use std::io::{Result, Write};

use std::sync::Arc;

use crate::{
    Registry,
    generation::{
        CodeGenerator, CodeGeneratorConfig, Container, Emitter, csharp::emitter::CSharp,
        indent::IndentedWriter, module::Module, other_modules::OtherModules, plugin::EmitterPlugin,
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for C# code generation.
///
/// Wraps a [`CodeGeneratorConfig`] and implements [`CodeGenerator`] to provide the
/// entry point for producing C# source from a registry.
pub struct CSharpCodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Pre-built plugins to apply during code generation.
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<CSharp>>>,
    /// The whole registry the module was split from, when the installer
    /// generates it, so that references to types in other modules can be
    /// serialized according to their kind (see [`OtherModules`]).
    pub(crate) whole_registry: Option<&'a Registry>,
}

impl<'a> CodeGenerator<'a> for CSharpCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            plugins: vec![],
            whole_registry: None,
        }
    }

    fn write_output<W: Write>(&mut self, writer: &mut W, registry: &Registry) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> CSharpCodeGenerator<'a> {
    /// Create a C# code generator with no plugins.
    ///
    /// Call [`with_plugins`](Self::with_plugins) to add serialization plugins.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            plugins: vec![],
            whole_registry: None,
        }
    }

    /// Set pre-built plugins, returning the modified generator.
    #[must_use]
    pub fn with_plugins(mut self, plugins: Vec<Arc<dyn EmitterPlugin<CSharp>>>) -> Self {
        self.plugins = plugins;
        self
    }

    /// Tell the generator the whole registry that the module it generates
    /// was split from, if there is one.
    #[must_use]
    pub(crate) const fn with_whole_registry(
        mut self,
        whole_registry: Option<&'a Registry>,
    ) -> Self {
        self.whole_registry = whole_registry;
        self
    }

    /// Output type definitions for `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, self.config.indent);

        let mut config = self.config.clone();
        config.update_from(registry);

        let updated_registry = Self::update_qualified_names(&config, registry);
        let mut lang = CSharp::new(&config, &updated_registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

        Module::new(&config).write(w, &lang)?;

        let other_modules = self
            .whole_registry
            .map(|whole| OtherModules::new(whole, registry, |name| Self::requalify(&config, name)));
        OtherModules::scope(other_modules, || {
            for (index, container) in updated_registry.iter().map(Container::from).enumerate() {
                if index > 0 {
                    writeln!(w)?;
                }
                container.write(w, &lang)?;
            }
            Ok(())
        })
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
                    *qualified_name = Self::requalify(config, qualified_name);
                }
                Ok(())
            });
        }

        updated_registry
    }

    /// The spelling [`update_qualified_names`](Self::update_qualified_names)
    /// gives a reference to `name`.
    fn requalify(config: &CodeGeneratorConfig, name: &QualifiedTypeName) -> QualifiedTypeName {
        match &name.namespace {
            Namespace::Named(namespace) => {
                let current_leaf_namespace = config
                    .module_name()
                    .rsplit_once('.')
                    .map_or_else(|| config.module_name(), |(_, leaf)| leaf);

                if namespace == current_leaf_namespace {
                    QualifiedTypeName::root(name.name.clone())
                } else {
                    QualifiedTypeName::namespaced(
                        format!("{}.{}", config.module_name(), namespace),
                        name.name.clone(),
                    )
                }
            }
            Namespace::Root => {
                if config.module_name().contains('.') {
                    QualifiedTypeName::namespaced(
                        config.module_name().to_string(),
                        name.name.clone(),
                    )
                } else {
                    name.clone()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
