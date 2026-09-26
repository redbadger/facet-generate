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
//! - `JsonPlugin` adds a `Codable` conformance (through
//!   [`EmitterPlugin::type_conformances`], which the emitter appends after
//!   `Hashable` / `Equatable`), the coding members `serde_json`'s format
//!   needs, and `jsonSerialize` / `jsonDeserialize` wrappers.
//! - With no plugins, only plain type declarations are emitted.
//!
//! # Feature helpers
//!
//! The bincode feature helpers (`serializeArray`, `serializeOption`, etc.) are
//! inlined in `BincodePlugin` (`generation/bincode/swift.rs`).
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
use super::naming::builtin;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    io::{self, Result, Write},
    sync::Arc,
};

use heck::ToLowerCamelCase as _;
use indoc::formatdoc;

use heck::ToUpperCamelCase as _;

use crate::generation::{CodeGeneratorConfig, Feature};
use crate::{
    Registry,
    generation::{
        Container, Emitter,
        indent::{IndentWrite, Newlines},
        module::Module,
        plugin::{EmitContext, EmitterPlugin},
        swift::{
            conformance::{self, Conformance},
            generator::SwiftCodeGenerator,
        },
    },
    reflection::format::{
        ContainerFormat, Doc, Format, Named, Namespace, QualifiedTypeName, VariantFormat,
    },
};

/// Language tag for Swift code generation.
///
/// Carries the active `Encoding` and the sets of type names that conform to
/// `Hashable` and `Equatable` respectively, in the spelling the emitter sees.
/// Both sets are decided by a preprocessing pass over the whole registry when
/// the [`Installer`](crate::generation::swift::Installer) generates the
/// module, and over the module's own registry otherwise.
///
/// The plugin list is built in [`new`](Self::new) from the config encoding.
/// Eventually, plugins will be supplied externally and `encoding` will be
/// removed.
#[derive(Debug, Clone)]
pub struct Swift {
    /// The code-generator configuration for the current module.
    pub(crate) config: CodeGeneratorConfig,
    /// Qualified names of every type whose conformance was decided; any
    /// other type (one from an external package, or absent from the
    /// registry) is assumed to conform to both protocols.
    pub(crate) decided_types: BTreeSet<QualifiedTypeName>,
    /// Decided types that conform to `Hashable`.
    pub(crate) hashable_types: BTreeSet<QualifiedTypeName>,
    /// Decided types that synthesize or manually implement `Equatable`
    /// conformance.
    pub(crate) equatable_types: BTreeSet<QualifiedTypeName>,
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Self>>>,
}

impl Swift {
    /// Create a Swift language tag with computed type sets and an empty plugin
    /// list. Plugins are added by the code generator (which holds the encoding)
    /// or explicitly via [`with_plugin`](Self::with_plugin).
    ///
    /// The `hashable_types` and `equatable_types` sets are computed from
    /// `registry` via fixed-point analysis and are unrelated to plugin
    /// selection. A type absent from `registry`, or in the namespace of one of
    /// the config's external packages, is assumed to conform to both.
    #[must_use]
    pub fn new(config: &CodeGeneratorConfig, registry: &Registry) -> Self {
        Self::decided_by(
            config,
            &Conformance::of(registry, &config.external_packages),
        )
    }

    /// Create a Swift language tag whose type sets come from `conformance`,
    /// decided over a registry that may be wider than this module's (the
    /// whole one, when the installer generates the module), respelled the way
    /// the generator rewrites this module's type references.
    pub(crate) fn decided_by(config: &CodeGeneratorConfig, conformance: &Conformance) -> Self {
        let respell = |set: &BTreeSet<QualifiedTypeName>| -> BTreeSet<QualifiedTypeName> {
            set.iter()
                .map(|name| SwiftCodeGenerator::requalify(config, name))
                .collect()
        };
        Self {
            config: config.clone(),
            decided_types: respell(&conformance.decided),
            hashable_types: respell(&conformance.hashable),
            equatable_types: respell(&conformance.equatable),
            plugins: vec![],
        }
    }

    /// Whether the type `name` (in the emitter's spelling) conforms to
    /// `Hashable`.
    fn is_hashable_type(&self, name: &QualifiedTypeName) -> bool {
        !self.decided_types.contains(name) || self.hashable_types.contains(name)
    }

    /// Whether the type `name` (in the emitter's spelling) conforms to
    /// `Equatable`.
    fn is_equatable_type(&self, name: &QualifiedTypeName) -> bool {
        !self.decided_types.contains(name) || self.equatable_types.contains(name)
    }

    /// Access the code-generator configuration.
    #[must_use]
    pub const fn config(&self) -> &CodeGeneratorConfig {
        &self.config
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

// ---------------------------------------------------------------------------
// Hashability helpers
// ---------------------------------------------------------------------------

/// Returns `true` if the Swift type produced by `format` can conform to
/// `Hashable`.
pub fn is_hashable(format: &Format, lang: &Swift) -> bool {
    conformance::is_hashable(format, &|name| lang.is_hashable_type(name))
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
pub(crate) fn needs_indirect(format: &Format, struct_name: &str) -> bool {
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
    conformance::is_equatable_auto(format, &|name| lang.is_equatable_type(name))
}

// ---------------------------------------------------------------------------
// Module emitter
// ---------------------------------------------------------------------------

/// Write the module header — the `import` lines for external namespaces and
/// for every plugin, merged with `extra_imports` (bare Swift module names).
///
/// Shared by [`Module`]'s emitter and by the generator when it renders a
/// plugin's companion file, which needs the same header but none of the module
/// helpers (they are declared once, in the module file).
///
/// # Errors
///
/// Returns an error if writing to `w` fails.
pub(crate) fn write_module_header<W: IndentWrite>(
    w: &mut W,
    config: &CodeGeneratorConfig,
    lang: &Swift,
    extra_imports: &[String],
) -> Result<()> {
    let mut imports = base_imports(config);

    // Plugin imports (e.g. `import Serde`).
    for plugin in lang.plugins() {
        imports.extend(plugin.imports(config));
    }

    imports.extend(extra_imports.iter().cloned());

    imports.sort();
    imports.dedup();
    for import in &imports {
        writeln!(w, "import {import}")?;
    }

    Ok(())
}

/// The modules a module imports whatever its plugins: each external namespace,
/// and `Foundation` for `UUID` (#191).
pub(crate) fn base_imports(config: &CodeGeneratorConfig) -> Vec<String> {
    let mut imports: Vec<String> = config
        .external_definitions
        .keys()
        .map(|ns| ns.to_upper_camel_case())
        .collect();
    if config.features.contains(&Feature::Uuid) {
        imports.push("Foundation".to_string());
    }
    imports
}

impl Emitter<Swift> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Swift) -> Result<()> {
        write_module_header(w, self.config(), lang, &[])?;

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
            ContainerFormat::UnitStruct(doc) => struct_(w, self, &[], doc, lang)?,
            ContainerFormat::NewTypeStruct(format, doc) => struct_(
                w,
                self,
                &[&Named::new(format, "value".to_string())],
                doc,
                lang,
            )?,
            ContainerFormat::TupleStruct(formats, doc) => {
                let formats = named(formats, "field");
                struct_(w, self, &formats.iter().collect::<Vec<_>>(), doc, lang)?;
            }
            ContainerFormat::Struct(nameds, doc) => {
                struct_(w, self, &nameds.iter().collect::<Vec<_>>(), doc, lang)?;
            }
            ContainerFormat::Enum(variants, _, doc) => enum_(w, self, variants, doc, lang)?,
        }

        // Plugin after-type hook — fires once per top-level type, after its
        // closing brace. Never called for individual enum cases.
        let ctx = EmitContext::top_level(self, &lang.config);
        for plugin in lang.plugins() {
            plugin.after_type(w as &mut dyn IndentWrite, &ctx)?;
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Public helpers for plugin authors
// ---------------------------------------------------------------------------

/// Render `format` as the Swift type expression the emitter would use for a
/// property of that type — for example `Int32`, `[String]`, `Foo?`,
/// `[String: Bar]`, or `Other.Child` for a type in another namespace.
///
/// The type names in `format` must be in the emitter's spelling: a format
/// from [`EmitContext`] already is, and a name a plugin knows from elsewhere
/// goes through [`requalify`] first.
///
/// `config` supplies the current module name, which decides whether a
/// namespaced type is qualified.
///
/// # Panics
///
/// Panics if `format` is one Swift cannot express: a `Set` whose element type,
/// or a `Map` whose key type, is a native tuple or a dictionary (neither
/// conforms to `Hashable`). The emitter rejects the same formats with an
/// error, so such a registry never reaches code generation.
#[must_use]
pub fn render_type(format: &Format, config: &CodeGeneratorConfig) -> String {
    let lang = Swift {
        config: config.clone(),
        decided_types: BTreeSet::new(),
        hashable_types: BTreeSet::new(),
        equatable_types: BTreeSet::new(),
        plugins: vec![],
    };
    let mut buf = Vec::new();
    {
        let mut w = crate::generation::indent::IndentedWriter::new(
            &mut buf,
            crate::generation::indent::IndentConfig::Space(0),
        );
        format
            .write(&mut w, &lang)
            .expect("Swift type expression is not renderable");
    }
    String::from_utf8(buf).expect("type expression should be valid UTF-8")
}

/// The spelling the emitter gives a reference to `name`, a type name in
/// registry spelling, from the module `config` describes.
///
/// Before emitting a module the generator rewrites every type reference in its
/// registry with this rule: a ROOT type seen from a namespaced module is
/// qualified with the root package, whose target it lives in (`Root/Event` →
/// `Named("SharedTypes")/Event`, rendered `SharedTypes.Event`, inside `kv`
/// under root package `SharedTypes`). Only when the config knows the root
/// package ([`CodeGeneratorConfig::parent`], which the installer sets);
/// otherwise, and in the root module itself, and for every namespaced type,
/// the name is unchanged — [`render_type`] decides how a namespaced type is
/// written (bare in its own module, `Other.Child` elsewhere).
///
/// # Registry spelling and emitter spelling
///
/// The formats a plugin receives through [`EmitContext`] are already
/// requalified. A name a plugin knows from elsewhere is in registry spelling —
/// the key of the [`container`](EmitContext::container) being emitted, a name
/// the plugin built itself, or a type name inside a format from
/// [`RegistryBuilder::format_of`](crate::reflection::RegistryBuilder::format_of) —
/// and must go through `requalify` (a whole format through
/// [`requalify_format`]) before it is passed to [`render_type`],
/// [`write_serialize_value`](crate::generation::bincode::swift::write_serialize_value),
/// [`CodeGeneratorConfig::is_enum`] /
/// [`is_unit_enum`](CodeGeneratorConfig::is_unit_enum), or any other function
/// that expects the emitter's spelling. Without it, a ROOT type named from a
/// namespaced module is written bare, and a same-named type of the module's
/// own captures it.
///
/// To requalify every type name inside a format, use [`requalify_format`].
///
/// Unlike the other languages' `requalify`, Swift's happens to be idempotent
/// (it only ever produces a namespaced name, which it leaves alone), but it is
/// still meant to be applied once, to a registry-spelled name.
#[must_use]
pub fn requalify(config: &CodeGeneratorConfig, name: &QualifiedTypeName) -> QualifiedTypeName {
    super::generator::SwiftCodeGenerator::requalify(config, name)
}

/// Requalifies every type name in `format` with [`requalify`], as the
/// generator does to each container of the module before emitting it.
///
/// Apply it exactly once, to a format in registry spelling such as one from
/// [`RegistryBuilder::format_of`](crate::reflection::RegistryBuilder::format_of).
/// The formats in [`EmitContext`] are already requalified; requalifying one
/// again happens to change nothing in Swift, but does in the other languages.
///
/// ```
/// use facet_generate::{
///     generation::{CodeGeneratorConfig, swift},
///     reflection::format::{Format, QualifiedTypeName},
/// };
///
/// let mut config = CodeGeneratorConfig::new("kv".to_string());
/// config.parent = Some("SharedTypes".to_string());
///
/// let event = QualifiedTypeName::root("Event".to_string());
/// let mut format = Format::Option(Box::new(Format::TypeName(event)));
/// swift::requalify_format(&config, &mut format);
///
/// assert_eq!(swift::render_type(&format, &config), "SharedTypes.Event?");
/// ```
pub fn requalify_format(config: &CodeGeneratorConfig, format: &mut Format) {
    super::generator::SwiftCodeGenerator::requalify_type_names(config, format);
}

/// The Swift `case` name the emitter gives to an enum variant.
///
/// Variant names are lower-camel-cased (`NotFound` → `notFound`) and Swift
/// keywords are escaped with backticks (`Default` → `` `default` ``),
/// matching the emitter.
#[must_use]
pub fn case_name(variant_name: &str) -> String {
    escape_identifier(&variant_name.to_lower_camel_case()).into_owned()
}

/// The Swift property name the emitter gives to a struct field, a
/// struct-variant field, or a tuple/newtype member.
///
/// Field names are lower-camel-cased (`not_found` → `notFound`) and Swift
/// keywords are escaped with backticks (`default` → `` `default` ``).
///
/// Plugins that emit a property access, a parameter label, or a binding
/// derived from a field name should route it through this so the result
/// matches the emitter.
#[must_use]
pub fn field_name(name: &str) -> String {
    escape_identifier(&name.to_lower_camel_case()).into_owned()
}

/// Escapes an identifier when it is a Swift keyword, by wrapping it in
/// backticks.
///
/// Backticks are pure quoting: the identifier's spelling is unchanged, so an
/// escaped name is interchangeable with the bare one everywhere except the
/// source text. Already-escaped identifiers are returned unchanged.
#[must_use]
pub fn escape_identifier(identifier: &str) -> Cow<'_, str> {
    super::naming::RULES.escape(identifier)
}

// ---------------------------------------------------------------------------
// Format emitter
// ---------------------------------------------------------------------------

/// Renders a type reference for display: same-module and root-namespace types
/// emit a bare name, while external namespaces are prefixed with the namespace
/// in `UpperCamelCase` (e.g. `Other.Child`).
fn render_type_name(qtn: &QualifiedTypeName, config: &CodeGeneratorConfig) -> String {
    match &qtn.namespace {
        Namespace::Root => qtn.name.clone(),
        Namespace::Named(ns) if ns == config.module_name() => qtn.name.clone(),
        Namespace::Named(ns) => format!("{}.{}", heck::AsUpperCamelCase(ns), qtn.name),
    }
}

impl Emitter<Swift> for Format {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Swift) -> Result<()> {
        match &self {
            Self::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Self::TypeName(qualified_type_name) => {
                write!(w, "{}", render_type_name(qualified_type_name, &lang.config))
            }
            Self::Unit => write!(w, "{}", builtin("Void", &lang.config)),
            Self::Bool => write!(w, "{}", builtin("Bool", &lang.config)),
            Self::I8 => write!(w, "{}", builtin("Int8", &lang.config)),
            Self::I16 => write!(w, "{}", builtin("Int16", &lang.config)),
            Self::I32 => write!(w, "{}", builtin("Int32", &lang.config)),
            Self::I64 => write!(w, "{}", builtin("Int64", &lang.config)),
            Self::I128 => write!(w, "Int128"),
            Self::U8 => write!(w, "{}", builtin("UInt8", &lang.config)),
            Self::U16 => write!(w, "{}", builtin("UInt16", &lang.config)),
            Self::U32 => write!(w, "{}", builtin("UInt32", &lang.config)),
            Self::U64 => write!(w, "{}", builtin("UInt64", &lang.config)),
            Self::U128 => write!(w, "UInt128"),
            Self::F32 => write!(w, "{}", builtin("Float", &lang.config)),
            Self::F64 => write!(w, "{}", builtin("Double", &lang.config)),
            Self::Char => write!(w, "{}", builtin("Character", &lang.config)),
            Self::Str => write!(w, "{}", builtin("String", &lang.config)),
            Self::Bytes => write!(w, "[{}]", builtin("UInt8", &lang.config)),
            Self::Uuid => write!(w, "{}", builtin("UUID", &lang.config)),

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
                write!(w, "{}<", builtin("Set", &lang.config))?;
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
                // Swift requires K: Hashable for [K: V] to compile, but V need not be Hashable.
                // If the containing type requires Hashable, that is gated by the all_hashable check.
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
        let name = &field_name(name);

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
        let name = case_name(name);

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
    let all_hashable = conformance::fields_are_hashable(fields.iter().map(|f| &f.value), &|name| {
        lang.is_hashable_type(name)
    });
    let all_equatable_auto = fields.iter().all(|f| is_equatable_auto(&f.value, lang));
    let all_can_eq = conformance::fields_are_equatable(fields.iter().map(|f| &f.value), &|name| {
        lang.is_equatable_type(name)
    });

    let mut implements = vec![];

    if all_hashable {
        implements.push(builtin("Hashable", &lang.config));
    }
    if all_equatable_auto || all_can_eq {
        implements.push(builtin("Equatable", &lang.config));
    }
    let ctx = EmitContext::top_level(container, &lang.config);
    let implements = with_plugin_conformances(implements, &ctx, lang);

    if has_plugins && !implements.is_empty() {
        write!(w, "public struct {name}: {} ", implements.join(", "))?;
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
            let fname = field_name(&field.name);
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
    let hashable = |name: &QualifiedTypeName| lang.is_hashable_type(name);
    let equatable = |name: &QualifiedTypeName| lang.is_equatable_type(name);
    let all_hashable =
        conformance::variants_are_hashable(variants.values().map(|v| &v.value), &hashable);
    let all_equatable_auto = variants
        .values()
        .all(|v| conformance::variant_is_equatable_auto(&v.value, &equatable));
    let all_can_eq =
        conformance::variants_are_equatable(variants.values().map(|v| &v.value), &equatable);

    let mut implements = vec![];

    if all_hashable {
        implements.push(builtin("Hashable", &lang.config));
    }
    if all_equatable_auto || all_can_eq {
        implements.push(builtin("Equatable", &lang.config));
    }
    let ctx = EmitContext::top_level(container, &lang.config);
    let implements = with_plugin_conformances(implements, &ctx, lang);

    if has_plugins && !implements.is_empty() {
        write!(w, "indirect public enum {name}: {} ", implements.join(", "))?;
    } else {
        write!(w, "indirect public enum {name} ")?;
    }

    let mut w = w.block(Newlines::BOTH)?;

    for variant in variants.values() {
        (variant, Usage::Field).write(&mut w, lang)?;
    }

    // Plugin type bodies (serialize / deserialize methods).
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
            let variant_name = case_name(&variant.name);
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
                        let fname = field_name(&n.name);
                        write!(w, "{fname}: let l{i}")?;
                    }
                    write!(w, "), .{variant_name}(")?;
                    for (i, n) in nameds.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        let fname = field_name(&n.name);
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

/// `implements` followed by every plugin's
/// [`type_conformances`](EmitterPlugin::type_conformances), without repeats.
fn with_plugin_conformances<'a>(
    implements: Vec<Cow<'a, str>>,
    ctx: &EmitContext,
    lang: &Swift,
) -> Vec<Cow<'a, str>> {
    let mut implements = implements;
    for plugin in lang.plugins() {
        for conformance in plugin.type_conformances(ctx) {
            if !implements.iter().any(|c| *c == conformance) {
                implements.push(Cow::Owned(conformance));
            }
        }
    }
    implements
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
