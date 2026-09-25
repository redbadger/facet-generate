//! Which types conform to Swift's `Hashable` and `Equatable`.
//!
//! A type's conformance depends on the types it holds, which can live in
//! other modules, so the [`Installer`](super::Installer) decides it once, over
//! the whole registry, and hands the result to every module's generator:
//! the module that declares a type and each module that holds it then agree
//! on it. [`SwiftCodeGenerator`](super::SwiftCodeGenerator) used on its own
//! decides it over the registry it is given.
//!
//! The rules for a single value live here too, and the emitter declares a
//! type's conformance with the same ones, looking up the types it holds in
//! the sets computed here, so the declaration and the sets cannot disagree.

use std::collections::BTreeSet;

use crate::{
    Registry,
    generation::ExternalPackages,
    reflection::format::{ContainerFormat, Format, Namespace, QualifiedTypeName, VariantFormat},
};

/// Whether a type, by name, conforms to the protocol being asked about.
pub(crate) type Conforms<'a> = &'a dyn Fn(&QualifiedTypeName) -> bool;

/// The types of a registry that conform to `Hashable` and to `Equatable`, in
/// registry spelling.
#[derive(Debug, Clone, Default)]
pub(crate) struct Conformance {
    /// The types whose conformance was decided: every type in the registry
    /// except those of an external package. Any other type is assumed to
    /// conform to both.
    pub(crate) decided: BTreeSet<QualifiedTypeName>,
    /// The decided types that conform to `Hashable`.
    pub(crate) hashable: BTreeSet<QualifiedTypeName>,
    /// The decided types that conform to `Equatable`, synthesized or through
    /// a hand-written `==`.
    pub(crate) equatable: BTreeSet<QualifiedTypeName>,
}

impl Conformance {
    /// Decides the conformance of every type in `registry` except those in the
    /// namespace of one of `external_packages`, which are generated elsewhere,
    /// so are assumed to conform, like types absent from `registry`.
    ///
    /// Each set is the greatest fixed point: start from every decided type
    /// and drop, until none is left to drop, each type holding a value that
    /// does not conform. A cycle of types therefore conforms exactly when
    /// nothing else they hold stops it, whichever type is looked at first.
    pub(crate) fn of(registry: &Registry, external_packages: &ExternalPackages) -> Self {
        let decided: BTreeSet<QualifiedTypeName> = registry
            .keys()
            .filter(|name| match &name.namespace {
                Namespace::Named(ns) => !external_packages.contains_key(ns),
                Namespace::Root => true,
            })
            .cloned()
            .collect();

        let hashable = greatest_fixed_point(registry, &decided, container_is_hashable);
        let equatable = greatest_fixed_point(registry, &decided, container_is_equatable);

        Self {
            decided,
            hashable,
            equatable,
        }
    }
}

/// The largest subset of `decided` whose every type satisfies `holds`, given
/// that the types in it, and every type not in `decided`, conform.
fn greatest_fixed_point(
    registry: &Registry,
    decided: &BTreeSet<QualifiedTypeName>,
    holds: fn(&ContainerFormat, Conforms) -> bool,
) -> BTreeSet<QualifiedTypeName> {
    let mut conforming = decided.clone();
    loop {
        let failing: Vec<QualifiedTypeName> = conforming
            .iter()
            .filter(|name| {
                let conforms =
                    |t: &QualifiedTypeName| !decided.contains(t) || conforming.contains(t);
                !holds(&registry[*name], &conforms)
            })
            .cloned()
            .collect();
        if failing.is_empty() {
            return conforming;
        }
        for name in &failing {
            conforming.remove(name);
        }
    }
}

/// Each stored value of a struct-like container, as the emitter declares
/// them: none for a unit struct, `value` for a newtype, one per tuple element.
fn fields(container: &ContainerFormat) -> Vec<&Format> {
    match container {
        ContainerFormat::UnitStruct(_) | ContainerFormat::Enum(..) => vec![],
        ContainerFormat::NewTypeStruct(format, _) => vec![format.as_ref()],
        ContainerFormat::TupleStruct(formats, _) => formats.iter().collect(),
        ContainerFormat::Struct(fields, _) => fields.iter().map(|f| &f.value).collect(),
    }
}

fn container_is_hashable(container: &ContainerFormat, hashable: Conforms) -> bool {
    match container {
        ContainerFormat::Enum(variants, _, _) => {
            variants_are_hashable(variants.values().map(|v| &v.value), hashable)
        }
        _ => fields_are_hashable(fields(container), hashable),
    }
}

fn container_is_equatable(container: &ContainerFormat, equatable: Conforms) -> bool {
    match container {
        ContainerFormat::Enum(variants, _, _) => {
            variants_are_equatable(variants.values().map(|v| &v.value), equatable)
        }
        _ => fields_are_equatable(fields(container), equatable),
    }
}

/// Whether a struct with these fields conforms to `Hashable`.
pub(crate) fn fields_are_hashable<'f>(
    fields: impl IntoIterator<Item = &'f Format>,
    hashable: Conforms,
) -> bool {
    fields.into_iter().all(|f| is_hashable(f, hashable))
}

/// Whether a struct with these fields conforms to `Equatable`: synthesized
/// when every field is, or through a hand-written `==` when every field can
/// be compared with one.
pub(crate) fn fields_are_equatable<'f>(
    fields: impl IntoIterator<Item = &'f Format>,
    equatable: Conforms,
) -> bool {
    fields
        .into_iter()
        .all(|f| can_use_eq_operator(f, equatable))
}

/// Whether an enum with these variants conforms to `Hashable`.
pub(crate) fn variants_are_hashable<'v>(
    variants: impl IntoIterator<Item = &'v VariantFormat>,
    hashable: Conforms,
) -> bool {
    variants
        .into_iter()
        .all(|v| variant_is_hashable(v, hashable))
}

/// Whether an enum with these variants conforms to `Equatable`, synthesized
/// or through a hand-written `==` (see [`fields_are_equatable`]).
pub(crate) fn variants_are_equatable<'v>(
    variants: impl IntoIterator<Item = &'v VariantFormat>,
    equatable: Conforms,
) -> bool {
    variants
        .into_iter()
        .all(|v| variant_can_use_eq_operator(v, equatable))
}

/// Returns `true` if the Swift type produced by `format` conforms to
/// `Hashable`.
pub(crate) fn is_hashable(format: &Format, hashable: Conforms) -> bool {
    match format {
        // Void does not conform to Hashable in Swift
        Format::Variable(_) | Format::Unit => false,
        // [K: V] is Hashable iff K is hashable and V is hashable
        Format::Map { key, value } => is_hashable(key, hashable) && is_hashable(value, hashable),
        Format::TypeName(qtn) => hashable(qtn),
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
        | Format::TupleArray { content: inner, .. } => is_hashable(inner, hashable),
        // A 1-element tuple is transparent; multi-element native tuples are not Hashable.
        Format::Tuple(formats) => formats.len() == 1 && is_hashable(&formats[0], hashable),
    }
}

pub(crate) fn variant_is_hashable(format: &VariantFormat, hashable: Conforms) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => is_hashable(fmt, hashable),
        VariantFormat::Tuple(fmts) => fmts.iter().all(|f| is_hashable(f, hashable)),
        VariantFormat::Struct(nameds) => nameds.iter().all(|n| is_hashable(&n.value, hashable)),
    }
}

/// Returns `true` if the Swift type produced by `format` conforms to
/// `Equatable`, so a type holding it can synthesize `==`.
pub(crate) fn is_equatable_auto(format: &Format, equatable: Conforms) -> bool {
    match format {
        Format::TypeName(qtn) => equatable(qtn),
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
        | Format::TupleArray { content: inner, .. } => is_equatable_auto(inner, equatable),
        Format::Map { key, value } => {
            is_equatable_auto(key, equatable) && is_equatable_auto(value, equatable)
        }
        Format::Tuple(formats) => formats.len() == 1 && is_equatable_auto(&formats[0], equatable),
    }
}

/// Returns `true` if two values of `format` can be compared with `==`: an
/// `Equatable` type, or a native tuple of them, which Swift's built-in tuple
/// `==` compares.
pub(crate) fn can_use_eq_operator(format: &Format, equatable: Conforms) -> bool {
    match format {
        Format::Tuple(formats) if formats.len() > 1 => {
            formats.iter().all(|f| is_equatable_auto(f, equatable))
        }
        _ => is_equatable_auto(format, equatable),
    }
}

pub(crate) fn variant_is_equatable_auto(format: &VariantFormat, equatable: Conforms) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => is_equatable_auto(fmt, equatable),
        VariantFormat::Tuple(formats) => formats.iter().all(|f| is_equatable_auto(f, equatable)),
        VariantFormat::Struct(nameds) => nameds
            .iter()
            .all(|n| is_equatable_auto(&n.value, equatable)),
    }
}

pub(crate) fn variant_can_use_eq_operator(format: &VariantFormat, equatable: Conforms) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => can_use_eq_operator(fmt, equatable),
        VariantFormat::Tuple(formats) => formats.iter().all(|f| can_use_eq_operator(f, equatable)),
        VariantFormat::Struct(nameds) => nameds
            .iter()
            .all(|n| can_use_eq_operator(&n.value, equatable)),
    }
}

#[cfg(test)]
#[path = "conformance_tests.rs"]
mod tests;
