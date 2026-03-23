//! Top-level orchestrator for Swift code generation.
//!
//! [`SwiftCodeGenerator`] implements [`CodeGenerator`] and is the entry point for
//! producing a single Swift source file from a [`Registry`].

use std::{
    collections::BTreeSet,
    io::{Result, Write},
};

use crate::{
    Registry,
    generation::{
        CodeGenerator, CodeGeneratorConfig, Container, Emitter, Encoding,
        indent::{IndentConfig, IndentedWriter},
        module::Module,
        swift::emitter::Swift,
    },
    reflection::format::{
        ContainerFormat, Format, FormatHolder, Named, Namespace, QualifiedTypeName, VariantFormat,
    },
};

/// Main configuration object for Swift code generation.
///
/// Wraps a [`CodeGeneratorConfig`] and implements [`CodeGenerator`] so it can be
/// used by the installer pipeline.
pub struct SwiftCodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
}

impl<'a> CodeGenerator<'a> for SwiftCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        SwiftCodeGenerator::new(config)
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
    /// Create a Swift code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self { config }
    }

    /// Produce a complete Swift source file for `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, IndentConfig::Space(4));

        let mut config = self.config.clone();
        config.update_from(registry);

        let lang = Swift::for_encoding(config.encoding, registry);

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

/// Extends [`Swift`] with the standard constructor that computes type sets
/// from a registry.
///
/// This is the standard constructor whenever a registry is available. The
/// precomputed [`Swift::hashable_types`] and [`Swift::equatable_types`] sets
/// are derived here using a monotone fixed-point pass over the registry.
///
/// > **Future direction** — when the plugin branch is merged this method will
/// > install `EmitterPlugin` implementations instead of precomputing sets, and
/// > the `registry` parameter will be dropped in favour of per-container
/// > registry access via `EmitContext`.
impl Swift {
    /// Create a [`Swift`] language tag with type sets computed from `registry`.
    ///
    /// Prefer this over [`Swift::new`] whenever a registry is available.
    #[must_use]
    pub fn for_encoding(encoding: Encoding, registry: &Registry) -> Self {
        Self {
            encoding,
            hashable_types: compute_hashable_types(registry),
            equatable_types: compute_equatable_types(registry),
        }
    }
}

/// Computes the set of type names (within this module) that can synthesise
/// Swift `Hashable` conformance.
///
/// Uses a monotone fixed-point iteration: on each pass a type is added to
/// the known-hashable set if all of its fields / variant associated values
/// are hashable given current knowledge. Iteration stops when a full pass
/// produces no new additions.
///
/// External types (not present in the registry) are assumed to be hashable.
/// Mutually recursive types that form a cycle will conservatively not be
/// added to the set.
pub(crate) fn compute_hashable_types(registry: &Registry) -> BTreeSet<String> {
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
            if container_can_be_hashable(container, &known, &local_names) {
                known.insert(name.clone());
                changed = true;
            }
        }
    }

    known
}

fn container_can_be_hashable(
    format: &ContainerFormat,
    known: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) -> bool {
    match format {
        ContainerFormat::UnitStruct(_) => true,
        ContainerFormat::NewTypeStruct(fmt, _) => fmt_can_be_hashable(fmt, known, local_names),
        ContainerFormat::TupleStruct(fmts, _) => fmts
            .iter()
            .all(|f| fmt_can_be_hashable(f, known, local_names)),
        ContainerFormat::Struct(nameds, _) => nameds
            .iter()
            .all(|n| fmt_can_be_hashable(&n.value, known, local_names)),
        ContainerFormat::Enum(variants, _) => variants
            .values()
            .all(|v| variant_can_be_hashable(&v.value, known, local_names)),
    }
}

fn variant_can_be_hashable(
    format: &VariantFormat,
    known: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => fmt_can_be_hashable(fmt, known, local_names),
        // Enum associated values are separate parameters, not a Swift tuple,
        // so each element is checked individually.
        VariantFormat::Tuple(fmts) => fmts
            .iter()
            .all(|f| fmt_can_be_hashable(f, known, local_names)),
        VariantFormat::Struct(nameds) => nameds
            .iter()
            .all(|n| fmt_can_be_hashable(&n.value, known, local_names)),
    }
}

fn fmt_can_be_hashable(
    format: &Format,
    known: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) -> bool {
    match format {
        Format::TypeName(qtn) => {
            if local_names.contains(&qtn.name) {
                // Same-module type: only hashable if already proven so.
                known.contains(&qtn.name)
            } else {
                // External type: assume hashable.
                true
            }
        }
        // Void is not Hashable
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
        | Format::Bytes => true,
        Format::Option(inner)
        | Format::Set(inner)
        | Format::Seq(inner)
        | Format::TupleArray { content: inner, .. } => {
            fmt_can_be_hashable(inner, known, local_names)
        }
        Format::Variable(_) | Format::Unit | Format::Map { .. } => false, // [K: V] is never Hashable
        // A 1-element tuple is transparent; multi-element native tuples are not Hashable.
        Format::Tuple(formats) => {
            formats.len() == 1 && fmt_can_be_hashable(&formats[0], known, local_names)
        }
    }
}

/// Computes the set of type names (within this module) that can synthesise
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
pub(crate) fn compute_equatable_types(registry: &Registry) -> BTreeSet<String> {
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
            if container_can_be_equatable(container, &known, &local_names) {
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
    local_names: &BTreeSet<String>,
) -> bool {
    match format {
        ContainerFormat::UnitStruct(_) => true,
        ContainerFormat::NewTypeStruct(fmt, _) => fmt_can_be_equatable(fmt, known, local_names),
        ContainerFormat::TupleStruct(fmts, _) => fmts
            .iter()
            .all(|f| fmt_can_be_equatable(f, known, local_names)),
        ContainerFormat::Struct(nameds, _) => nameds
            .iter()
            .all(|n| fmt_can_be_equatable(&n.value, known, local_names)),
        ContainerFormat::Enum(variants, _) => variants
            .values()
            .all(|v| variant_fmt_can_be_equatable(&v.value, known, local_names)),
    }
}

fn variant_fmt_can_be_equatable(
    format: &VariantFormat,
    known: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => fmt_can_be_equatable(fmt, known, local_names),
        // Enum associated values are separate parameters, not a Swift tuple,
        // so each element is checked individually.
        VariantFormat::Tuple(fmts) => fmts
            .iter()
            .all(|f| fmt_can_be_equatable(f, known, local_names)),
        VariantFormat::Struct(nameds) => nameds
            .iter()
            .all(|n| fmt_can_be_equatable(&n.value, known, local_names)),
    }
}

fn fmt_can_be_equatable(
    format: &Format,
    known: &BTreeSet<String>,
    local_names: &BTreeSet<String>,
) -> bool {
    match format {
        Format::Variable(_) => false,
        Format::TypeName(qtn) => {
            if local_names.contains(&qtn.name) {
                // Same-module type: only equatable if already proven so.
                known.contains(&qtn.name)
            } else {
                // External type: assume equatable.
                true
            }
        }
        Format::Unit
        | Format::Bool
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
        | Format::Bytes => true, // Void IS Equatable in Swift
        Format::Option(inner)
        | Format::Set(inner)
        | Format::Seq(inner)
        | Format::TupleArray { content: inner, .. } => {
            fmt_can_be_equatable(inner, known, local_names)
        }
        // [K: V] IS Equatable when K and V are Equatable.
        Format::Map { key, value } => {
            fmt_can_be_equatable(key, known, local_names)
                && fmt_can_be_equatable(value, known, local_names)
        }
        // Multi-element tuples are included: we generate a manual `==` operator
        // for structs/enums that contain them, using Swift's built-in tuple `==`.
        Format::Tuple(formats) => formats
            .iter()
            .all(|f| fmt_can_be_equatable(f, known, local_names)),
    }
}

#[cfg(test)]
mod tests;
