//! Top-level orchestrator for Kotlin code generation.
//!
//! [`KotlinCodeGenerator`] implements [`CodeGenerator`] and is the entry point for
//! producing a single Kotlin source file from a [`Registry`].

use std::{
    io::{Result, Write},
    sync::Arc,
};

use crate::{
    Registry,
    generation::{
        CodeGenerator, CodeGeneratorConfig, Container, Emitter,
        config::PackageLocation,
        indent::IndentedWriter,
        kotlin::{
            emitter::{Kotlin, write_module_header},
            naming,
        },
        module::Module,
        naming::check_reserved_names,
        plugin::{CompanionFile, EmitterPlugin, render_companion_files},
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Kotlin code generator — holds a reference to the shared
/// [`CodeGeneratorConfig`] and implements [`CodeGenerator`].
pub struct KotlinCodeGenerator<'a> {
    /// Language-independent configuration (module name, external packages, etc.).
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Plugins applied during code generation.
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Kotlin>>>,
}

impl<'a> CodeGenerator<'a> for KotlinCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        KotlinCodeGenerator::new(config)
    }

    fn write_output<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        registry: &Registry,
    ) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> KotlinCodeGenerator<'a> {
    /// Create a Kotlin code generator for the given config with no plugins
    /// (plain type declarations only, no serialize/deserialize methods).
    ///
    /// Call [`with_plugins`](Self::with_plugins) to enable serialization.
    #[must_use]
    pub const fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            plugins: vec![],
        }
    }

    /// Set pre-built plugins, returning the modified generator.
    #[must_use]
    pub fn with_plugins(mut self, plugins: Vec<Arc<dyn EmitterPlugin<Kotlin>>>) -> Self {
        self.plugins = plugins;
        self
    }

    /// Produce a complete Kotlin source file for the given `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying writer fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, self.config.indent);

        let mut config = self.config.clone();
        config.update_from(registry);
        config.requalify_enums(registry, Self::requalify);
        check_reserved_names(registry, &naming::RULES)?;

        let mut lang = Kotlin::new(&config, registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

        Module::new(&config).write(w, &lang)?;

        for (i, container) in Self::update_qualified_names(&config, registry)
            .iter()
            .map(Container::from)
            .enumerate()
        {
            if i > 0 {
                writeln!(w)?;
            }
            container.write(w, &lang)?;
        }
        Ok(())
    }

    /// Render the companion files contributed by the plugins for `registry`.
    ///
    /// Each returned [`CompanionFile`] carries the file's final contents: the
    /// same module header [`output`](Self::output) writes (minus the module
    /// helpers), merged with the file's own imports, followed by the plugin's
    /// body. The installer writes them into the module's package directory.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering a header fails.
    pub fn companion_files(&self, registry: &Registry) -> Result<Vec<CompanionFile>> {
        let mut config = self.config.clone();
        config.update_from(registry);

        let mut lang = Kotlin::new(&config, registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

        render_companion_files(lang.plugins(), &config, |imports| {
            let mut header = Vec::new();
            write_module_header(
                &mut IndentedWriter::new(&mut header, config.indent),
                &config,
                &lang,
                imports,
            )?;
            String::from_utf8(header).map_err(std::io::Error::other)
        })
    }

    /// Rewrites every [`QualifiedTypeName`] in the registry to a fully-qualified
    /// Kotlin package path, returning a new registry.
    ///
    /// The registry coming in from reflection uses short, language-agnostic
    /// names like `Namespace::Named("Other") + "MyType"`. Kotlin needs these
    /// turned into dot-separated package paths so the emitter can write e.g.
    /// `com.example.other.Other.MyType` in type references and deserialize
    /// calls.
    ///
    /// # Resolution rules (checked in this order)
    ///
    /// 1. **External package with a [`PackageLocation::Path`]** — the
    ///    configured path replaces the namespace prefix.
    ///    `Other::MyType` with path `com.acme.other` → `com.acme.other.Other.MyType`
    ///
    /// 2. **External definition in a different namespace** — prefixed with the
    ///    *root package* (the parent this module was nested under, or the module
    ///    itself when it has no parent), because sibling namespaces are peers
    ///    rather than children.
    ///    Module `com.example.main`, namespace `auth` → `com.example.main.auth.User`
    ///    Module `com.example.main.orders` (parent `com.example.main`), namespace
    ///    `auth` → `com.example.main.auth.User` — *not* `…main.orders.auth.User`
    ///
    /// 3. **Same namespace as the current module** — collapsed to just the
    ///    module name (no double-nesting).
    ///    Module `com.example.other`, namespace `other` → `com.example.other.LocalType`
    ///
    /// 4. **[`Namespace::Root`]** — uses the *root package*, for the same
    ///    reason as rule 2.
    ///    Module `com.example.service` → `com.example.service.RootType`
    ///    Module `com.example.kv` (parent `com.example`) →
    ///    `com.example.RootType` — *not* `com.example.kv.RootType`
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
                // First check if this namespace has an external package configuration with a Path
                if let Some(external_package) = config.external_packages.get(namespace)
                    && let PackageLocation::Path(path) = &external_package.location
                {
                    return QualifiedTypeName::namespaced(
                        format!("{path}.{namespace}"),
                        name.name.clone(),
                    );
                }
                // PackageLocation::Url is ignored for Kotlin generation - fall through

                // Check if this type's namespace matches the current module's namespace
                let current_leaf_namespace = config
                    .module_name()
                    .rsplit_once('.')
                    .map_or_else(|| config.module_name(), |(_, leaf)| leaf);

                if config.external_definitions.contains_key(namespace)
                    && namespace != current_leaf_namespace
                {
                    // A sibling namespace, so the path is rooted at the
                    // parent package — NOT at `module_name()`, which already
                    // ends in *this* module's namespace and would yield
                    // `com.example.main.Directory.Kit.Row`.
                    QualifiedTypeName::namespaced(
                        format!("{}.{namespace}", config.root_package()),
                        name.name.clone(),
                    )
                } else if namespace == current_leaf_namespace {
                    // For same-module types, use current module name only
                    QualifiedTypeName::namespaced(
                        config.module_name().to_string(),
                        name.name.clone(),
                    )
                } else {
                    // Same reasoning as the external case above: a named
                    // namespace that is not our own hangs off the parent.
                    QualifiedTypeName::namespaced(
                        format!("{}.{namespace}", config.root_package()),
                        name.name.clone(),
                    )
                }
            }
            Namespace::Root => {
                // Root types live in the root package, which is not
                // `module_name()` for a namespaced module (#148)
                QualifiedTypeName::namespaced(config.root_package().to_string(), name.name.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests;
