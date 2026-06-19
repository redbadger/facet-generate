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
        CodeGenerator, CodeGeneratorConfig, Container, Emitter, indent::IndentedWriter,
        module::Module, plugin::EmitterPlugin, swift::emitter::Swift,
    },
    reflection::format::{ContainerFormat, Format, QualifiedTypeName, VariantFormat},
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
    /// Returns an error if writing to `out` fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, self.config.indent);

        let mut config = self.config.clone();
        config.update_from(registry);

        let mut lang = Swift::new(&config, registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

        Module::new(&config).write(w, &lang)?;

        for container in registry.iter().map(Container::from) {
            writeln!(w)?;
            container.write(w, &lang)?;
        }

        Ok(())
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
        ContainerFormat::Enum(variants, _) => variants
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
        ContainerFormat::Enum(variants, _) => variants
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
