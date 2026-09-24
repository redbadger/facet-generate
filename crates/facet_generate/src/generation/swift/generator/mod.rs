//! Top-level orchestrator for Swift code generation.
//!
//! [`SwiftCodeGenerator`] implements [`CodeGenerator`] and is the entry point for
//! producing a single Swift source file from a [`Registry`].

use std::{
    collections::BTreeSet,
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
        plugin::{self, CompanionFile, EmitterPlugin, render_companion_files},
        registry_references,
        swift::{
            conformance::Conformance,
            emitter::{Swift, write_module_header},
            naming,
        },
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for Swift code generation.
///
/// Wraps a [`CodeGeneratorConfig`] and implements [`CodeGenerator`] so it can be
/// used by the installer pipeline.
pub struct SwiftCodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Pre-built plugins supplied by the caller (e.g. from the installer).
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Swift>>>,
    /// Which types conform to `Hashable` and `Equatable`, decided over the
    /// whole registry, when the installer supplies it. Without it, they are
    /// decided over the registry the generator is given, and every type from
    /// elsewhere is assumed to conform.
    pub(crate) conformance: Option<Arc<Conformance>>,
}

impl<'a> CodeGenerator<'a> for SwiftCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            plugins: vec![],
            conformance: None,
        }
    }

    fn write_output<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        registry: &Registry,
    ) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> SwiftCodeGenerator<'a> {
    /// Create a Swift code generator with no encoding (plain types only).
    ///
    /// Call [`with_plugins`](Self::with_plugins) to enable serialization.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            plugins: vec![],
            conformance: None,
        }
    }

    /// Set the pre-built plugin list, returning the modified generator.
    #[must_use]
    pub fn with_plugins(mut self, plugins: Vec<Arc<dyn EmitterPlugin<Swift>>>) -> Self {
        self.plugins = plugins;
        self
    }

    /// Set the conformance, decided over the whole registry, that the module
    /// is written with, returning the modified generator.
    #[must_use]
    pub(crate) fn with_conformance(mut self, conformance: Arc<Conformance>) -> Self {
        self.conformance = Some(conformance);
        self
    }

    /// The language tag the module for `registry` is written with, carrying
    /// this generator's plugins.
    fn lang(&self, config: &CodeGeneratorConfig, registry: &Registry) -> Swift {
        let mut lang = match &self.conformance {
            Some(conformance) => Swift::decided_by(config, conformance),
            None => Swift::new(config, registry),
        };
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }
        lang
    }

    /// Produce a complete Swift source file for `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails, or if a plugin declares a
    /// reference to a type that is not in the registry.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let config = self.module_config(registry)?;
        self.write_module(out, &config, registry)
    }

    /// Render the companion files contributed by the plugins for `registry`.
    ///
    /// Each returned [`CompanionFile`] carries the file's final contents: the
    /// same module header [`output`](Self::output) writes (minus the module
    /// helpers), merged with the file's own imports, followed by the plugin's
    /// body. The installer writes them next to the module's source file.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering a header fails, or if a plugin declares a
    /// reference to a type that is not in the registry.
    pub fn companion_files(&self, registry: &Registry) -> Result<Vec<CompanionFile>> {
        let config = self.module_config(registry)?;
        self.render_companion_files(&config, registry)
    }

    /// The config the module for `registry` is written with: this
    /// generator's, updated from the registry, with every type reference the
    /// registry and the plugins make recorded in
    /// [`external_definitions`](CodeGeneratorConfig::external_definitions),
    /// which decide the module's imports and its target's dependencies.
    ///
    /// Asks each plugin for its
    /// [`referenced_types`](EmitterPlugin::referenced_types), so the installer
    /// computes this once per module and hands it to
    /// [`write_module`](Self::write_module) and
    /// [`render_companion_files`](Self::render_companion_files).
    ///
    /// # Errors
    ///
    /// Returns an error if a plugin declares a reference to a type that is not
    /// in the registry.
    pub(crate) fn module_config(&self, registry: &Registry) -> Result<CodeGeneratorConfig> {
        let mut config = self.config.clone();
        config.update_from(registry);
        config.requalify_enums(registry, Self::requalify);
        Self::reference_root_types(&mut config, &registry_references(registry));

        let plugin_references = plugin::referenced_types(&self.plugins, &config)?;
        config.reference_types(&plugin_references);
        Self::reference_root_types(&mut config, &plugin_references);

        Ok(config)
    }

    /// Write the module for `registry` with `config`, as returned by
    /// [`module_config`](Self::module_config).
    pub(crate) fn write_module(
        &self,
        out: &mut impl Write,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> Result<()> {
        let w = &mut IndentedWriter::new(out, self.config.indent);

        check_reserved_names(registry, &naming::RULES)?;

        let registry = &Self::update_qualified_names(config, registry);
        let lang = self.lang(config, registry);

        Module::new(config).write(w, &lang)?;

        for container in registry.iter().map(Container::from) {
            writeln!(w)?;
            container.write(w, &lang)?;
        }

        Ok(())
    }

    /// Render the companion files for `registry` with `config`, as returned
    /// by [`module_config`](Self::module_config).
    pub(crate) fn render_companion_files(
        &self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> Result<Vec<CompanionFile>> {
        let lang = self.lang(config, registry);

        render_companion_files(lang.plugins(), config, |imports| {
            let mut header = Vec::new();
            write_module_header(
                &mut IndentedWriter::new(&mut header, config.indent),
                config,
                &lang,
                imports,
            )?;
            String::from_utf8(header).map_err(std::io::Error::other)
        })
    }

    /// Rewrites every type reference in `registry` with
    /// [`requalify`](Self::requalify), returning a new registry.
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            Self::requalify_type_names(config, container_format);
        }

        updated_registry
    }

    /// Rewrites every type reference in `holder` with
    /// [`requalify`](Self::requalify), exported to plugins for a [`Format`] as
    /// [`swift::requalify_format`](crate::generation::swift::requalify_format).
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

    /// The spelling a reference to `name` is written with.
    ///
    /// A ROOT type seen from a namespaced module lives in the root package's
    /// target, so it is qualified with it (`Example.Shared`), which also keeps
    /// a same-named type of the module's own from capturing it. Only when the
    /// config knows the root package ([`CodeGeneratorConfig::parent`], which
    /// the installer sets); otherwise, and in the root module itself, every
    /// reference is unchanged. Exported to plugins as
    /// [`swift::requalify`](crate::generation::swift::requalify).
    pub(crate) fn requalify(
        config: &CodeGeneratorConfig,
        name: &QualifiedTypeName,
    ) -> QualifiedTypeName {
        match &name.namespace {
            Namespace::Root if config.root_package() != config.module_name() => {
                QualifiedTypeName::namespaced(config.root_package().to_string(), name.name.clone())
            }
            _ => name.clone(),
        }
    }

    /// Records the ROOT types among `names` (the types a namespaced module
    /// references, in registry spelling) in its
    /// [`external_definitions`](CodeGeneratorConfig::external_definitions),
    /// under the root package, so that the module imports the root package's
    /// target and the installer makes it a dependency.
    pub(crate) fn reference_root_types<'n>(
        config: &mut CodeGeneratorConfig,
        names: impl IntoIterator<Item = &'n QualifiedTypeName>,
    ) {
        let root_package = config.root_package().to_string();
        if root_package == config.module_name() {
            return;
        }

        let names: BTreeSet<&str> = names
            .into_iter()
            .filter(|name| name.namespace == Namespace::Root)
            .map(|name| name.name.as_str())
            .collect();
        if names.is_empty() {
            return;
        }

        let known = config.external_definitions.entry(root_package).or_default();
        for name in names {
            if !known.iter().any(|k| k == name) {
                known.push(name.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests;
