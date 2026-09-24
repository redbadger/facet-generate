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
            emitter::{Swift, write_module_header},
            naming,
        },
    },
    reflection::format::{
        ContainerFormat, Format, FormatHolder, Namespace, QualifiedTypeName, VariantFormat,
    },
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
}

impl<'a> CodeGenerator<'a> for SwiftCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            plugins: vec![],
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
        }
    }

    /// Set the pre-built plugin list, returning the modified generator.
    #[must_use]
    pub fn with_plugins(mut self, plugins: Vec<Arc<dyn EmitterPlugin<Swift>>>) -> Self {
        self.plugins = plugins;
        self
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
        let mut lang = Swift::new(config, registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

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
        let mut lang = Swift::new(config, registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

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

/// Computes the set of type names (within this module) that can synthesize
/// Swift `Hashable` conformance.
///
/// Uses depth-first search with optimistic cycle handling: if a type appears
/// on the current evaluation stack it is assumed hashable. This correctly
/// supports self-referential and mutually-recursive types because Swift
/// permits `Hashable` synthesis for recursive value types as long as all
/// non-recursive stored properties are themselves `Hashable`.
///
/// External types (not present in the registry) are assumed to be hashable.
pub fn compute_hashable_types(registry: &Registry) -> BTreeSet<QualifiedTypeName> {
    let mut known: BTreeSet<&QualifiedTypeName> = BTreeSet::new();
    let mut visiting: BTreeSet<&QualifiedTypeName> = BTreeSet::new();

    for qtn in registry.keys() {
        if !known.contains(qtn) {
            check_type_hashable(registry, qtn, &mut known, &mut visiting);
        }
    }

    known.into_iter().cloned().collect()
}

/// Checks whether a type (and transitively all types it references) can
/// conform to `Hashable`.
///
/// On success the qualified type name is inserted into `known` so that
/// subsequent checks short-circuit.
fn check_type_hashable<'a>(
    registry: &'a Registry,
    qtn: &'a QualifiedTypeName,
    known: &mut BTreeSet<&'a QualifiedTypeName>,
    visiting: &mut BTreeSet<&'a QualifiedTypeName>,
) -> bool {
    if known.contains(qtn) {
        return true;
    }
    if visiting.contains(qtn) {
        // Cycle detected — strongly-connected components are either all
        // hashable or all non-hashable, so optimism is safe here.
        return true;
    }

    let Some(container) = registry.get(qtn) else {
        return true; // external type (absent from the registry) — assume hashable
    };

    visiting.insert(qtn);

    let result = match container {
        ContainerFormat::UnitStruct(_) => true,
        ContainerFormat::NewTypeStruct(fmt, _) => fmt_is_hashable(registry, fmt, known, visiting),
        ContainerFormat::TupleStruct(fmts, _) => fmts
            .iter()
            .all(|f| fmt_is_hashable(registry, f, known, visiting)),
        ContainerFormat::Struct(fields, _) => fields
            .iter()
            .all(|f| fmt_is_hashable(registry, &f.value, known, visiting)),
        ContainerFormat::Enum(variants, _, _) => variants
            .values()
            .all(|v| variant_is_hashable(registry, &v.value, known, visiting)),
    };

    visiting.remove(qtn);

    if result {
        known.insert(qtn);
    }

    result
}

fn variant_is_hashable<'a>(
    registry: &'a Registry,
    format: &'a VariantFormat,
    known: &mut BTreeSet<&'a QualifiedTypeName>,
    visiting: &mut BTreeSet<&'a QualifiedTypeName>,
) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => fmt_is_hashable(registry, fmt, known, visiting),
        VariantFormat::Tuple(fmts) => fmts
            .iter()
            .all(|f| fmt_is_hashable(registry, f, known, visiting)),
        VariantFormat::Struct(fields) => fields
            .iter()
            .all(|f| fmt_is_hashable(registry, &f.value, known, visiting)),
    }
}

fn fmt_is_hashable<'a>(
    registry: &'a Registry,
    format: &'a Format,
    known: &mut BTreeSet<&'a QualifiedTypeName>,
    visiting: &mut BTreeSet<&'a QualifiedTypeName>,
) -> bool {
    match format {
        Format::TypeName(qtn) => check_type_hashable(registry, qtn, known, visiting),
        Format::Bool
        | Format::I8
        | Format::I16
        | Format::I32
        | Format::I64
        | Format::I128
        | Format::U8
        | Format::U16
        | Format::U32
        | Format::U64
        | Format::U128
        | Format::F32
        | Format::F64
        | Format::Char
        | Format::Str
        | Format::Bytes
        | Format::Uuid => true,
        Format::Variable(_) | Format::Unit => false,
        Format::Option(inner)
        | Format::Set(inner)
        | Format::Seq(inner)
        | Format::TupleArray { content: inner, .. } => {
            fmt_is_hashable(registry, inner, known, visiting)
        }
        Format::Map { key, value } => {
            fmt_is_hashable(registry, key, known, visiting)
                && fmt_is_hashable(registry, value, known, visiting)
        }
        Format::Tuple(formats) => {
            formats.len() == 1 && fmt_is_hashable(registry, &formats[0], known, visiting)
        }
    }
}

/// Computes the set of type names (within this module) that can synthesize
/// Swift `Equatable` conformance.
///
/// Uses depth-first search with optimistic cycle handling: if a type appears
/// on the current evaluation stack it is assumed equatable. This correctly
/// supports self-referential and mutually-recursive types because Swift
/// permits `Equatable` synthesis (or manual `==` emission) for recursive
/// value types as long as all non-recursive stored properties are themselves
/// `Equatable`.
///
/// Multi-element tuples are included because the emitter generates a manual
/// `==` operator using Swift's built-in tuple `==`.
///
/// External types (not present in the registry) are assumed to be equatable.
pub fn compute_equatable_types(registry: &Registry) -> BTreeSet<QualifiedTypeName> {
    let mut known: BTreeSet<&QualifiedTypeName> = BTreeSet::new();
    let mut visiting: BTreeSet<&QualifiedTypeName> = BTreeSet::new();

    for qtn in registry.keys() {
        if !known.contains(qtn) {
            check_type_equatable(registry, qtn, &mut known, &mut visiting);
        }
    }

    known.into_iter().cloned().collect()
}

/// Checks whether a type (and transitively all types it references) can
/// conform to `Equatable`.
///
/// On success the qualified type name is inserted into `known` so that
/// subsequent checks short-circuit.
fn check_type_equatable<'a>(
    registry: &'a Registry,
    qtn: &'a QualifiedTypeName,
    known: &mut BTreeSet<&'a QualifiedTypeName>,
    visiting: &mut BTreeSet<&'a QualifiedTypeName>,
) -> bool {
    if known.contains(qtn) {
        return true;
    }
    if visiting.contains(qtn) {
        // Cycle detected — strongly-connected components are either all
        // equatable or all non-equatable, so optimism is safe here.
        return true;
    }

    let Some(container) = registry.get(qtn) else {
        return true; // external type (absent from the registry) — assume equatable
    };

    visiting.insert(qtn);

    let result = match container {
        ContainerFormat::UnitStruct(_) => true,
        ContainerFormat::NewTypeStruct(fmt, _) => fmt_is_equatable(registry, fmt, known, visiting),
        ContainerFormat::TupleStruct(fmts, _) => fmts
            .iter()
            .all(|f| fmt_is_equatable(registry, f, known, visiting)),
        ContainerFormat::Struct(fields, _) => fields
            .iter()
            .all(|f| fmt_is_equatable(registry, &f.value, known, visiting)),
        ContainerFormat::Enum(variants, _, _) => variants
            .values()
            .all(|v| variant_is_equatable(registry, &v.value, known, visiting)),
    };

    visiting.remove(qtn);

    if result {
        known.insert(qtn);
    }

    result
}

fn variant_is_equatable<'a>(
    registry: &'a Registry,
    format: &'a VariantFormat,
    known: &mut BTreeSet<&'a QualifiedTypeName>,
    visiting: &mut BTreeSet<&'a QualifiedTypeName>,
) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => fmt_is_equatable(registry, fmt, known, visiting),
        VariantFormat::Tuple(fmts) => fmts
            .iter()
            .all(|f| fmt_is_equatable(registry, f, known, visiting)),
        VariantFormat::Struct(fields) => fields
            .iter()
            .all(|f| fmt_is_equatable(registry, &f.value, known, visiting)),
    }
}

fn fmt_is_equatable<'a>(
    registry: &'a Registry,
    format: &'a Format,
    known: &mut BTreeSet<&'a QualifiedTypeName>,
    visiting: &mut BTreeSet<&'a QualifiedTypeName>,
) -> bool {
    match format {
        Format::TypeName(qtn) => check_type_equatable(registry, qtn, known, visiting),
        Format::Bool
        | Format::I8
        | Format::I16
        | Format::I32
        | Format::I64
        | Format::I128
        | Format::U8
        | Format::U16
        | Format::U32
        | Format::U64
        | Format::U128
        | Format::F32
        | Format::F64
        | Format::Char
        | Format::Str
        | Format::Bytes
        | Format::Uuid => true,
        Format::Variable(_) | Format::Unit => false,
        Format::Option(inner)
        | Format::Set(inner)
        | Format::Seq(inner)
        | Format::TupleArray { content: inner, .. } => {
            fmt_is_equatable(registry, inner, known, visiting)
        }
        Format::Map { key, value } => {
            fmt_is_equatable(registry, key, known, visiting)
                && fmt_is_equatable(registry, value, known, visiting)
        }
        Format::Tuple(formats) => formats
            .iter()
            .all(|f| fmt_is_equatable(registry, f, known, visiting)),
    }
}

#[cfg(test)]
mod tests;
