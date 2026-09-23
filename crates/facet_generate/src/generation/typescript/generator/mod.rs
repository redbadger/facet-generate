//! Top-level orchestrator for TypeScript code generation.
//!
//! [`TypeScriptCodeGenerator`] implements [`CodeGenerator`] and is the entry point for
//! producing a single TypeScript source file from a [`Registry`]. It carries
//! It delegates writing to the emitter layer.

use std::{
    io::{Result, Write},
    sync::Arc,
};

use crate::{
    Registry,
    generation::{
        CodeGenerator, CodeGeneratorConfig, Container, Emitter, indent::IndentedWriter,
        module::Module, other_modules::OtherModules, plugin::EmitterPlugin,
        typescript::emitter::TypeScript,
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for TypeScript code generation.
///
/// Wraps a [`CodeGeneratorConfig`] and implements [`CodeGenerator`] so it
/// can be used by the installer pipeline.
pub struct TypeScriptCodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Plugins that control encoding-specific code generation.
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<TypeScript>>>,
    /// The whole registry the module was split from, when the installer
    /// generates it, so that references to types in other modules can be
    /// serialized according to their kind (see [`OtherModules`]).
    pub(crate) whole_registry: Option<&'a Registry>,
}

impl<'a> CodeGenerator<'a> for TypeScriptCodeGenerator<'a> {
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

impl<'a> TypeScriptCodeGenerator<'a> {
    /// Create a TypeScript code generator with no plugins (plain types only).
    ///
    /// Call [`with_plugins`](Self::with_plugins) to enable serialization.
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
    pub fn with_plugins(mut self, plugins: Vec<Arc<dyn EmitterPlugin<TypeScript>>>) -> Self {
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

    /// Produce a complete TypeScript source file for the types in `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, self.config.indent);

        let mut config = self.config.clone();
        config.update_from(registry);

        let mut lang = TypeScript::new(&config, registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

        Module::new(&config).write(w, &lang)?;

        let updated_registry = Self::update_qualified_names(&config, registry);
        let other_modules = self
            .whole_registry
            .map(|whole| OtherModules::new(whole, registry, |name| Self::requalify(&config, name)));
        OtherModules::scope(other_modules, || {
            for container in updated_registry.iter().map(Container::from) {
                container.write(w, &lang)?;
            }
            Ok(())
        })
    }

    /// Updates [`QualifiedTypeName`] instances for TypeScript's ES-module
    /// namespacing:
    ///
    /// 1. **Same-module type** — strip namespace to `Root` so it renders as a
    ///    bare name (e.g. `Child`).
    /// 2. **External type in different namespace** — keep its `Named` namespace,
    ///    which renders as `Namespace.Type` (e.g. `Other.Child`) via the
    ///    wildcard import added by the [`Module`](super::super::module::Module)
    ///    emitter.
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
            // Same-module type: strip namespace so it renders as a bare name
            Namespace::Named(namespace) if namespace == config.module_name() => {
                QualifiedTypeName::root(name.name.clone())
            }
            _ => name.clone(),
        }
    }
}

#[cfg(test)]
mod tests;
