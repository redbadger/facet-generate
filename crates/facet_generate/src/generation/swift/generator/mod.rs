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

        let updated_registry = Self::update_qualified_names(&config, registry);
        for container in updated_registry.iter().map(Container::from) {
            writeln!(w)?;
            container.write(w, &lang)?;
        }

        Ok(())
    }

    /// Updates `QualifiedTypeName` instances so external types include their namespace prefix.
    /// For example, a type `Tree` in namespace `foo` becomes `Foo.Tree` in the output.
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            let _ = container_format.visit_mut(&mut |format| {
                if let Format::TypeName(qualified_name) = format
                    && let Namespace::Named(namespace) = &qualified_name.namespace
                {
                    if namespace == config.module_name() {
                        // Same-module type: strip namespace so it renders as a bare name
                        *qualified_name = QualifiedTypeName::root(qualified_name.name.clone());
                    } else if config.external_definitions.contains_key(namespace) {
                        *qualified_name = QualifiedTypeName::namespaced(
                            namespace.clone(),
                            qualified_name.name.clone(),
                        );
                    }
                }
                Ok(())
            });
        }

        updated_registry
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
pub fn compute_hashable_types(registry: &Registry) -> BTreeSet<String> {
    let local_names: BTreeSet<&str> = registry
        .keys()
        .filter(|k| matches!(k.namespace, Namespace::Root))
        .map(|k| k.name.as_str())
        .collect();

    let mut known: BTreeSet<String> = BTreeSet::new();
    let mut visiting: BTreeSet<String> = BTreeSet::new();

    for qtn in registry.keys() {
        if local_names.contains(qtn.name.as_str()) && !known.contains(&qtn.name) {
            check_type_hashable(registry, qtn, &local_names, &mut known, &mut visiting);
        }
    }

    known
}

/// Checks whether a type (and transitively all types it references) can
/// conform to `Hashable`.
///
/// On success the type name is inserted into `known` so that subsequent
/// checks short-circuit.
fn check_type_hashable(
    registry: &Registry,
    qtn: &QualifiedTypeName,
    local_names: &BTreeSet<&str>,
    known: &mut BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    let name = &qtn.name;

    if known.contains(name) {
        return true;
    }
    if visiting.contains(name) {
        // Cycle detected — strongly-connected components are either all
        // hashable or all non-hashable, so optimism is safe here.
        return true;
    }
    if !local_names.contains(name.as_str()) {
        return true; // external type
    }

    let Some(container) = registry.get(qtn) else {
        return false;
    };

    visiting.insert(name.clone());

    let result = match container {
        ContainerFormat::UnitStruct(_) => true,
        ContainerFormat::NewTypeStruct(fmt, _) => {
            fmt_is_hashable(registry, fmt, local_names, known, visiting)
        }
        ContainerFormat::TupleStruct(fmts, _) => fmts
            .iter()
            .all(|f| fmt_is_hashable(registry, f, local_names, known, visiting)),
        ContainerFormat::Struct(fields, _) => fields
            .iter()
            .all(|f| fmt_is_hashable(registry, &f.value, local_names, known, visiting)),
        ContainerFormat::Enum(variants, _) => variants.values().all(|v| {
            variant_is_hashable(registry, &v.value, local_names, known, visiting)
        }),
    };

    visiting.remove(name);

    if result {
        known.insert(name.clone());
    }

    result
}

fn variant_is_hashable(
    registry: &Registry,
    format: &VariantFormat,
    local_names: &BTreeSet<&str>,
    known: &mut BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => {
            fmt_is_hashable(registry, fmt, local_names, known, visiting)
        }
        VariantFormat::Tuple(fmts) => fmts
            .iter()
            .all(|f| fmt_is_hashable(registry, f, local_names, known, visiting)),
        VariantFormat::Struct(fields) => fields
            .iter()
            .all(|f| fmt_is_hashable(registry, &f.value, local_names, known, visiting)),
    }
}

fn fmt_is_hashable(
    registry: &Registry,
    format: &Format,
    local_names: &BTreeSet<&str>,
    known: &mut BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    match format {
        Format::TypeName(qtn) => {
            check_type_hashable(registry, qtn, local_names, known, visiting)
        }
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
            fmt_is_hashable(registry, inner, local_names, known, visiting)
        }
        Format::Map { key, value } => {
            fmt_is_hashable(registry, key, local_names, known, visiting)
                && fmt_is_hashable(registry, value, local_names, known, visiting)
        }
        Format::Tuple(formats) => {
            formats.len() == 1
                && fmt_is_hashable(registry, &formats[0], local_names, known, visiting)
        }
    }
}

/// Computes the set of type names (within this module) that can synthesize
/// Swift `Equatable` conformance.
///
/// Uses a monotone fixed-point iteration: on each pass a type is added to
/// the known-equatable set if all of its fields / variant associated values
/// are equatable given current knowledge. Iteration stops when a full pass
/// produces no new additions.
///
/// Differs from [`compute_hashable_types`] in three ways:
/// - `Unit` (Void) IS `Equatable` in Swift.
/// - `Map { K, V }` (`[K: V]`) IS `Equatable` when `K` and `V` are.
/// - Multi-element native tuples are counted as equatable (we emit a
///   manual `==` operator for them).
///
/// Mutually recursive or self-referential types that cannot resolve the
/// cycle will conservatively not be added to the set.
pub fn compute_equatable_types(registry: &Registry) -> BTreeSet<String> {
    // Names of all types defined in this module.
    let local_names: BTreeSet<String> = registry
        .keys()
        .filter(|k| matches!(k.namespace, Namespace::Root))
        .map(|k| k.name.clone())
        .collect();

    let mut known: BTreeSet<String> = BTreeSet::new();
    let mut changed = true;

    while changed {
        changed = false;
        for (qtn, container) in registry {
            let name = &qtn.name;
            if known.contains(name) {
                continue;
            }
            if container_can_be_equatable(container, &known, qtn.name.as_str(), &local_names) {
                known.insert(name.clone());
                changed = true;
            }
        }
    }

    known
}

fn container_can_be_equatable(
    format: &ContainerFormat,
    known: &BTreeSet<String>,
    container: &str,
    local_names: &BTreeSet<String>,
) -> bool {
    match format {
        ContainerFormat::UnitStruct(_) => true,
        ContainerFormat::NewTypeStruct(fmt, _) => {
            fmt_can_be_equatable(fmt, known, container, local_names)
        }
        ContainerFormat::TupleStruct(fmts, _) => fmts
            .iter()
            .all(|f| fmt_can_be_equatable(f, known, container, local_names)),
        ContainerFormat::Struct(nameds, _) => nameds
            .iter()
            .all(|n| fmt_can_be_equatable(&n.value, known, container, local_names)),
        ContainerFormat::Enum(variants, _) => variants
            .values()
            .all(|v| variant_fmt_can_be_equatable(&v.value, known, container, local_names)),
    }
}

fn variant_fmt_can_be_equatable(
    format: &VariantFormat,
    known: &BTreeSet<String>,
    container: &str,

    local_names: &BTreeSet<String>,
) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => fmt_can_be_equatable(fmt, known, container, local_names),
        // Enum associated values are separate parameters, not a Swift tuple,
        // so each element is checked individually.
        VariantFormat::Tuple(fmts) => fmts
            .iter()
            .all(|f| fmt_can_be_equatable(f, known, container, local_names)),
        VariantFormat::Struct(nameds) => nameds
            .iter()
            .all(|n| fmt_can_be_equatable(&n.value, known, container, local_names)),
    }
}

fn fmt_can_be_equatable(
    format: &Format,
    known: &BTreeSet<String>,
    container: &str,
    local_names: &BTreeSet<String>,
) -> bool {
    match format {
        Format::TypeName(qtn) => {
            if container == &qtn.name {
                // Cycle to the currently evaluated struct: assume equatable.
                true
            } else if local_names.contains(&qtn.name) {
                // Same-module type: only equatable if already proven so.
                known.contains(&qtn.name)
            } else {
                // External type: assume equatable.
                true
            }
        }
        // Void does not conform to Equatable in Swift — a stored property of
        // type Void prevents Equatable auto-synthesis.
        Format::Variable(_) | Format::Unit => false,
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
        Format::Option(inner)
        | Format::Set(inner)
        | Format::Seq(inner)
        | Format::TupleArray { content: inner, .. } => {
            fmt_can_be_equatable(inner, known, container, local_names)
        }
        // [K: V] IS Equatable when K and V are Equatable.
        Format::Map { key, value } => {
            fmt_can_be_equatable(key, known, container, local_names)
                && fmt_can_be_equatable(value, known, container, local_names)
        }
        // Multi-element tuples are included: we generate a manual `==` operator
        // for structs/enums that contain them, using Swift's built-in tuple `==`.
        Format::Tuple(formats) => formats
            .iter()
            .all(|f| fmt_can_be_equatable(f, known, container, local_names)),
    }
}

#[cfg(test)]
mod tests;
