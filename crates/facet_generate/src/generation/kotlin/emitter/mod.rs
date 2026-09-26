//! AST-to-Kotlin source rendering.
//!
//! This module implements [`Emitter<Kotlin>`](super::super::Emitter) for each
//! node type in the format AST, turning abstract type descriptions into
//! idiomatic Kotlin code.
//!
//! # Emitter implementations
//!
//! | AST node | Kotlin output |
//! |---|---|
//! | [`Module`] | `package` declaration, `import` statements, feature helpers |
//! | [`Container`] | `data class`, `data object`, `sealed interface`, or `enum class` |
//! | [`Named<Format>`](Named) | A single `val` property declaration |
//! | [`Format`] | An inline type expression (`Int`, `List<String>`, `Pair<A, B>`, …) |
//! | [`Doc`] | `///` doc comments |
//! | `(Named<VariantFormat>, VariantContext)` | An enum/sealed-interface variant |
//!
//! # Kotlin type mapping
//!
//! The [`Format`] emitter maps Rust/reflection types to Kotlin equivalents —
//! for example `I32` → `Int`, `Seq(T)` → `List<T>`, `Option(T)` → `T?`,
//! tuples of size 2/3 → `Pair`/`Triple`, and tuples of 4 to
//! [`MAX_TUPLE_LEN`] elements to the serde runtime's `Tuple4<…>` to
//! `Tuple12<…>`.
//!
//! # Plugin-dependent output
//!
//! The [`Kotlin`] language tag carries a list of [`EmitterPlugin`]s. All
//! encoding-specific behaviour is delegated to those plugins — the emitter
//! itself contains no encoding checks. For example:
//!
//! - `JsonPlugin` supplies `@Serializable` type annotations, `@SerialName`
//!   annotations on properties (through
//!   [`EmitterPlugin::field_annotations`]) and on all-unit enum class
//!   variants, and a nested `JsonSerializer` for the types whose JSON the
//!   compiler plugin would not write the way Rust does.
//! - `BincodePlugin` supplies `serialize` / `deserialize` methods and
//!   convenience `bincodeSerialize` / `bincodeDeserialize` wrappers.
//! - With no plugins, only plain type declarations are emitted.
//!
//! # Feature helpers
//!
//! The encoding-independent `TupleArray` helper (`buildList` polyfill for
//! Kotlin < 1.6.0) is inlined here as [`FEATURE_TUPLE_ARRAY`] and emitted
//! when [`Feature::TupleArray`] is set by [`CodeGeneratorConfig::update_from`].
//!
//! Bincode container helpers (`List<T>.serialize`, `Set<T>.serialize`, etc.)
//! are inlined in `BincodePlugin` (`generation/bincode/kotlin.rs`).
//! The JSON `Bytes`, `UUID` and `BigInteger` `KSerializer`s are inlined in
//! `JsonPlugin` (`generation/json/kotlin.rs`).

use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{Result, Write},
    string::ToString,
    sync::Arc,
};

use heck::ToLowerCamelCase;

use super::naming::{self, builtin};
use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Container, Emitter, Feature,
        collision::TypeName,
        indent::{IndentWrite, Newlines},
        module::Module,
        naming::qualify_helper,
        plugin::{EmitContext, EmitterPlugin, VariantInfo},
    },
    reflection::format::{
        ContainerFormat, Doc, Format, FormatHolder, Named, QualifiedTypeName, VariantFormat,
    },
};

const FEATURE_TUPLE_ARRAY: &str = r"/**
 * Compatibility functions for buildList, ensuring support for Kotlin versions < 1.6.0
 *
 * These functions provide the same functionality as the standard library buildList functions
 * introduced in Kotlin 1.6.0. On Kotlin 1.6+, the compiler will prefer the standard library
 * versions due to better overload resolution, so these serve as fallbacks for older versions.
 *
 * The functions are inline and generate efficient bytecode equivalent to the standard library
 * implementations, so there's no performance penalty when included.
 */
inline fun <T> buildList(capacity: Int, builderAction: MutableList<T>.() -> Unit): List<T> {
    val list = ArrayList<T>(capacity)
    list.builderAction()
    return list
}

inline fun <T> buildList(builderAction: MutableList<T>.() -> Unit): List<T> {
    val list = mutableListOf<T>()
    list.builderAction()
    return list
}
";

/// Language tag for Kotlin code generation.
///
/// Passed as the `L` parameter to every [`Emitter<L>`](super::super::Emitter)
/// call. Carries a plugin list that controls all encoding-specific behaviour.
#[derive(Debug, Clone)]
pub struct Kotlin {
    pub(crate) config: CodeGeneratorConfig,
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Self>>>,
}

impl Kotlin {
    /// Create a Kotlin language tag with no default plugins.
    ///
    /// Use [`with_plugin`](Self::with_plugin) to attach plugins.
    #[must_use]
    pub fn new(config: &CodeGeneratorConfig, _registry: &Registry) -> Self {
        Self {
            config: config.clone(),
            plugins: vec![],
        }
    }

    /// Access the generator config.
    #[must_use]
    pub const fn config(&self) -> &CodeGeneratorConfig {
        &self.config
    }

    /// Add a plugin to this language tag, returning the modified tag.
    ///
    /// Plugins are invoked in the order they are added.
    #[must_use]
    pub fn with_plugin(mut self, plugin: Arc<dyn EmitterPlugin<Self>>) -> Self {
        self.plugins.push(plugin);
        self
    }

    /// Access the plugin list.
    #[must_use]
    pub fn plugins(&self) -> &[Arc<dyn EmitterPlugin<Self>>] {
        &self.plugins
    }
}

/// Write the module header — the `package` declaration and the sorted,
/// deduplicated `import` lines (feature-driven, plugin-provided, and the
/// caller's `extra_imports`, each a whole `import …` line), less any that
/// would hide a plugin's [module declaration](EmitterPlugin::module_declarations).
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
    lang: &Kotlin,
    extra_imports: &[String],
) -> Result<()> {
    let CodeGeneratorConfig {
        module_name,
        features,
        ..
    } = config;

    writeln!(w, "package {module_name}")?;
    writeln!(w)?;

    // --- Imports ---
    // Language-level imports that are NOT driven by plugins stay here.
    // Bincode imports are now provided by BincodePlugin::imports().
    let mut imports: Vec<String> = vec![];

    // `import java.math.BigInteger` is needed regardless of plugins, including
    // when no plugin runs. Plugin-specific BigInt imports (JSON KSerializer,
    // Bincode Int128) are added by their respective plugins.
    if features.contains(&Feature::BigInt) {
        imports.push("import java.math.BigInteger".to_string());
    }

    // The emitter, not a plugin, imports `UUID`, so that it's in scope with
    // no plugin (#191). JSON's `UUID` alias drops it below.
    if features.contains(&Feature::Uuid) {
        imports.push("import java.util.UUID".to_string());
    }

    // --- Plugin imports ---
    for plugin in lang.plugins() {
        imports.extend(plugin.imports(config));
    }

    imports.extend(extra_imports.iter().cloned());

    // An import outranks a declaration in the same package, so drop any
    // import of a name a plugin declares, such as JSON's `UUID` and `Bytes`
    // aliases. Bincode then uses those aliases, which are the same types (#204).
    let declared: Vec<String> = lang
        .plugins()
        .iter()
        .flat_map(|plugin| plugin.module_declarations(config))
        .collect();
    imports.retain(|import| !declared.iter().any(|name| imported_name(import) == name));

    imports.sort_unstable();
    imports.dedup();
    if !imports.is_empty() {
        for import in imports {
            writeln!(w, "{import}")?;
        }
        writeln!(w)?;
    }

    Ok(())
}

/// The name an `import …` line brings into scope: its alias, or the last
/// segment of its path.
fn imported_name(import: &str) -> &str {
    let import = import.trim_start_matches("import ").trim();
    match import.split_once(" as ") {
        Some((_, alias)) => alias.trim(),
        None => import.rsplit('.').next().unwrap_or(import),
    }
}

impl Emitter<Kotlin> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        write_module_header(w, self.config(), lang, &[])?;

        // --- Feature helpers (non-plugin) ---
        let mut features_out = vec![];
        if self.config().features.contains(&Feature::TupleArray) {
            // TupleArray is encoding-independent — stays in the emitter.
            write!(
                features_out,
                "{}",
                qualify_helper(FEATURE_TUPLE_ARRAY, naming::QUALIFIED, |name| {
                    naming::shadows(name, self.config())
                })
            )?;
            writeln!(features_out)?;
        }

        // --- Plugin module helpers ---
        {
            let mut fw = w.child(&mut features_out);
            for plugin in lang.plugins() {
                plugin.module_helpers(&mut fw, self.config())?;
            }
        }

        w.write_all(&features_out)?;

        Ok(())
    }
}

impl Emitter<Kotlin> for Container<'_> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        let Container {
            name: QualifiedTypeName { namespace: _, name },
            format,
            ..
        } = self;
        match format {
            ContainerFormat::UnitStruct(doc) => {
                data_object(w, name, Site::TopLevel(self), doc, lang)?;
            }
            ContainerFormat::NewTypeStruct(format, doc) => {
                data_class(
                    w,
                    name,
                    Site::TopLevel(self),
                    &[Named::new(format, "value".to_string())],
                    doc,
                    lang,
                )?;
            }
            ContainerFormat::TupleStruct(formats, doc) => {
                data_class(w, name, Site::TopLevel(self), &named(formats), doc, lang)?;
            }
            ContainerFormat::Struct(fields, doc) => {
                if fields.is_empty() {
                    data_object(w, name, Site::TopLevel(self), doc, lang)?;
                } else {
                    data_class(w, name, Site::TopLevel(self), fields, doc, lang)?;
                }
            }
            ContainerFormat::Enum(variants, _, doc) => {
                let variant_list: Vec<_> = variants.values().cloned().collect();

                let all_unit_variants = variants
                    .values()
                    .all(|variant| matches!(variant.value, VariantFormat::Unit));

                if all_unit_variants {
                    enum_class(w, name, variants, doc, lang, self)?;
                } else {
                    sealed_interface(w, name, &variant_list, doc, lang, self)?;
                }
            }
        }

        // Plugin after-type hook — fires once per top-level type, after its
        // closing brace. `data_class` / `data_object` deliberately do not call
        // it: they are reused for sealed-interface variants (with a temporary
        // container), and `after_type` is a top-level-only hook.
        let ctx = EmitContext::top_level(self, &lang.config);
        for plugin in lang.plugins() {
            plugin.after_type(w as &mut dyn IndentWrite, &ctx)?;
        }

        Ok(())
    }
}

impl Emitter<Kotlin> for Named<Format> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        write_property(w, self, &[], lang)
    }
}

/// Writes a `val` property declaration, preceded on its line by `annotations`.
fn write_property<W: IndentWrite>(
    w: &mut W,
    field: &Named<Format>,
    annotations: &[String],
    lang: &Kotlin,
) -> Result<()> {
    field.doc.write(w, lang)?;

    for annotation in annotations {
        write!(w, "{annotation} ")?;
    }

    let name = &property_name(&field.name);
    write!(w, "val {name}: ")?;

    field.value.write(w, lang)?;

    // Add = null default only for top-level Option types
    if matches!(field.value, Format::Option(_)) {
        write!(w, " = null")?;
    }

    writeln!(w, ",")
}

impl Emitter<Kotlin> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: &Kotlin) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }

        Ok(())
    }
}

/// Tells a variant emitter whether it is being written inside a
/// `sealed interface` or an `enum class`, since the Kotlin syntax differs.
#[derive(Clone)]
pub enum VariantContext {
    /// Variant inside a `sealed interface` — carries the interface name and
    /// the variant's zero-based index (used as the bincode discriminant).
    SealedInterface(String, usize),
    /// Variant inside an `enum class` (all-unit variants only).
    EnumClass,
}

impl Emitter<Kotlin> for (&Named<VariantFormat>, &VariantContext) {
    #[allow(clippy::too_many_lines)]
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        let (
            Named {
                name,
                doc,
                value: format,
            },
            context,
        ) = self;

        match (&format, context) {
            (VariantFormat::Variable(_), _) => {
                unreachable!("placeholders should not get this far")
            }
            (VariantFormat::Unit, VariantContext::SealedInterface(interface_name, index)) => {
                data_object(w, name, Site::variant(interface_name, *index), doc, lang)?;
            }
            (VariantFormat::Unit, VariantContext::EnumClass) => {
                doc.write(w, lang)?;
                let name_upper = name.to_uppercase();
                let prefix_parts: Vec<String> = lang
                    .plugins()
                    .iter()
                    .flat_map(|p| p.enum_variant_annotations(name))
                    .collect();
                if prefix_parts.is_empty() {
                    write!(w, "{name_upper}")?;
                } else {
                    let prefix = prefix_parts.join(" ");
                    write!(w, "{prefix} {name_upper}")?;
                }
            }
            (
                VariantFormat::NewType(inner),
                VariantContext::SealedInterface(interface_name, index),
            ) => {
                data_class(
                    w,
                    name,
                    Site::variant(interface_name, *index),
                    &[Named::new(inner, "value".to_string())],
                    doc,
                    lang,
                )?;
            }
            (VariantFormat::NewType(_format), VariantContext::EnumClass) => {
                unreachable!("NewType variants are not supported in enum classes")
            }
            (
                VariantFormat::Tuple(formats),
                VariantContext::SealedInterface(interface_name, index),
            ) => {
                data_class(
                    w,
                    name,
                    Site::variant(interface_name, *index),
                    &named(formats),
                    doc,
                    lang,
                )?;
            }
            (VariantFormat::Tuple(_formats), VariantContext::EnumClass) => {
                unreachable!("Tuple variants are not supported in enum classes")
            }
            (
                VariantFormat::Struct(fields),
                VariantContext::SealedInterface(interface_name, index),
            ) => {
                data_class(
                    w,
                    name,
                    Site::variant(interface_name, *index),
                    fields,
                    doc,
                    lang,
                )?;
            }
            (VariantFormat::Struct(_fields), VariantContext::EnumClass) => {
                unreachable!("Struct variants are not supported in enum classes")
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Public helpers for plugin authors
// ---------------------------------------------------------------------------

/// Render `format` as the Kotlin type expression the emitter would use for a
/// property of that type — for example `Int`, `List<String>`, `Foo?`,
/// `Map<String, Bar>`, or `com.example.other.Child` for a type in namespace
/// `other` of root package `com.example`.
///
/// The type names in `format` must be in the emitter's spelling, a full
/// package path: a format from [`EmitContext`] already is, and a name a plugin
/// knows from elsewhere goes through [`requalify`] first. A registry-spelled
/// `Named("other")/Child` would render as a bare `other.Child`.
///
/// `config` is accepted for symmetry with the other languages and to keep the
/// helper stable if Kotlin's type rendering becomes configuration-dependent.
///
/// # Panics
///
/// Panics if `format` is a placeholder ([`Format::Variable`]), which never
/// survives registry construction.
#[must_use]
pub fn render_type(format: &Format, config: &CodeGeneratorConfig) -> String {
    let lang = Kotlin {
        config: config.clone(),
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
            .expect("writing to a Vec cannot fail");
    }
    String::from_utf8(buf).expect("type expression should be valid UTF-8")
}

/// The spelling the emitter gives a reference to `name`, a type name in
/// registry spelling, from the module `config` describes.
///
/// Before emitting a module the generator rewrites every type reference in its
/// registry to a full package path with this rule, which roots every
/// namespace at the root package (the parent the installer nested the module
/// under, or the module itself):
///
/// - a namespace with an external package whose location is a path is
///   qualified with that path (`Named("other")/Row` with path `com.acme` →
///   `Named("com.acme.other")/Row`);
/// - a type of the module's own namespace is qualified with the module
///   (`Named("kit")/Row` → `Named("com.example.kit")/Row` inside
///   `com.example.kit`);
/// - a type of any other namespace is qualified with the root package
///   (`Named("kit")/Row` → `Named("com.example.kit")/Row` inside
///   `com.example` or `com.example.kv`), even when the root package ends in
///   that namespace's name (`Named("kv")/Entry` → `Named("com.kv.kv")/Entry`
///   inside the root module `com.kv`) — which namespace is the module's own
///   comes from its config, not from its name;
/// - a ROOT type is qualified with the root package (`Root/Event` →
///   `Named("com.example")/Event`, rendered `com.example.Event`).
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
/// [`write_serialize_value`](crate::generation::bincode::kotlin::write_serialize_value),
/// [`CodeGeneratorConfig::is_enum`] /
/// [`is_unit_enum`](CodeGeneratorConfig::is_unit_enum), or any other function
/// that expects the emitter's spelling.
///
/// To requalify every type name inside a format, use [`requalify_format`].
///
/// Apply it once, to a registry-spelled name: it is not idempotent. A second
/// pass roots an already-qualified path at the root package again
/// (`com.example.com.example.Event`).
#[must_use]
pub fn requalify(config: &CodeGeneratorConfig, name: &QualifiedTypeName) -> QualifiedTypeName {
    super::generator::KotlinCodeGenerator::requalify(config, name)
}

/// Requalifies every type name in `format` with [`requalify`], as the
/// generator does to each container of the module before emitting it.
///
/// Apply it exactly once, to a format in registry spelling such as one from
/// [`RegistryBuilder::format_of`](crate::reflection::RegistryBuilder::format_of).
/// The formats in [`EmitContext`] are already requalified; requalifying one
/// again nests the root package (`com.example.com.example.Event`).
///
/// ```
/// use facet_generate::{
///     generation::{CodeGeneratorConfig, kotlin},
///     reflection::format::{Format, QualifiedTypeName},
/// };
///
/// let config = CodeGeneratorConfig::new("kv".to_string()).with_parent("com.example");
///
/// let event = QualifiedTypeName::root("Event".to_string());
/// let mut format = Format::Option(Box::new(Format::TypeName(event)));
/// kotlin::requalify_format(&config, &mut format);
///
/// assert_eq!(kotlin::render_type(&format, &config), "com.example.Event?");
/// ```
pub fn requalify_format(config: &CodeGeneratorConfig, format: &mut Format) {
    super::generator::KotlinCodeGenerator::requalify_type_names(config, format);
}

/// The name of the nested `data class` / `data object` the emitter generates
/// for a variant of a `sealed interface` (an enum with at least one variant
/// that carries data).
///
/// The variant name is used verbatim, so from outside the interface the class
/// is referred to as `Parent.Variant`.
#[must_use]
pub fn variant_class_name(variant_name: &str) -> String {
    variant_name.to_string()
}

/// The name of the constant the emitter generates for a variant of an
/// `enum class` (an enum whose variants are all unit variants).
#[must_use]
pub fn enum_constant_name(variant_name: &str) -> String {
    variant_name.to_uppercase()
}

/// The Kotlin property name the emitter gives to a struct field, a
/// struct-variant field, or a tuple/newtype member.
///
/// Field names are lower-camel-cased (`not_found` → `notFound`) and Kotlin
/// hard keywords are escaped with backticks (`in` → `` `in` ``). Soft
/// keywords — including the synthetic member names `value` and `field0` — are
/// left alone.
///
/// Plugins that emit a property access, a local binding, or a constructor
/// argument derived from a field name should route it through this so the
/// result matches the emitter.
#[must_use]
pub fn property_name(name: &str) -> String {
    escape_identifier(&name.to_lower_camel_case()).into_owned()
}

/// Escapes an identifier when it is a Kotlin hard keyword, by wrapping it in
/// backticks.
///
/// Backticks are pure quoting: the identifier's spelling is unchanged, so the
/// name a serialization format sees (a `@SerialName`, a JSON key) is the bare
/// one. Already-escaped identifiers are returned unchanged.
#[must_use]
pub fn escape_identifier(identifier: &str) -> Cow<'_, str> {
    super::naming::RULES.escape(identifier)
}

/// The most elements a tuple the generated code holds can have: the serde
/// runtime's tuple types go up to `Tuple12`. That is as far as `facet`
/// reflects a tuple too (with its `tuples-12` feature, and to four without),
/// and as far as the standard library implements traits such as `PartialEq`
/// and `Debug` for one, so only a hand-built registry has a longer tuple.
pub(crate) const MAX_TUPLE_LEN: usize = 12;

/// Reject a registry holding a tuple longer than [`MAX_TUPLE_LEN`], which the
/// generated code would name as a `TupleN` the serde runtime does not have.
///
/// Run before anything is written, so a failing registry produces no output.
/// A tuple struct or a tuple variant can have any number of fields: the
/// generated code declares a class for each.
///
/// # Errors
///
/// Returns [`io::ErrorKind::InvalidInput`](std::io::ErrorKind::InvalidInput)
/// naming the first type that holds such a tuple.
pub(crate) fn check_tuple_sizes(registry: &Registry) -> Result<()> {
    for (name, container) in registry {
        let mut longest = 0;
        // The visitor only fails on an unresolved variable, which a finished
        // registry never contains.
        let _ = container.visit(&mut |format| {
            if let Format::Tuple(formats) = format {
                longest = longest.max(formats.len());
            }
            Ok(())
        });
        if longest > MAX_TUPLE_LEN {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Kotlin: {type_name} holds a tuple of {longest} elements, but the serde \
                     runtime's tuple types stop at `Tuple{MAX_TUPLE_LEN}`; group some of its \
                     elements into a struct or a nested tuple",
                    type_name = TypeName(name),
                ),
            ));
        }
    }
    Ok(())
}

impl Emitter<Kotlin> for Format {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        match &self {
            Self::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Self::TypeName(qualified_type_name) => {
                write!(
                    w,
                    "{ty}",
                    ty = qualified_type_name.format(ToString::to_string, ".")
                )
            }
            Self::Unit => write!(w, "{}", builtin("Unit", &lang.config)),
            Self::Bool => write!(w, "{}", builtin("Boolean", &lang.config)),
            Self::I8 => write!(w, "{}", builtin("Byte", &lang.config)),
            Self::I16 => write!(w, "{}", builtin("Short", &lang.config)),
            Self::I32 => write!(w, "{}", builtin("Int", &lang.config)),
            Self::I64 => write!(w, "{}", builtin("Long", &lang.config)),
            Self::U8 => write!(w, "{}", builtin("UByte", &lang.config)),
            Self::U16 => write!(w, "{}", builtin("UShort", &lang.config)),
            Self::U32 => write!(w, "{}", builtin("UInt", &lang.config)),
            Self::U64 => write!(w, "{}", builtin("ULong", &lang.config)),
            Self::I128 | Self::U128 => write!(w, "BigInteger"),
            Self::F32 => write!(w, "{}", builtin("Float", &lang.config)),
            Self::F64 => write!(w, "{}", builtin("Double", &lang.config)),
            Self::Char | Self::Str => write!(w, "{}", builtin("String", &lang.config)),
            Self::Bytes => write!(w, "Bytes"),
            Self::Uuid => write!(w, "UUID"),

            Self::Option(format) => {
                format.write(w, lang)?;
                write!(w, "?")
            }
            Self::Seq(format) => {
                write!(w, "{}<", builtin("List", &lang.config))?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Self::Set(format) => {
                write!(w, "{}<", builtin("Set", &lang.config))?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Self::Map { key, value } => {
                write!(w, "{}<", builtin("Map", &lang.config))?;
                key.write(w, lang)?;
                write!(w, ", ")?;
                value.write(w, lang)?;
                write!(w, ">")
            }
            Self::Tuple(formats) => {
                let len = formats.len();
                match len {
                    0 => write!(w, "{}", builtin("Unit", &lang.config)),
                    1 => {
                        // A single-element tuple is just the element itself
                        formats[0].write(w, lang)
                    }
                    2 => {
                        write!(w, "{}<", builtin("Pair", &lang.config))?;
                        formats[0].write(w, lang)?;
                        write!(w, ", ")?;
                        formats[1].write(w, lang)?;
                        write!(w, ">")
                    }
                    3 => {
                        write!(w, "{}<", builtin("Triple", &lang.config))?;
                        formats[0].write(w, lang)?;
                        write!(w, ", ")?;
                        formats[1].write(w, lang)?;
                        write!(w, ", ")?;
                        formats[2].write(w, lang)?;
                        write!(w, ">")
                    }
                    _ => {
                        // The serde runtime's `TupleN`, which the plugins
                        // import; `check_tuple_sizes` rejects a longer one.
                        write!(w, "Tuple{len}<")?;
                        for (i, format) in formats.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            format.write(w, lang)?;
                        }
                        write!(w, ">")
                    }
                }
            }
            Self::TupleArray { content, size: _ } => {
                write!(w, "{}<", builtin("List", &lang.config))?;
                content.write(w, lang)?;
                write!(w, ">")
            }
        }
    }
}

/// Where a `data object` or `data class` is declared: at the top level, for
/// the container it renders, or nested in a `sealed interface` as a variant.
#[derive(Clone, Copy)]
enum Site<'a> {
    TopLevel(&'a Container<'a>),
    Variant { interface: &'a str, index: usize },
}

impl<'a> Site<'a> {
    const fn variant(interface: &'a str, index: usize) -> Self {
        Self::Variant { interface, index }
    }

    const fn interface(self) -> Option<&'a str> {
        match self {
            Self::TopLevel(_) => None,
            Self::Variant { interface, .. } => Some(interface),
        }
    }

    /// Calls `f` with the plugin context for a type named `name` declared
    /// here: the container itself at the top level, and a variant `format`
    /// with `fields` inside a `sealed interface`.
    fn with_context<T>(
        self,
        name: &str,
        format: &VariantFormat,
        fields: &[Named<Format>],
        lang: &Kotlin,
        f: impl FnOnce(&EmitContext) -> T,
    ) -> T {
        match self {
            Self::TopLevel(container) => f(&EmitContext::top_level(container, &lang.config)),
            Self::Variant { interface, index } => {
                let temp_name = QualifiedTypeName::root(name.to_string());
                let temp_format = match format {
                    VariantFormat::Unit => ContainerFormat::UnitStruct(Doc::default()),
                    _ => ContainerFormat::Struct(fields.to_vec(), Doc::default()),
                };
                let temp_container = Container {
                    name: &temp_name,
                    format: &temp_format,
                };
                f(&EmitContext::for_variant(
                    &temp_container,
                    &lang.config,
                    VariantInfo {
                        name,
                        index,
                        format,
                        fields,
                        parent_name: interface,
                    },
                ))
            }
        }
    }
}

/// Emits a Kotlin `data object` — used for unit structs and unit variants.
///
/// A variant object implements its `sealed interface`. Encoding-specific body
/// code (e.g. serialize / deserialize methods) is delegated to plugins via the
/// `type_body` hook.
fn data_object<W: IndentWrite>(
    w: &mut W,
    name: &str,
    site: Site,
    doc: &Doc,
    lang: &Kotlin,
) -> Result<()> {
    doc.write(w, lang)?;

    site.with_context(name, &VariantFormat::Unit, &[], lang, |ctx| {
        write_plugin_annotations(w, lang, ctx)?;

        write!(w, "data object {name}")?;

        if let Some(interface) = site.interface() {
            write!(w, ": {interface}")?;
        }

        write_plugin_body(w, lang, ctx)
    })
}

/// Emits a Kotlin `data class` — used for structs (with fields), newtype
/// structs, tuple structs, and non-unit sealed-interface variants.
///
/// A variant class implements its `sealed interface`. Encoding-specific body
/// code (e.g. serialize / deserialize methods) is delegated to plugins via the
/// `type_body` hook, and annotations on each property to the
/// `field_annotations` hook.
fn data_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    site: Site,
    fields: &[Named<Format>],
    doc: &Doc,
    lang: &Kotlin,
) -> Result<()> {
    doc.write(w, lang)?;

    let variant_format = VariantFormat::Struct(fields.to_vec());
    site.with_context(name, &variant_format, fields, lang, |ctx| {
        write_plugin_annotations(w, lang, ctx)?;

        writeln!(w, "data class {name}(")?;

        w.indent();
        for field in fields {
            let annotations: Vec<String> = lang
                .plugins()
                .iter()
                .flat_map(|p| p.field_annotations(field, ctx))
                .collect();
            write_property(w, field, &annotations, lang)?;
        }
        w.unindent();

        write!(w, ")")?;

        if let Some(interface) = site.interface() {
            write!(w, " : {interface}")?;
        }

        write_plugin_body(w, lang, ctx)
    })
}

/// Emits a Kotlin `enum class` — used when all variants are unit variants.
///
/// Encoding-specific annotations (e.g. `@SerialName` for JSON) are handled
/// by the variant emitter; type-body code is delegated to plugins.
fn enum_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    lang: &Kotlin,
    container: &Container,
) -> Result<()> {
    doc.write(w, lang)?;

    write_plugin_annotations(w, lang, &EmitContext::top_level(container, &lang.config))?;

    write!(w, "enum class {name} ")?;
    let mut w = w.block(Newlines::BOTH)?;

    for (i, variant) in variants {
        if *i > 0 {
            writeln!(w, ",")?;
        }

        (variant, &VariantContext::EnumClass).write(&mut w, lang)?;
    }
    writeln!(w, ";")?;

    // Plugin type body (e.g. JSON serialName accessor)
    {
        let ctx = EmitContext::top_level(container, &lang.config);
        for plugin in lang.plugins() {
            plugin.type_body(&mut w as &mut dyn IndentWrite, &ctx)?;
        }
    }

    Ok(())
}

/// Emits a Kotlin `sealed interface` — used when at least one variant
/// carries data (newtype, tuple, or struct variant).
///
/// Each variant becomes a nested `data class` or `data object` that
/// implements the interface. Encoding-specific body code (preamble and
/// companion objects) is delegated to plugins.
fn sealed_interface<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &[Named<VariantFormat>],
    doc: &Doc,
    lang: &Kotlin,
    container: &Container,
) -> Result<()> {
    doc.write(w, lang)?;

    write_plugin_annotations(w, lang, &EmitContext::top_level(container, &lang.config))?;

    write!(w, "sealed interface {name} ")?;
    let mut w = w.block(Newlines::BOTH)?;

    // Plugin type body preamble (before variants)
    {
        let ctx = EmitContext::top_level(container, &lang.config);
        for plugin in lang.plugins() {
            plugin.type_body_preamble(&mut w as &mut dyn IndentWrite, &ctx)?;
        }
    }

    for (index, variant) in variants.iter().enumerate() {
        if index > 0 {
            writeln!(w)?;
        }
        let ctx = VariantContext::SealedInterface(name.to_string(), index);
        (variant, &ctx).write(&mut w, lang)?;
    }

    // Plugin type body (after variants)
    {
        let ctx = EmitContext::top_level(container, &lang.config);
        for plugin in lang.plugins() {
            plugin.type_body(&mut w as &mut dyn IndentWrite, &ctx)?;
        }
    }

    Ok(())
}

/// Run plugin type-body hooks, opening a `{ }` block if any plugin needs one.
/// If no plugin needs a body, emits a plain newline instead.
fn write_plugin_body<W: IndentWrite>(w: &mut W, lang: &Kotlin, ctx: &EmitContext) -> Result<()> {
    let needs_body = lang.plugins().iter().any(|p| p.has_type_body(ctx));
    if needs_body {
        write!(w, " ")?;
        let mut w = w.block(Newlines::BOTH)?;
        for plugin in lang.plugins() {
            plugin.type_body(&mut w as &mut dyn IndentWrite, ctx)?;
        }
    } else {
        writeln!(w)?;
    }
    Ok(())
}

/// Emits plugin type annotations (e.g. `@Serializable`, `@SerialName`), each
/// on its own line, for the type `ctx` describes.
fn write_plugin_annotations<W: IndentWrite>(
    w: &mut W,
    lang: &Kotlin,
    ctx: &EmitContext,
) -> Result<()> {
    for plugin in lang.plugins() {
        for annotation in plugin.type_annotations(ctx) {
            writeln!(w, "{annotation}")?;
        }
    }
    Ok(())
}

fn named<Format: Clone>(formats: &[Format]) -> Vec<Named<Format>> {
    formats
        .iter()
        .enumerate()
        .map(|(i, f)| Named::new(f, format!("field{i}")))
        .collect()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bincode;
#[cfg(test)]
mod tests_json;
