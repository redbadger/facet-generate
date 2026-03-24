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
//! | [`Doc`] | `///` doc comments |
//! | `(Named<VariantFormat>, Usage)` | An enum case declaration |
//!
//! # Swift type mapping
//!
//! The [`Format`] emitter maps Rust/reflection types to Swift equivalents —
//! for example `I32` → `Int32`, `Seq(T)` → `[T]`, `Option(T)` → `T?`,
//! `Map(K,V)` → `[K: V]`, tuples of any size → native `(A, B)` (always).
//!
//! # Encoding-dependent output
//!
//! The [`Swift`] language tag carries a list of [`EmitterPlugin`]s. All
//! encoding-specific behaviour is delegated to those plugins — the emitter
//! itself contains no encoding checks. For example:
//!
//! - `BincodePlugin` supplies `serialize` / `deserialize` methods and
//!   `bincodeSerialize` / `bincodeDeserialize` wrappers.
//! - `JsonPlugin` supplies the same `serialize` / `deserialize` methods and
//!   `jsonSerialize` / `jsonDeserialize` wrappers.
//! - With no plugins (`Encoding::None`), only plain type declarations are
//!   emitted.
//!
//! # Feature helpers
//!
//! The encoding-specific feature helpers (`serializeArray`, `serializeOption`,
//! etc.) are inlined in `BincodePlugin` and `JsonPlugin`
//! (`generation/bincode/swift.rs` and `generation/json/swift.rs`).
//! They are emitted via the [`EmitterPlugin::module_helpers`] hook when the
//! corresponding [`Feature`] flag is set by
//! [`CodeGeneratorConfig::update_from`].
//!
//! # Hashable / Equatable
//!
//! Types whose fields are all [`Hashable`](https://developer.apple.com/documentation/swift/hashable)
//! will declare `: Hashable` conformance so they can serve as `Set` elements
//! or `Dictionary` keys. A generation-time error is raised if a non-`Hashable`
//! type (native tuple or `[K:V]` dictionary) is used directly as a `Set`
//! element or `Map` key.

#![allow(clippy::too_many_lines)]
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Result, Write},
    sync::Arc,
};

use heck::ToLowerCamelCase as _;
use indoc::formatdoc;

use heck::ToUpperCamelCase as _;

#[cfg(test)]
use crate::generation::CodeGeneratorConfig;
use crate::{
    Registry,
    generation::{
        Container, Emitter,
        indent::{IndentWrite, Newlines},
        module::Module,
        plugin::{EmitContext, EmitterPlugin},
        swift::generator::{compute_equatable_types, compute_hashable_types},
    },
    reflection::format::{ContainerFormat, Doc, Format, Named, Namespace, VariantFormat},
};

/// Language tag for Swift code generation.
///
/// Carries the active `Encoding` and the sets of type names (within the
/// current module) that are known to be able to synthesize `Hashable` and
/// `Equatable` conformance respectively. Both sets are computed by a
/// preprocessing pass — see
/// [`SwiftCodeGenerator`](crate::generation::swift::generator::SwiftCodeGenerator).
///
/// The plugin list is built in [`new`](Self::new) from the config encoding.
/// Eventually, plugins will be supplied externally and `encoding` will be
/// removed.
#[derive(Debug, Clone)]
pub struct Swift {
    /// Type names (root-namespace) that can synthesize `Hashable` conformance.
    pub(crate) hashable_types: BTreeSet<String>,
    /// Type names (root-namespace) that can synthesize or manually implement
    /// `Equatable` conformance.
    pub(crate) equatable_types: BTreeSet<String>,
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Self>>>,
}

impl Swift {
    /// Create a Swift language tag with computed type sets and an empty plugin
    /// list. Plugins are added by the code generator (which holds the encoding)
    /// or explicitly via [`with_plugin`](Self::with_plugin).
    ///
    /// The `hashable_types` and `equatable_types` sets are computed from the
    /// registry via fixed-point analysis and are unrelated to plugin selection.
    #[must_use]
    pub fn new(registry: &Registry) -> Self {
        Self {
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

    /// Add a plugin to this language tag (builder-style).
    #[must_use]
    pub fn with_plugin(mut self, plugin: Arc<dyn EmitterPlugin<Self>>) -> Self {
        self.plugins.push(plugin);
        self
    }
}

#[cfg(test)]
impl crate::generation::plugin::FromEncoding for Swift {
    fn from_encoding(
        encoding: crate::generation::Encoding,
        _config: &CodeGeneratorConfig,
        registry: &crate::Registry,
    ) -> Self {
        use crate::generation::{Encoding, bincode::BincodePlugin, json::JsonPlugin};
        let plugins: Vec<Arc<dyn EmitterPlugin<Self>>> = match encoding {
            Encoding::Bincode => vec![Arc::new(BincodePlugin)],
            Encoding::Json => vec![Arc::new(JsonPlugin)],
            Encoding::None => vec![],
        };
        Self {
            hashable_types: compute_hashable_types(registry),
            equatable_types: compute_equatable_types(registry),
            plugins,
        }
    }
}

// ---------------------------------------------------------------------------
// Hashability helpers
// ---------------------------------------------------------------------------

/// Returns `true` if the Swift type produced by `format` can conform to
/// `Hashable`.
pub fn is_hashable(format: &Format, lang: &Swift) -> bool {
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
        | Format::Set(inner)
        | Format::Seq(inner)
        | Format::TupleArray { content: inner, .. } => is_hashable(inner, lang),

        // A 1-element tuple is transparent; multi-element native tuples are not Hashable.
        Format::Tuple(formats) => {
            formats.len() == 1 && is_hashable(&formats[0], lang)
        }
    }
}

fn variant_is_hashable(format: &VariantFormat, lang: &Swift) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => is_hashable(fmt, lang),
        VariantFormat::Tuple(fmts) => fmts.iter().all(|f| is_hashable(f, lang)),
        VariantFormat::Struct(nameds) => nameds.iter().all(|n| is_hashable(&n.value, lang)),
    }
}

// ---------------------------------------------------------------------------
// Usage — field / case rendering context
// ---------------------------------------------------------------------------

enum Usage {
    Field,
    /// Like `Field` but prefixed with `@Indirect` for recursive struct fields
    /// that would otherwise create an infinite-size value type.
    IndirectField,
    Parameter,
    Assignment,
}

// ---------------------------------------------------------------------------
// Recursion / indirection helpers
// ---------------------------------------------------------------------------

/// Returns `true` if a struct field of this format would create an
/// infinite-size value-type cycle back to the containing struct named
/// `struct_name`.
fn needs_indirect(format: &Format, struct_name: &str) -> bool {
    match format {
        Format::TypeName(qtn) => qtn.name == struct_name,
        Format::Option(inner) => needs_indirect(inner, struct_name),
        Format::Tuple(formats) => formats.iter().any(|f| needs_indirect(f, struct_name)),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Equatable helpers
// ---------------------------------------------------------------------------

fn is_equatable_auto(format: &Format, lang: &Swift) -> bool {
    match format {
        Format::TypeName(qtn) => match &qtn.namespace {
            Namespace::Root => lang.equatable_types.contains(&qtn.name),
            Namespace::Named(_) => true,
        },
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
        Format::Map { key, value } => {
            is_equatable_auto(key, lang) && is_equatable_auto(value, lang)
        }
        Format::Tuple(formats) => formats.len() == 1 && is_equatable_auto(&formats[0], lang),
    }
}

fn can_use_eq_operator(format: &Format, lang: &Swift) -> bool {
    match format {
        Format::Tuple(formats) if formats.len() > 1 => {
            formats.iter().all(|f| is_equatable_auto(f, lang))
        }
        _ => is_equatable_auto(format, lang),
    }
}

fn variant_is_equatable_auto(format: &VariantFormat, lang: &Swift) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => is_equatable_auto(fmt, lang),
        VariantFormat::Tuple(formats) => formats.iter().all(|f| is_equatable_auto(f, lang)),
        VariantFormat::Struct(nameds) => nameds.iter().all(|n| is_equatable_auto(&n.value, lang)),
    }
}

fn variant_can_use_eq_operator(format: &VariantFormat, lang: &Swift) -> bool {
    match format {
        VariantFormat::Variable(_) => false,
        VariantFormat::Unit => true,
        VariantFormat::NewType(fmt) => can_use_eq_operator(fmt, lang),
        VariantFormat::Tuple(formats) => formats.iter().all(|f| can_use_eq_operator(f, lang)),
        VariantFormat::Struct(nameds) => nameds.iter().all(|n| can_use_eq_operator(&n.value, lang)),
    }
}

// ---------------------------------------------------------------------------
// Module emitter
// ---------------------------------------------------------------------------

impl Emitter<Swift> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Swift) -> Result<()> {
        let mut imports = vec![];

        // Encoding-independent base imports (external namespaces).
        for ns in self.config().external_definitions.keys() {
            imports.push(ns.to_upper_camel_case());
        }

        // Plugin imports (e.g. `import Serde`).
        for plugin in lang.plugins() {
            imports.extend(plugin.imports(self.config()));
        }

        imports.sort();
        imports.dedup();
        for import in &imports {
            writeln!(w, "import {import}")?;
        }

        // Plugin module helpers (feature snippets).
        for plugin in lang.plugins() {
            plugin.module_helpers(w, self.config())?;
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Container emitter
// ---------------------------------------------------------------------------

impl Emitter<Swift> for Container<'_> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Swift) -> Result<()> {
        let Container { format, .. } = self;
        match format {
            ContainerFormat::UnitStruct(doc) => struct_(w, self, &[], doc, lang),
            ContainerFormat::NewTypeStruct(format, doc) => struct_(
                w,
                self,
                &[&Named::new(format, "value".to_string())],
                doc,
                lang,
            ),
            ContainerFormat::TupleStruct(formats, doc) => {
                let formats = named(formats, "field");
                struct_(w, self, &formats.iter().collect::<Vec<_>>(), doc, lang)
            }
            ContainerFormat::Struct(nameds, doc) => {
                struct_(w, self, &nameds.iter().collect::<Vec<_>>(), doc, lang)
            }
            ContainerFormat::Enum(variants, doc) => enum_(w, self, variants, doc, lang),
        }
    }
}

// ---------------------------------------------------------------------------
// Format emitter
// ---------------------------------------------------------------------------

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
                    formats[0].write(w, lang)
                } else {
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

// ---------------------------------------------------------------------------
// Named<Format> emitter — field / parameter / argument / assignment
// ---------------------------------------------------------------------------

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
            Usage::Assignment => writeln!(w, "self.{name} = {name}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Named<VariantFormat> emitter — case declarations only
// ---------------------------------------------------------------------------

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
                VariantFormat::Unit => writeln!(w, "case {name}"),
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
            Usage::Parameter | Usage::Assignment => Ok(()),
        }
    }
}

// ---------------------------------------------------------------------------
// Doc emitter
// ---------------------------------------------------------------------------

impl Emitter<Swift> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: &Swift) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// struct_ — emits a public struct
// ---------------------------------------------------------------------------

/// Emit a `public struct` with optional `Hashable` / `Equatable` conformance,
/// a memberwise initializer, and (via plugins) `serialize` / `deserialize`
/// methods.
fn struct_<W: IndentWrite>(
    w: &mut W,
    container: &Container<'_>,
    fields: &[&Named<Format>],
    doc: &Doc,
    lang: &Swift,
) -> Result<()> {
    let name = &container.name.name;

    doc.write(w, lang)?;

    let has_plugins = !lang.plugins().is_empty();
    let all_hashable = fields.iter().all(|f| is_hashable(&f.value, lang));
    let all_equatable_auto = fields.iter().all(|f| is_equatable_auto(&f.value, lang));
    let all_can_eq = fields.iter().all(|f| can_use_eq_operator(&f.value, lang));

    if !has_plugins {
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
        let usage = if has_plugins && needs_indirect(&field.value, name) {
            Usage::IndirectField
        } else {
            Usage::Field
        };
        (*field, usage).write(&mut w, lang)?;
    }

    if !fields.is_empty() {
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

    // Plugin type bodies (serialize / deserialize methods).
    let ctx = EmitContext::top_level(container);
    for plugin in lang.plugins() {
        plugin.type_body(&mut w as &mut dyn IndentWrite, &ctx)?;
    }

    // Emit manual Equatable implementation when auto-synthesis is blocked
    // (because one or more fields are native tuples) but == can still be
    // written field-by-field using Swift's built-in tuple == operator.
    if has_plugins && !all_hashable && !all_equatable_auto && all_can_eq {
        write_struct_eq(&mut w, name, fields)?;
    }

    Ok(())
}

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

// ---------------------------------------------------------------------------
// enum_ — emits an indirect public enum
// ---------------------------------------------------------------------------

/// Emit an `indirect public enum` with optional `Hashable` / `Equatable`
/// conformance, case declarations, and (via plugins) `serialize` /
/// `deserialize` methods.
fn enum_<W: IndentWrite>(
    w: &mut W,
    container: &Container<'_>,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    lang: &Swift,
) -> Result<()> {
    let name = &container.name.name;

    doc.write(w, lang)?;

    let has_plugins = !lang.plugins().is_empty();
    let all_hashable = variants
        .values()
        .all(|v| variant_is_hashable(&v.value, lang));
    let all_equatable_auto = variants
        .values()
        .all(|v| variant_is_equatable_auto(&v.value, lang));
    let all_can_eq = variants
        .values()
        .all(|v| variant_can_use_eq_operator(&v.value, lang));

    if !has_plugins {
        write!(w, "indirect public enum {name} ")?;
    } else if all_hashable {
        write!(w, "indirect public enum {name}: Hashable ")?;
    } else if all_equatable_auto || all_can_eq {
        write!(w, "indirect public enum {name}: Equatable ")?;
    } else {
        write!(w, "indirect public enum {name} ")?;
    }
    let mut w = w.block(Newlines::BOTH)?;

    for variant in variants.values() {
        (variant, Usage::Field).write(&mut w, lang)?;
    }

    // Plugin type bodies (serialize / deserialize methods).
    let ctx = EmitContext::top_level(container);
    for plugin in lang.plugins() {
        plugin.type_body(&mut w as &mut dyn IndentWrite, &ctx)?;
    }

    // Emit manual Equatable implementation when auto-synthesis is blocked.
    if has_plugins && !all_hashable && !all_equatable_auto && all_can_eq {
        write_enum_eq(&mut w, name, &variants.values().collect::<Vec<_>>())?;
    }

    Ok(())
}

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
            let variant_name = variant.name.to_lower_camel_case();
            match &variant.value {
                VariantFormat::Unit => {
                    writeln!(w, "case (.{variant_name}, .{variant_name}): return true")?;
                }
                VariantFormat::NewType(_) => {
                    writeln!(
                        w,
                        "case (.{variant_name}(let l), .{variant_name}(let r)): return l == r"
                    )?;
                }
                VariantFormat::Tuple(formats) => {
                    write!(w, "case (.{variant_name}(")?;
                    for (i, _) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "let l{i}")?;
                    }
                    write!(w, "), .{variant_name}(")?;
                    for (i, _) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "let r{i}")?;
                    }
                    write!(w, ")): return ")?;
                    for (i, _) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, " && ")?;
                        }
                        write!(w, "l{i} == r{i}")?;
                    }
                    writeln!(w)?;
                }
                VariantFormat::Struct(nameds) => {
                    write!(w, "case (.{variant_name}(")?;
                    for (i, n) in nameds.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        let fname = n.name.to_lower_camel_case();
                        write!(w, "{fname}: let l{i}")?;
                    }
                    write!(w, "), .{variant_name}(")?;
                    for (i, n) in nameds.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        let fname = n.name.to_lower_camel_case();
                        write!(w, "{fname}: let r{i}")?;
                    }
                    write!(w, ")): return ")?;
                    for (i, _) in nameds.iter().enumerate() {
                        if i > 0 {
                            write!(w, " && ")?;
                        }
                        write!(w, "l{i} == r{i}")?;
                    }
                    writeln!(w)?;
                }
                VariantFormat::Variable(_) => {
                    unreachable!("placeholders should not get this far")
                }
            }
        }
        writeln!(w, "default: return false")?;
        w.indent();
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn named<Format: Clone>(formats: &[Format], prefix: &str) -> Vec<Named<Format>> {
    formats
        .iter()
        .enumerate()
        .map(|(i, f)| Named::new(f, format!("{prefix}{i}")))
        .collect()
}

#[cfg(test)]
#[allow(unused_imports)]
pub use crate::generation::Encoding;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bincode;
#[cfg(test)]
mod tests_json;
