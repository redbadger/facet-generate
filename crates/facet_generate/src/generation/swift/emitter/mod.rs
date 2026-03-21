//! AST-to-Swift source rendering.
//!
//! This module implements [`Emitter<Swift>`](super::super::Emitter) for each
//! node type in the format AST, turning abstract type descriptions into
//! idiomatic Swift code.
//!
//! # Emitter implementations
//!
//! | AST node | Swift output |
//! |---|---|
//! | [`Module`] | `import` statements, feature helpers |
//! | [`Container`] | `public struct` or `indirect public enum` |
//! | [`Named<Format>`](Named) | `public var` property / `case` declaration |
//! | [`Format`] | Inline type expression (`Int32`, `[String]`, `Set<T>`, …) |
//! | [`Doc`] | `///` doc comments (stripped for bincode) |
//! | `(Named<VariantFormat>, Usage)` | An enum case variant |
//!
//! # Swift type mapping
//!
//! The [`Format`] emitter maps Rust/reflection types to Swift equivalents —
//! for example `I32` → `Int32`, `Seq(T)` → `[T]`, `Option(T)` → `T?`,
//! `Map(K,V)` → `[K: V]`, tuples of any size → native `(A, B)` (always).
//!
//! # Encoding-dependent output
//!
//! The [`Swift`] language tag carries the active [`Encoding`]. When encoding
//! is `Json`, types get `Serializer`/`Deserializer` protocol-based
//! serialization methods plus `jsonSerialize`/`jsonDeserialize` wrappers.
//! When encoding is `Bincode`, each type gets `serialize`/`deserialize`
//! methods and `bincodeSerialize`/`bincodeDeserialize` wrappers. When
//! encoding is `None`, only plain type declarations are emitted.
//!
//! Types whose fields are all [`Hashable`](https://developer.apple.com/documentation/swift/hashable)
//! will declare `: Hashable` conformance so they can serve as `Set` elements
//! or `Dictionary` keys. A generation-time error is raised if a non-`Hashable`
//! type (native tuple or `[K:V]` dictionary) is used directly as a `Set`
//! element or `Map` key.
//!
//! # Feature helpers (`features/` directory)
//!
//! Swift has `Array`, `Set`, `Dictionary`, and optional types built in, but
//! the Serde `Serializer`/`Deserializer` runtime only handles primitives and
//! user-defined types (which get their own `serialize`/`deserialize` methods).
//! The feature helpers are Swift functions that bridge this gap — they teach
//! the serde runtime how to length-prefix and iterate over generic containers.
//!
//! | Helper | What it provides | When included |
//! |---|---|---|
//! | `ListOfT.swift` | `[T]` serialize/deserialize | Bincode + `Seq` type used |
//! | `SetOfT.swift` | `Set<T>` serialize/deserialize | Bincode + `Set` type used |
//! | `MapOfT.swift` | `[K:V]` serialize/deserialize | Bincode + `Map` type used |
//! | `OptionOfT.swift` | `T?` serialize/deserialize | Bincode + `Option` type used |
//! | `TupleArray.swift` | Fixed-size array support | `TupleArray` type used |
//!
//! These `.swift` snippets are embedded at compile time via `include_bytes!`
//! and written into the file header by the [`Module`] emitter when the
//! corresponding [`Feature`] flag is active (discovered automatically by
//! [`CodeGeneratorConfig::update_from`]).

#![allow(clippy::too_many_lines)]
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Result, Write},
    sync::Arc,
};

use heck::ToLowerCamelCase as _;
use indoc::{formatdoc, writedoc};

use heck::ToUpperCamelCase as _;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Container, Emitter, Encoding, Feature,
        indent::{IndentWrite, Newlines},
        module::Module,
        plugin::EmitterPlugin,
        swift::generator::{compute_equatable_types, compute_hashable_types},
    },
    reflection::format::{
        ContainerFormat, Doc, Format, Named, Namespace, QualifiedTypeName, VariantFormat,
    },
};

const FEATURE_LIST_OF_T: &[u8] = include_bytes!("features/ListOfT.swift");
const FEATURE_MAP_OF_T: &[u8] = include_bytes!("features/MapOfT.swift");
const FEATURE_OPTION_OF_T: &[u8] = include_bytes!("features/OptionOfT.swift");
const FEATURE_SET_OF_T: &[u8] = include_bytes!("features/SetOfT.swift");
const FEATURE_TUPLE_ARRAY: &[u8] = include_bytes!("features/TupleArray.swift");

/// Language tag for Swift code generation.
///
/// Carries the active [`Encoding`] and the sets of type names (within the
/// current module) that are known to be able to synthesize `Hashable` and
/// `Equatable` conformance respectively. Both sets are computed by a
/// preprocessing pass — see
/// [`SwiftCodeGenerator`](crate::generation::swift::generator::SwiftCodeGenerator)
/// and the `for_encoding` constructor.
///
/// An empty set means "no same-module type is known to be hashable/equatable"
/// (conservative). Use [`new`](Self::new) and supply a [`Registry`](crate::Registry)
/// so that the sets are computed correctly.
///
/// > **Future direction** — when the plugin branch is merged, `hashable_types`
/// > and `equatable_types` will be replaced by `EmitterPlugin` implementations
/// > that receive the registry via `EmitContext.container.registry` at emission
/// > time. `for_encoding` will then simply install the appropriate plugins and
/// > the precomputed sets will be removed.
///
/// Carries the active [`Encoding`] so that each emitter implementation can
/// decide whether to emit serialization methods, protocol conformances, or
/// plain type declarations.
#[derive(Debug, Clone)]
pub struct Swift {
    pub(crate) encoding: Encoding,
    /// Type names (root-namespace) that can synthesize `Hashable` conformance.
    /// Empty → no same-module type is known to be hashable (conservative).
    pub(crate) hashable_types: BTreeSet<String>,
    /// Type names (root-namespace) that can synthesize or manually implement
    /// `Equatable` conformance.
    /// Empty → no same-module type is known to be equatable (conservative).
    pub(crate) equatable_types: BTreeSet<String>,
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Self>>>,
}

impl Swift {
    /// Create a Swift language tag with computed type sets.
    #[must_use]
    pub fn new(config: &CodeGeneratorConfig, registry: &Registry) -> Self {
        Self {
            encoding: config.encoding,
            hashable_types: compute_hashable_types(registry),
            equatable_types: compute_equatable_types(registry),
            plugins: vec![],
        }
    }

    /// Access the plugin list.
    #[must_use]
    pub fn plugins(&self) -> &[Arc<dyn EmitterPlugin<Self>>] {
        &self.plugins
    }
}

// ---------------------------------------------------------------------------
// Hashability helpers
// ---------------------------------------------------------------------------

/// Returns `true` if the Swift type produced by `format` can conform to
/// `Hashable`.
///
/// This drives two decisions:
/// 1. Whether a generated struct/enum should declare `: Hashable`.
/// 2. Whether a `Set` element or `Map` key type is valid (error if not).
///
/// For `TypeName` references the answer depends on the precomputed
/// [`Swift::hashable_types`] set:
/// - `Namespace::Root` types are looked up in the set (or assumed hashable
///   when the set is absent).
/// - `Namespace::Named` types are external and assumed hashable.
pub(crate) fn is_hashable(format: &Format, lang: &Swift) -> bool {
    match format {
        Format::Variable(_)
        | Format::Unit  // Void does not conform to Hashable in Swift
        | Format::Map { .. } => false, // [K: V] is never Hashable
        Format::TypeName(qtn) => match &qtn.namespace {
            Namespace::Root => lang.hashable_types.contains(&qtn.name),
            Namespace::Named(_) => true, // external — assume hashable
        },
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
        | Format::Seq(inner)
        | Format::TupleArray { content: inner, .. }
        | Format::Set(inner) => is_hashable(inner, lang),
        // A 1-element tuple is transparent (emitted as the inner type).
        // Multi-element native Swift tuples do NOT conform to Hashable.
        Format::Tuple(formats) => formats.len() == 1 && is_hashable(&formats[0], lang),
    }
}

/// Returns `true` if a variant is compatible with `Hashable` synthesis for
/// the enclosing enum.
///
/// Note: enum associated values are separate parameters, not a Swift tuple,
/// so `VariantFormat::Tuple` checks each element individually.
fn variant_is_hashable(format: &VariantFormat, lang: &Swift) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => is_hashable(fmt, lang),
        VariantFormat::Tuple(formats) => formats.iter().all(|f| is_hashable(f, lang)),
        VariantFormat::Struct(nameds) => nameds.iter().all(|n| is_hashable(&n.value, lang)),
    }
}

/// Controls how a [`Named<Format>`](Named) is rendered in different contexts.
///
/// The same field definition produces different Swift syntax depending on
/// whether it appears as a property declaration, an initializer parameter,
/// a serialization call, etc.
enum Usage {
    Field,
    /// Like `Field` but prefixed with `@Indirect` for recursive struct fields
    /// that would otherwise create an infinite-size value type.
    IndirectField,
    Parameter,
    Argument,
    Assignment,
    Serialize {
        receiver: String,
    },
    Deserialize {
        receiver: String,
    },
}

// ---------------------------------------------------------------------------
// Recursion / indirection helpers
// ---------------------------------------------------------------------------

/// Returns `true` if a struct field of this format would create an
/// infinite-size value-type cycle back to the containing struct named
/// `struct_name`.
///
/// Recurses through value-typed wrappers (`Option`, tuples) but stops at
/// heap-allocated containers (`Seq`, `Map`, `Set`, `TupleArray`), because
/// those break the infinite-size chain.
fn needs_indirect(format: &Format, struct_name: &str) -> bool {
    match format {
        Format::TypeName(qtn) => qtn.name == struct_name,
        Format::Option(inner) => needs_indirect(inner, struct_name),
        Format::Tuple(formats) => formats.iter().any(|f| needs_indirect(f, struct_name)),
        // Seq / Map / Set / TupleArray use heap storage — no infinite-size cycle.
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Equatable helpers
// ---------------------------------------------------------------------------

/// Returns `true` if Swift can **auto-synthesize** `Equatable` for a field
/// of this format type.
///
/// Differs from [`is_hashable`] in two ways:
/// - `Map { .. }` → `true` when key and value are equatable — `[K:V]` IS `Equatable`.
/// - `TypeName(Namespace::Root)` → looked up in [`Swift::equatable_types`] (or assumed
///   equatable when the set is absent); `Namespace::Named` types are external and assumed
///   equatable.
///
/// The only format that blocks auto-synthesis is a multi-element native tuple,
/// because tuples do not conform to the `Equatable` *protocol* in Swift.
fn is_equatable_auto(format: &Format, lang: &Swift) -> bool {
    match format {
        // Look up same-module types in the equatable set; external types are assumed equatable.
        Format::TypeName(qtn) => match &qtn.namespace {
            Namespace::Root => lang.equatable_types.contains(&qtn.name),
            Namespace::Named(_) => true, // external — assume equatable
        },
        // Void does not conform to Equatable in Swift — a stored property of
        // type Void prevents both Hashable and Equatable auto-synthesis.
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
        | Format::Bytes => true,
        Format::Option(inner) | Format::Set(inner) => is_equatable_auto(inner, lang),
        Format::Seq(inner) | Format::TupleArray { content: inner, .. } => {
            is_equatable_auto(inner, lang)
        }
        // [K: V] IS Equatable when K and V are Equatable.
        Format::Map { key, value } => {
            is_equatable_auto(key, lang) && is_equatable_auto(value, lang)
        }
        // A 1-element tuple is transparent (emitted as the inner type).
        // Multi-element native tuples do NOT conform to the Equatable protocol.
        Format::Tuple(formats) => formats.len() == 1 && is_equatable_auto(&formats[0], lang),
    }
}

/// Returns `true` if `lhs.field == rhs.field` compiles in Swift for a field
/// of this format type.
///
/// Identical to [`is_equatable_auto`] except that multi-element native tuples
/// are accepted when all their elements are auto-equatable. Swift supplies a
/// built-in `==` operator for tuples whose elements are `Equatable`, even
/// though tuples themselves do not conform to the `Equatable` *protocol*.
fn can_use_eq_operator(format: &Format, lang: &Swift) -> bool {
    match format {
        Format::Tuple(formats) if formats.len() > 1 => {
            formats.iter().all(|f| is_equatable_auto(f, lang))
        }
        _ => is_equatable_auto(format, lang),
    }
}

/// Variant-level counterpart to [`is_equatable_auto`].
///
/// Note: `VariantFormat::Tuple` represents separate enum associated values,
/// not a Swift tuple, so each element is checked individually.
fn variant_is_equatable_auto(format: &VariantFormat, lang: &Swift) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => is_equatable_auto(fmt, lang),
        VariantFormat::Tuple(formats) => formats.iter().all(|f| is_equatable_auto(f, lang)),
        VariantFormat::Struct(nameds) => nameds.iter().all(|n| is_equatable_auto(&n.value, lang)),
    }
}

/// Variant-level counterpart to [`can_use_eq_operator`].
fn variant_can_use_eq_operator(format: &VariantFormat, lang: &Swift) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => can_use_eq_operator(fmt, lang),
        VariantFormat::Tuple(formats) => formats.iter().all(|f| can_use_eq_operator(f, lang)),
        VariantFormat::Struct(nameds) => nameds.iter().all(|n| can_use_eq_operator(&n.value, lang)),
    }
}

impl Emitter<Swift> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: &Swift) -> Result<()> {
        let CodeGeneratorConfig {
            encoding, features, ..
        } = self.config();

        let mut imports = vec![];
        if !encoding.is_none() {
            imports.push("Serde".to_string());
        }
        for ns in self.config().external_definitions.keys() {
            imports.push(ns.to_upper_camel_case());
        }
        imports.sort();
        imports.dedup();

        for import in &imports {
            writeln!(w, "import {import}")?;
        }

        if encoding.is_none() {
            return Ok(());
        }

        for feature in features {
            match feature {
                Feature::OptionOfT => {
                    writeln!(w)?;
                    w.write_all(FEATURE_OPTION_OF_T)?;
                }
                Feature::ListOfT => {
                    writeln!(w)?;
                    w.write_all(FEATURE_LIST_OF_T)?;
                }
                Feature::SetOfT => {
                    writeln!(w)?;
                    w.write_all(FEATURE_SET_OF_T)?;
                }
                Feature::MapOfT => {
                    writeln!(w)?;
                    w.write_all(FEATURE_MAP_OF_T)?;
                }
                Feature::TupleArray => {
                    writeln!(w)?;
                    w.write_all(FEATURE_TUPLE_ARRAY)?;
                }
                Feature::BigInt | Feature::Bytes => {}
            }
        }

        Ok(())
    }
}

impl Emitter<Swift> for Container<'_> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Swift) -> Result<()> {
        let Container {
            name: QualifiedTypeName { namespace: _, name },
            format,
            ..
        } = self;
        match format {
            ContainerFormat::UnitStruct(doc) => struct_(w, name, &[], doc, lang),
            ContainerFormat::NewTypeStruct(format, doc) => struct_(
                w,
                name,
                &[&Named::new(format, "value".to_string())],
                doc,
                lang,
            ),
            ContainerFormat::TupleStruct(formats, doc) => {
                let formats = named(formats, "field");
                struct_(w, name, &formats.iter().collect::<Vec<_>>(), doc, lang)
            }
            ContainerFormat::Struct(nameds, doc) => {
                struct_(w, name, &nameds.iter().collect::<Vec<_>>(), doc, lang)
            }
            ContainerFormat::Enum(variants, doc) => enum_(w, name, variants, doc, lang),
        }
    }
}

impl Emitter<Swift> for Format {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Swift) -> Result<()> {
        match &self {
            Self::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Self::TypeName(qualified_type_name) => {
                write!(
                    w,
                    "{ty}",
                    ty = qualified_type_name
                        .format(|ns| heck::AsUpperCamelCase(ns).to_string(), ".")
                )
            }
            Self::Unit => write!(w, "Void"),
            Self::Bool => write!(w, "Bool"),
            Self::I8 => write!(w, "Int8"),
            Self::I16 => write!(w, "Int16"),
            Self::I32 => write!(w, "Int32"),
            Self::I64 => write!(w, "Int64"),
            Self::I128 => write!(w, "Int128"),
            Self::U8 => write!(w, "UInt8"),
            Self::U16 => write!(w, "UInt16"),
            Self::U32 => write!(w, "UInt32"),
            Self::U64 => write!(w, "UInt64"),
            Self::U128 => write!(w, "UInt128"),
            Self::F32 => write!(w, "Float"),
            Self::F64 => write!(w, "Double"),
            Self::Char => write!(w, "Character"),
            Self::Str => write!(w, "String"),
            Self::Bytes => write!(w, "[UInt8]"),

            Self::Option(format) => {
                format.write(w, lang)?;
                write!(w, "?")
            }
            Self::Seq(format)
            | Self::TupleArray {
                content: format,
                size: _,
            } => {
                write!(w, "[")?;
                format.write(w, lang)?;
                write!(w, "]")
            }
            Self::Set(format) => {
                if !is_hashable(format, lang) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        formatdoc!(
                            "Set element type is not Hashable in Swift; \
                             native tuples and dictionaries do not conform to Hashable"
                        ),
                    ));
                }
                write!(w, "Set<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Self::Map { key, value } => {
                if !is_hashable(key, lang) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        formatdoc!(
                            "Map key type is not Hashable in Swift; \
                             native tuples and dictionaries do not conform to Hashable"
                        ),
                    ));
                }
                write!(w, "[")?;
                key.write(w, lang)?;
                write!(w, ": ")?;
                value.write(w, lang)?;
                write!(w, "]")
            }
            Self::Tuple(formats) => {
                let len = formats.len();
                if len == 1 {
                    // A single-element tuple is just the element itself
                    formats[0].write(w, lang)
                } else {
                    // Always use native Swift tuples
                    write!(w, "(")?;
                    for (i, format) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        format.write(w, lang)?;
                    }
                    write!(w, ")")
                }
            }
        }
    }
}

impl Emitter<Swift> for (&Named<Format>, Usage) {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Swift) -> Result<()> {
        let (Named { name, doc, value }, usage) = self;
        let name = &name.to_lower_camel_case();

        match usage {
            Usage::Field => {
                doc.write(w, lang)?;
                write!(w, "public var {name}: ")?;
                value.write(w, lang)?;
                writeln!(w)
            }
            Usage::IndirectField => {
                doc.write(w, lang)?;
                write!(w, "@Indirect public var {name}: ")?;
                value.write(w, lang)?;
                writeln!(w)
            }
            Usage::Parameter => {
                write!(w, "{name}: ")?;
                value.write(w, lang)
            }
            Usage::Argument => {
                write!(w, "{name}: {name}")
            }
            Usage::Assignment => writeln!(w, "self.{name} = {name}"),
            Usage::Serialize { receiver } => match value {
                Format::Variable(_) => {
                    unreachable!("placeholders should not get this far")
                }
                Format::Tuple(formats) if !lang.encoding.is_bincode() => {
                    push_serializer(w)?;
                    let formats = named(formats, "");
                    for format in formats {
                        (
                            &format,
                            Usage::Serialize {
                                receiver: name.clone(),
                            },
                        )
                            .write(w, lang)?;
                    }
                    pop_serializer(w)
                }
                _ => write_format_serialize(w, value, &format!("{receiver}.{name}")),
            },
            Usage::Deserialize { receiver: _ } => match value {
                Format::Variable(_) => {
                    unreachable!("placeholders should not get this far")
                }
                Format::Tuple(formats) if !lang.encoding.is_bincode() => {
                    push_deserializer(w)?;
                    let formats = named(formats, name);
                    for (i, format) in formats.iter().enumerate() {
                        (
                            format,
                            Usage::Deserialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(w, lang)?;
                    }
                    write!(w, "let {name} = (")?;
                    for (i, format) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "{}", format.name)?;
                    }
                    writeln!(w, ")")?;
                    pop_deserializer(w)
                }
                _ => write_format_deserialize(w, value, name),
            },
        }
    }
}

impl Emitter<Swift> for (&Named<VariantFormat>, Usage) {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Swift) -> Result<()> {
        let (
            Named {
                name,
                doc,
                value: format,
            },
            usage,
        ) = self;
        let name = name.to_lower_camel_case();

        doc.write(w, lang)?;

        match usage {
            Usage::IndirectField => {
                unreachable!("@Indirect is only used for struct fields, not enum variants")
            }
            Usage::Field => match format {
                VariantFormat::Variable(_variable) => {
                    unreachable!("placeholders should not get this far")
                }
                VariantFormat::Unit => {
                    writeln!(w, "case {name}")
                }
                VariantFormat::NewType(format) => {
                    write!(w, "case {name}(")?;
                    format.write(w, lang)?;
                    writeln!(w, ")")
                }
                VariantFormat::Tuple(formats) => {
                    write!(w, "case {name}(")?;
                    for (i, format) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        format.write(w, lang)?;
                    }
                    writeln!(w, ")")
                }
                VariantFormat::Struct(nameds) => {
                    write!(w, "case {name}(")?;
                    for (i, format) in nameds.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        (format, Usage::Parameter).write(w, lang)?;
                    }
                    writeln!(w, ")")
                }
            },
            Usage::Parameter | Usage::Argument | Usage::Assignment => Ok(()),
            Usage::Serialize { receiver: index } => {
                match format {
                    VariantFormat::Variable(_) => {
                        unreachable!("placeholders should not get this far")
                    }
                    VariantFormat::Unit => {
                        writeln!(w, "case .{name}:")?;
                        w.indent();
                        writeln!(w, "try serializer.serialize_variant_index(value: {index})")?;
                        w.unindent();
                    }
                    VariantFormat::NewType(fmt) => {
                        writeln!(w, "case .{name}(let x):")?;
                        w.indent();
                        writeln!(w, "try serializer.serialize_variant_index(value: {index})")?;
                        write_format_serialize(w, fmt, "x")?;
                        w.unindent();
                    }
                    VariantFormat::Tuple(formats) => {
                        write!(w, "case .{name}(")?;
                        for (i, _) in formats.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            write!(w, "let x{i}")?;
                        }
                        writeln!(w, "):")?;
                        w.indent();
                        writeln!(w, "try serializer.serialize_variant_index(value: {index})")?;
                        for (i, fmt) in formats.iter().enumerate() {
                            write_format_serialize(w, fmt, &format!("x{i}"))?;
                        }
                        w.unindent();
                    }
                    VariantFormat::Struct(nameds) => {
                        write!(w, "case .{name}(")?;
                        for (i, named) in nameds.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            let field_name = named.name.to_lower_camel_case();
                            write!(w, "let {field_name}")?;
                        }
                        writeln!(w, "):")?;
                        w.indent();
                        writeln!(w, "try serializer.serialize_variant_index(value: {index})")?;
                        for named in nameds {
                            let field_name = named.name.to_lower_camel_case();
                            write_format_serialize(w, &named.value, &field_name)?;
                        }
                        w.unindent();
                    }
                }
                Ok(())
            }
            Usage::Deserialize { receiver: index } => {
                writeln!(w, "case {index}:")?;
                w.indent();
                match format {
                    VariantFormat::Variable(_) => {
                        unreachable!("placeholders should not get this far")
                    }
                    VariantFormat::Unit => {
                        pop_deserializer(w)?;
                        writeln!(w, "return .{name}")?;
                    }
                    VariantFormat::NewType(fmt) => {
                        write_format_deserialize(w, fmt, "x")?;
                        pop_deserializer(w)?;
                        writeln!(w, "return .{name}(x)")?;
                    }
                    VariantFormat::Tuple(formats) => {
                        for (i, fmt) in formats.iter().enumerate() {
                            write_format_deserialize(w, fmt, &format!("x{i}"))?;
                        }
                        pop_deserializer(w)?;
                        write!(w, "return .{name}(")?;
                        for (i, _) in formats.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            write!(w, "x{i}")?;
                        }
                        writeln!(w, ")")?;
                    }
                    VariantFormat::Struct(nameds) => {
                        for named in nameds {
                            let field_name = named.name.to_lower_camel_case();
                            write_format_deserialize(w, &named.value, &field_name)?;
                        }
                        pop_deserializer(w)?;
                        write!(w, "return .{name}(")?;
                        for (i, named) in nameds.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            let field_name = named.name.to_lower_camel_case();
                            write!(w, "{field_name}: {field_name}")?;
                        }
                        writeln!(w, ")")?;
                    }
                }
                w.unindent();
                Ok(())
            }
        }
    }
}

/// Doc-comment emitter. Writes `///` lines for each comment.
///
impl Emitter<Swift> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: &Swift) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }

        Ok(())
    }
}

/// Emit a `public struct` with `Hashable` conformance (when encoding is
/// active), a memberwise initializer, and encoding-specific
/// `serialize`/`deserialize` methods.
fn struct_<W: IndentWrite>(
    w: &mut W,
    name: &str,
    fields: &[&Named<Format>],
    doc: &Doc,
    lang: &Swift,
) -> Result<()> {
    doc.write(w, lang)?;

    let all_hashable = fields.iter().all(|f| is_hashable(&f.value, lang));
    let all_equatable_auto = fields.iter().all(|f| is_equatable_auto(&f.value, lang));
    let all_can_eq = fields.iter().all(|f| can_use_eq_operator(&f.value, lang));

    if lang.encoding.is_none() {
        write!(w, "public struct {name} ")?;
    } else if all_hashable {
        write!(w, "public struct {name}: Hashable ")?;
    } else if all_equatable_auto || all_can_eq {
        write!(w, "public struct {name}: Equatable ")?;
    } else {
        write!(w, "public struct {name} ")?;
    }
    let mut w = w.block(Newlines::BOTH)?;

    for field in fields {
        let usage = if !lang.encoding.is_none() && needs_indirect(&field.value, name) {
            Usage::IndirectField
        } else {
            Usage::Field
        };
        (*field, usage).write(&mut w, lang)?;
    }

    if !fields.is_empty() || lang.encoding.is_bincode() {
        writeln!(w)?;
    }

    write!(w, "public init(")?;
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(w, ", ")?;
        }
        (*field, Usage::Parameter).write(&mut w, lang)?;
    }
    write!(w, ") ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        for field in fields {
            (*field, Usage::Assignment).write(&mut w, lang)?;
        }
    }

    match lang.encoding {
        Encoding::None => {}
        Encoding::Json => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                push_serializer(&mut w)?;
                for field in fields {
                    (
                        *field,
                        Usage::Serialize {
                            receiver: "self".to_string(),
                        },
                    )
                        .write(&mut w, lang)?;
                }
                pop_serializer(&mut w)?;
            }
            write_json_serialize(&mut w)?;
            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                push_deserializer(&mut w)?;
                for field in fields {
                    (
                        *field,
                        Usage::Deserialize {
                            receiver: "self".to_string(),
                        },
                    )
                        .write(&mut w, lang)?;
                }
                pop_deserializer(&mut w)?;
                write!(w, "return {name}(")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    (*field, Usage::Argument).write(&mut w, lang)?;
                }
                writeln!(w, ")")?;
            }
            write_json_deserialize(&mut w, name)?;
        }
        Encoding::Bincode => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                push_serializer(&mut w)?;
                for field in fields {
                    (
                        *field,
                        Usage::Serialize {
                            receiver: "self".to_string(),
                        },
                    )
                        .write(&mut w, lang)?;
                }
                pop_serializer(&mut w)?;
            }
            write_bincode_serialize(&mut w)?;
            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                push_deserializer(&mut w)?;
                for field in fields {
                    (
                        *field,
                        Usage::Deserialize {
                            receiver: "self".to_string(),
                        },
                    )
                        .write(&mut w, lang)?;
                }
                pop_deserializer(&mut w)?;
                write!(w, "return {name}(")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    (*field, Usage::Argument).write(&mut w, lang)?;
                }
                writeln!(w, ")")?;
            }
            write_bincode_deserialize(&mut w, name)?;
        }
    }

    // Emit manual Equatable implementation when auto-synthesis is blocked
    // (because one or more fields are native tuples) but == can still be
    // written field-by-field using Swift's built-in tuple == operator.
    if !lang.encoding.is_none() && !all_hashable && !all_equatable_auto && all_can_eq {
        write_struct_eq(&mut w, name, fields)?;
    }

    Ok(())
}

/// Emit `public static func == (lhs: Name, rhs: Name) -> Bool { ... }` for
/// structs that cannot use `Equatable` auto-synthesis because one or more
/// fields are native tuples (which don't conform to the `Equatable` protocol)
/// but whose fields all support the `==` operator.
fn write_struct_eq<W: IndentWrite>(w: &mut W, name: &str, fields: &[&Named<Format>]) -> Result<()> {
    writeln!(w)?;
    write!(
        w,
        "public static func == (lhs: {name}, rhs: {name}) -> Bool "
    )?;
    let mut w = w.block(Newlines::BOTH)?;
    if fields.is_empty() {
        writeln!(w, "return true")?;
    } else {
        write!(w, "return ")?;
        for (i, field) in fields.iter().enumerate() {
            let fname = field.name.to_lower_camel_case();
            if i > 0 {
                writeln!(w)?;
                write!(w, "    && ")?;
            }
            write!(w, "lhs.{fname} == rhs.{fname}")?;
        }
        writeln!(w)?;
    }
    Ok(())
}

/// Emit an `indirect public enum` with `Hashable` conformance (when encoding
/// is active), case variants, and encoding-specific `serialize`/`deserialize`
/// methods.
fn enum_<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    lang: &Swift,
) -> Result<()> {
    doc.write(w, lang)?;

    let all_hashable = variants
        .values()
        .all(|v| variant_is_hashable(&v.value, lang));
    let all_equatable_auto = variants
        .values()
        .all(|v| variant_is_equatable_auto(&v.value, lang));
    let all_can_eq = variants
        .values()
        .all(|v| variant_can_use_eq_operator(&v.value, lang));

    if lang.encoding.is_none() {
        write!(w, "indirect public enum {name} ")?;
    } else if all_hashable {
        write!(w, "indirect public enum {name}: Hashable ")?;
    } else if all_equatable_auto || all_can_eq {
        write!(w, "indirect public enum {name}: Equatable ")?;
    } else {
        write!(w, "indirect public enum {name} ")?;
    }
    let mut w = w.block(Newlines::BOTH)?;

    let variants = variants.values().collect::<Vec<_>>();

    for format in &variants {
        (*format, Usage::Field).write(&mut w, lang)?;
    }

    match lang.encoding {
        Encoding::None => {}
        Encoding::Json => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                push_serializer(&mut w)?;
                write!(w, "switch self ")?;
                {
                    let mut w = w.block(Newlines::BOTH)?;
                    w.unindent();
                    for (i, variant) in variants.iter().enumerate() {
                        (
                            &variant.without_docs(),
                            Usage::Serialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(&mut w, lang)?;
                    }
                    w.indent();
                }
                pop_serializer(&mut w)?;
            }
            write_json_serialize(&mut w)?;

            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                writeln!(
                    w,
                    "let index = try deserializer.deserialize_variant_index()"
                )?;
                push_deserializer(&mut w)?;
                write!(w, "switch index ")?;
                {
                    let mut w = w.block(Newlines::BOTH)?;
                    w.unindent();
                    for (i, variant) in variants.iter().enumerate() {
                        (
                            &variant.without_docs(),
                            Usage::Deserialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(&mut w, lang)?;
                    }
                    writeln!(
                        w,
                        r#"default: throw DeserializationError.invalidInput(issue: "Unknown variant index for {name}: \(index)")"#
                    )?;
                    w.indent();
                }
            }
            write_json_deserialize(&mut w, name)?;
        }
        Encoding::Bincode => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                push_serializer(&mut w)?;
                write!(w, "switch self ")?;
                {
                    let mut w = w.block(Newlines::BOTH)?;
                    w.unindent();
                    for (i, variant) in variants.iter().enumerate() {
                        (
                            &variant.without_docs(),
                            Usage::Serialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(&mut w, lang)?;
                    }
                    w.indent();
                }
                pop_serializer(&mut w)?;
            }
            write_bincode_serialize(&mut w)?;

            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                writeln!(
                    w,
                    "let index = try deserializer.deserialize_variant_index()"
                )?;
                push_deserializer(&mut w)?;
                write!(w, "switch index ")?;
                {
                    let mut w = w.block(Newlines::BOTH)?;
                    w.unindent();
                    for (i, variant) in variants.iter().enumerate() {
                        (
                            &variant.without_docs(),
                            Usage::Deserialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(&mut w, lang)?;
                    }
                    writeln!(
                        w,
                        r#"default: throw DeserializationError.invalidInput(issue: "Unknown variant index for {name}: \(index)")"#
                    )?;
                    w.indent();
                }
            }
            write_bincode_deserialize(&mut w, name)?;
        }
    }

    // Emit manual Equatable implementation when auto-synthesis is blocked.
    if !lang.encoding.is_none() && !all_hashable && !all_equatable_auto && all_can_eq {
        write_enum_eq(&mut w, name, &variants)?;
    }

    Ok(())
}

/// Emit `public static func == (lhs: Name, rhs: Name) -> Bool { ... }` for
/// enums that cannot use `Equatable` auto-synthesis because one or more
/// variant associated values are native tuples.
fn write_enum_eq<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &[&Named<VariantFormat>],
) -> Result<()> {
    writeln!(w)?;
    write!(
        w,
        "public static func == (lhs: {name}, rhs: {name}) -> Bool "
    )?;
    let mut w = w.block(Newlines::BOTH)?;
    write!(w, "switch (lhs, rhs) ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        w.unindent();
        for variant in variants {
            let vname = variant.name.to_lower_camel_case();
            match &variant.value {
                VariantFormat::Variable(_) => {
                    unreachable!("placeholders should not get this far")
                }
                VariantFormat::Unit => {
                    writeln!(w, "case (.{vname}, .{vname}): return true")?;
                }
                VariantFormat::NewType(_) => {
                    writeln!(w, "case (.{vname}(let l), .{vname}(let r)): return l == r")?;
                }
                VariantFormat::Tuple(formats) => {
                    write!(w, "case (.{vname}(")?;
                    for i in 0..formats.len() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "let l{i}")?;
                    }
                    write!(w, "), .{vname}(")?;
                    for i in 0..formats.len() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "let r{i}")?;
                    }
                    write!(w, ")): return ")?;
                    for i in 0..formats.len() {
                        if i > 0 {
                            write!(w, " && ")?;
                        }
                        write!(w, "l{i} == r{i}")?;
                    }
                    writeln!(w)?;
                }
                VariantFormat::Struct(nameds) => {
                    write!(w, "case (.{vname}(")?;
                    for i in 0..nameds.len() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "let l{i}")?;
                    }
                    write!(w, "), .{vname}(")?;
                    for i in 0..nameds.len() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "let r{i}")?;
                    }
                    write!(w, ")): return ")?;
                    for i in 0..nameds.len() {
                        if i > 0 {
                            write!(w, " && ")?;
                        }
                        write!(w, "l{i} == r{i}")?;
                    }
                    writeln!(w)?;
                }
            }
        }
        writeln!(w, "default: return false")?;
        w.indent();
    }
    Ok(())
}

fn write_bincode_serialize<W: IndentWrite>(w: &mut W) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r"
        public func bincodeSerialize() throws -> [UInt8] {{
            let serializer = BincodeSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }}
        "
    )
}

fn write_bincode_deserialize<W: IndentWrite>(w: &mut W, name: &str) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r#"
        public static func bincodeDeserialize(input: [UInt8]) throws -> {name} {{
            let deserializer = BincodeDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {{
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }}
            return obj
        }}
        "#
    )
}

fn write_json_serialize<W: IndentWrite>(w: &mut W) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r"
        public func jsonSerialize() throws -> [UInt8] {{
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }}
        "
    )
}

fn write_json_deserialize<W: IndentWrite>(w: &mut W, name: &str) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r#"
        public static func jsonDeserialize(input: [UInt8]) throws -> {name} {{
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {{
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }}
            return obj
        }}
        "#
    )
}

fn write_format_serialize<W: IndentWrite>(
    w: &mut W,
    format: &Format,
    value_expr: &str,
) -> Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(_) => {
            writeln!(w, "try {value_expr}.serialize(serializer: serializer)")
        }
        Format::Option(inner) => {
            write!(
                w,
                "try serializeOption(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block(Newlines::CLOSE)?;
                writeln!(w, " value, serializer in")?;
                write_format_serialize(&mut w, inner, "value")
            }
        }
        Format::Seq(inner) => {
            write!(
                w,
                "try serializeArray(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block(Newlines::CLOSE)?;
                writeln!(w, " item, serializer in")?;
                write_format_serialize(&mut w, inner, "item")
            }
        }
        Format::Set(inner) => {
            write!(
                w,
                "try serializeSet(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block(Newlines::CLOSE)?;
                writeln!(w, " item, serializer in")?;
                write_format_serialize(&mut w, inner, "item")
            }
        }
        Format::Map { key, value } => {
            write!(
                w,
                "try serializeMap(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block(Newlines::CLOSE)?;
                writeln!(w, " key, value, serializer in")?;
                write_format_serialize(&mut w, key, "key")?;
                write_format_serialize(&mut w, value, "value")
            }
        }
        Format::Tuple(formats) => {
            for (i, fmt) in formats.iter().enumerate() {
                write_format_serialize(w, fmt, &format!("{value_expr}.{i}"))?;
            }
            Ok(())
        }
        Format::TupleArray { content, .. } => {
            write!(
                w,
                "try serializeTupleArray(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block(Newlines::CLOSE)?;
                writeln!(w, " item, serializer in")?;
                write_format_serialize(&mut w, content, "item")
            }
        }
        primitive => {
            let t = format!("{primitive:?}").to_lowercase();
            writeln!(w, "try serializer.serialize_{t}(value: {value_expr})")
        }
    }
}

fn write_format_deserialize<W: IndentWrite>(w: &mut W, format: &Format, var: &str) -> Result<()> {
    match format {
        Format::Tuple(formats) if formats.len() > 1 => {
            for (i, fmt) in formats.iter().enumerate() {
                write_format_deserialize(w, fmt, &format!("{var}Field{i}"))?;
            }
            write!(w, "let {var} = (")?;
            for i in 0..formats.len() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write!(w, "{var}Field{i}")?;
            }
            writeln!(w, ")")
        }
        _ => {
            write!(w, "let {var} = ")?;
            write_deserialize_expr(w, format)?;
            writeln!(w)
        }
    }
}

/// Writes a deserialization expression (no `let` prefix, no trailing newline).
fn write_deserialize_expr<W: IndentWrite>(w: &mut W, format: &Format) -> Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qtn) => {
            let type_name = &qtn.format(|ns| heck::AsUpperCamelCase(ns).to_string(), ".");
            write!(w, "try {type_name}.deserialize(deserializer: deserializer)")
        }
        Format::Option(inner) => {
            writeln!(
                w,
                "try deserializeOption(deserializer: deserializer) {{ deserializer in"
            )?;
            w.indent();
            write_deserialize_expr(w, inner)?;
            writeln!(w)?;
            w.unindent();
            write!(w, "}}")
        }
        Format::Seq(inner) => {
            writeln!(
                w,
                "try deserializeArray(deserializer: deserializer) {{ deserializer in"
            )?;
            w.indent();
            write_deserialize_expr(w, inner)?;
            writeln!(w)?;
            w.unindent();
            write!(w, "}}")
        }
        Format::Set(inner) => {
            writeln!(
                w,
                "try deserializeSet(deserializer: deserializer) {{ deserializer in"
            )?;
            w.indent();
            write_deserialize_expr(w, inner)?;
            writeln!(w)?;
            w.unindent();
            write!(w, "}}")
        }
        Format::Map { key, value } => {
            writeln!(
                w,
                "try deserializeMap(deserializer: deserializer) {{ deserializer in"
            )?;
            w.indent();
            write_format_deserialize(w, key, "key")?;
            write_format_deserialize(w, value, "value")?;
            writeln!(w, "return (key, value)")?;
            w.unindent();
            write!(w, "}}")
        }
        Format::Tuple(formats) if formats.len() == 1 => write_deserialize_expr(w, &formats[0]),
        Format::Tuple(formats) => {
            // Multi-element native tuple: emit let-bindings then a tuple literal.
            // Uses an explicit `return` because this appears inside a multi-statement
            // closure body (e.g. deserializeOption { … }) where Swift requires it.
            for (i, fmt) in formats.iter().enumerate() {
                write_format_deserialize(w, fmt, &format!("field{i}"))?;
            }
            write!(w, "return (")?;
            for i in 0..formats.len() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write!(w, "field{i}")?;
            }
            write!(w, ")")
        }
        Format::TupleArray { content, size } => {
            writeln!(
                w,
                "try deserializeTupleArray(deserializer: deserializer, size: {size}) {{ deserializer in"
            )?;
            w.indent();
            write_deserialize_expr(w, content)?;
            writeln!(w)?;
            w.unindent();
            write!(w, "}}")
        }
        primitive => {
            let t = format!("{primitive:?}").to_lowercase();
            write!(w, "try deserializer.deserialize_{t}()")
        }
    }
}

fn push_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try serializer.increase_container_depth()")
}

fn pop_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try serializer.decrease_container_depth()")
}

fn push_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try deserializer.increase_container_depth()")
}

fn pop_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try deserializer.decrease_container_depth()")
}

fn named<Format: Clone>(formats: &[Format], prefix: &str) -> Vec<Named<Format>> {
    formats
        .iter()
        .enumerate()
        .map(|(i, f)| Named::new(f, format!("{prefix}{i}")))
        .collect()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bincode;
#[cfg(test)]
mod tests_json;
