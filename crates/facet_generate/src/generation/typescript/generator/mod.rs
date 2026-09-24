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
        CodeGenerator, CodeGeneratorConfig, Container, Emitter,
        indent::IndentedWriter,
        module::Module,
        naming::check_reserved_names,
        plugin::{self, EmitterPlugin},
        registry_references,
        typescript::{emitter::TypeScript, naming},
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
}

impl<'a> CodeGenerator<'a> for TypeScriptCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            plugins: vec![],
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
        }
    }

    /// Set pre-built plugins, returning the modified generator.
    #[must_use]
    pub fn with_plugins(mut self, plugins: Vec<Arc<dyn EmitterPlugin<TypeScript>>>) -> Self {
        self.plugins = plugins;
        self
    }

    /// Produce a complete TypeScript source file for the types in `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails, or if a plugin declares a
    /// reference to a type that is not in the registry.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, self.config.indent);

        let mut config = self.config.clone();
        config.update_from(registry);
        config.requalify_enums(registry, Self::requalify);
        Self::reference_requalified_namespaces(&mut config, registry_references(registry).iter());
        let plugin_references = plugin::referenced_types(&self.plugins, &config)?;
        config.reference_types(&plugin_references);
        Self::reference_requalified_namespaces(&mut config, plugin_references.iter());
        check_reserved_names(registry, &naming::RULES)?;

        let mut lang = TypeScript::new(&config, registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

        Module::new(&config).write(w, &lang)?;

        let updated_registry = Self::update_qualified_names(&config, registry);
        for container in updated_registry.iter().map(Container::from) {
            container.write(w, &lang)?;
        }

        Ok(())
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
    /// 3. **Root type seen from a namespaced module** — qualify it with the
    ///    root package (e.g. `Example.Shared`), which lives in a module of its
    ///    own. Only when the config knows the root package
    ///    ([`CodeGeneratorConfig::parent`], which the installer sets);
    ///    otherwise, and in the root module itself, it stays bare.
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            Self::requalify_type_names(config, container_format);
        }

        updated_registry
    }

    /// Rewrites every type reference in `holder` with
    /// [`requalify`](Self::requalify), exported to plugins for a [`Format`] as
    /// [`typescript::requalify_format`](crate::generation::typescript::requalify_format).
    pub(crate) fn requalify_type_names(
        config: &CodeGeneratorConfig,
        holder: &mut impl FormatHolder,
    ) {
        let _ = holder.visit_mut(&mut |format| {
            if let Format::TypeName(qualified_name) = format {
                *qualified_name = Self::requalify(config, qualified_name);
            }
            Ok(())
        });
    }

    /// The spelling [`update_qualified_names`](Self::update_qualified_names)
    /// gives a reference to `name`, exported to plugins as
    /// [`typescript::requalify`](crate::generation::typescript::requalify).
    pub(crate) fn requalify(
        config: &CodeGeneratorConfig,
        name: &QualifiedTypeName,
    ) -> QualifiedTypeName {
        match &name.namespace {
            // Same-module type: strip namespace so it renders as a bare name
            Namespace::Named(namespace) if namespace == config.module_name() => {
                QualifiedTypeName::root(name.name.clone())
            }
            // Root type from a namespaced module: reach it through the root
            // module's namespace import
            Namespace::Root if config.root_package() != config.module_name() => {
                QualifiedTypeName::namespaced(config.root_package().to_string(), name.name.clone())
            }
            _ => name.clone(),
        }
    }

    /// Adds the namespace of every reference in `names` that
    /// [`requalify`](Self::requalify) points into another module to
    /// [`referenced_namespaces`](CodeGeneratorConfig::referenced_namespaces),
    /// so that the module imports it. `update_from` (for the registry's
    /// references) and
    /// [`reference_types`](CodeGeneratorConfig::reference_types) (for the
    /// plugins') have already recorded the references that name their
    /// namespace; this adds the root package for the root types a namespaced
    /// module references.
    fn reference_requalified_namespaces<'n>(
        config: &mut CodeGeneratorConfig,
        names: impl Iterator<Item = &'n QualifiedTypeName>,
    ) {
        let namespaces: Vec<String> = names
            .filter_map(|name| match Self::requalify(config, name).namespace {
                Namespace::Named(namespace) if namespace != config.module_name() => Some(namespace),
                _ => None,
            })
            .collect();
        config.referenced_namespaces.extend(namespaces);
    }
}

#[cfg(test)]
mod tests;
