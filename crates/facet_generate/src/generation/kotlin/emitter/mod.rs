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
//! tuples of size 2/3 → `Pair`/`Triple`, and larger tuples to `NTupleN<…>`.
//!
//! # Encoding-dependent output
//!
//! The [`Kotlin`] language tag carries the active [`Encoding`] and a list of
//! [`EmitterPlugin`]s. All encoding-specific behaviour is delegated to those
//! plugins — the emitter itself contains no encoding checks. For example:
//!
//! - `JsonPlugin` supplies `@Serializable` / `@SerialName` type annotations
//!   and inline `@SerialName` annotations for all-unit enum class variants.
//! - `BincodePlugin` supplies `serialize` / `deserialize` methods and
//!   convenience `bincodeSerialize` / `bincodeDeserialize` wrappers.
//! - With no plugins (`Encoding::None`), only plain type declarations are
//!   emitted.
//!
//! # Feature helpers
//!
//! The encoding-independent `TupleArray` helper (`buildList` polyfill for
//! Kotlin < 1.6.0) is inlined here as [`FEATURE_TUPLE_ARRAY`] and emitted
//! when [`Feature::TupleArray`] is set by [`CodeGeneratorConfig::update_from`].
//!
//! Bincode container helpers (`List<T>.serialize`, `Set<T>.serialize`, etc.)
//! are inlined in `BincodePlugin` (`generation/bincode/kotlin.rs`).
//! The JSON `BigInteger` `KSerializer` is inlined in `JsonPlugin`
//! (`generation/json/kotlin.rs`).

use std::{
    collections::BTreeMap,
    io::{Result, Write},
    string::ToString,
    sync::Arc,
};

use heck::ToLowerCamelCase;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Container, Emitter, Encoding, Feature,
        bincode::kotlin::KotlinBincodePlugin,
        indent::{IndentWrite, Newlines},
        json::JsonPlugin,
        module::Module,
        plugin::{EmitContext, EmitterPlugin, VariantInfo},
    },
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName, VariantFormat},
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
/// call. Carries the target [`Encoding`] so emitters can conditionally
/// produce serialization code.
#[derive(Debug, Clone)]
pub struct Kotlin {
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Self>>>,
}

impl Kotlin {
    /// Create a Kotlin language tag, building the appropriate plugins for the
    /// encoding specified in `config`.
    ///
    /// - [`Encoding::Json`] → includes `JsonPlugin`
    /// - [`Encoding::Bincode`] → includes `KotlinBincodePlugin` (resolves JVM
    ///   package names from `config.external_packages`)
    /// - [`Encoding::None`] → no plugins
    #[must_use]
    pub fn new(config: &CodeGeneratorConfig, _registry: &Registry) -> Self {
        let plugins: Vec<Arc<dyn EmitterPlugin<Self>>> = match config.encoding {
            Encoding::Bincode => vec![Arc::new(KotlinBincodePlugin::from_config(config))],
            Encoding::Json => vec![Arc::new(JsonPlugin)],
            Encoding::None => vec![],
        };
        Self { plugins }
    }

    /// Access the plugin list.
    #[must_use]
    pub fn plugins(&self) -> &[Arc<dyn EmitterPlugin<Self>>] {
        &self.plugins
    }
}

impl Emitter<Kotlin> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        let CodeGeneratorConfig {
            module_name,
            features,
            ..
        } = self.config();

        writeln!(w, "package {module_name}")?;
        writeln!(w)?;

        // --- Imports ---
        // Language-level imports that are NOT driven by plugins stay here.
        // Bincode imports are now provided by BincodePlugin::imports().
        let mut imports: Vec<String> = vec![];

        // --- Feature-driven imports (non-plugin) ---
        let mut features_out = vec![];
        for feature in features {
            match feature {
                Feature::BigInt => {
                    // `import java.math.BigInteger` is needed for all encodings,
                    // including `Encoding::None` where no plugin runs.
                    // Plugin-specific BigInt imports (JSON KSerializer, Bincode
                    // Int128) are added by their respective plugins.
                    imports.push("import java.math.BigInteger".to_string());
                }
                Feature::TupleArray => {
                    // TupleArray is encoding-independent — stays in the emitter.
                    write!(features_out, "{FEATURE_TUPLE_ARRAY}")?;
                    writeln!(features_out)?;
                }
                // Bincode feature helpers (ListOfT, SetOfT, MapOfT, OptionOfT, Bytes)
                // are now provided by BincodePlugin::module_helpers() / imports().
                _ => {}
            }
        }

        // --- Plugin imports ---
        for plugin in lang.plugins() {
            imports.extend(plugin.imports(self.config()));
        }

        // --- Plugin module helpers ---
        {
            let mut fw = w.child(&mut features_out);
            for plugin in lang.plugins() {
                plugin.module_helpers(&mut fw, self.config())?;
            }
        }

        let mut imports = imports
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        imports.sort_unstable();
        imports.dedup();
        if !imports.is_empty() {
            for import in imports {
                writeln!(w, "{import}")?;
            }
            writeln!(w)?;
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
                data_object(w, name, None, doc, lang, None)?;
            }
            ContainerFormat::NewTypeStruct(format, doc) => {
                data_class(
                    w,
                    name,
                    None,
                    &[Named::new(format, "value".to_string())],
                    doc,
                    lang,
                    None,
                )?;
            }
            ContainerFormat::TupleStruct(formats, doc) => {
                data_class(w, name, None, &named(formats), doc, lang, None)?;
            }
            ContainerFormat::Struct(fields, doc) => {
                if fields.is_empty() {
                    data_object(w, name, None, doc, lang, None)?;
                } else {
                    data_class(w, name, None, fields, doc, lang, None)?;
                }
            }
            ContainerFormat::Enum(variants, doc) => {
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

        Ok(())
    }
}

impl Emitter<Kotlin> for Named<Format> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        self.doc.write(w, lang)?;

        let name = &self.name.to_lower_camel_case();
        write!(w, "val {name}: ")?;

        self.value.write(w, lang)?;

        // Add = null default only for top-level Option types
        if matches!(self.value, Format::Option(_)) {
            write!(w, " = null")?;
        }

        writeln!(w, ",")
    }
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
                data_object(w, name, Some(interface_name), doc, lang, Some(*index))?;
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
                    Some(interface_name),
                    &[Named::new(inner, "value".to_string())],
                    doc,
                    lang,
                    Some(*index),
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
                    Some(interface_name),
                    &named(formats),
                    doc,
                    lang,
                    Some(*index),
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
                    Some(interface_name),
                    fields,
                    doc,
                    lang,
                    Some(*index),
                )?;
            }
            (VariantFormat::Struct(_fields), VariantContext::EnumClass) => {
                unreachable!("Struct variants are not supported in enum classes")
            }
        }

        Ok(())
    }
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
            Self::Unit => write!(w, "Unit"),
            Self::Bool => write!(w, "Boolean"),
            Self::I8 => write!(w, "Byte"),
            Self::I16 => write!(w, "Short"),
            Self::I32 => write!(w, "Int"),
            Self::I64 => write!(w, "Long"),
            Self::U8 => write!(w, "UByte"),
            Self::U16 => write!(w, "UShort"),
            Self::U32 => write!(w, "UInt"),
            Self::U64 => write!(w, "ULong"),
            Self::I128 | Self::U128 => write!(w, "BigInteger"),
            Self::F32 => write!(w, "Float"),
            Self::F64 => write!(w, "Double"),
            Self::Char | Self::Str => write!(w, "String"),
            Self::Bytes => write!(w, "Bytes"),

            Self::Option(format) => {
                format.write(w, lang)?;
                write!(w, "?")
            }
            Self::Seq(format) => {
                write!(w, "List<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Self::Set(format) => {
                write!(w, "Set<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Self::Map { key, value } => {
                write!(w, "Map<")?;
                key.write(w, lang)?;
                write!(w, ", ")?;
                value.write(w, lang)?;
                write!(w, ">")
            }
            Self::Tuple(formats) => {
                let len = formats.len();
                match len {
                    0 => write!(w, "Unit"),
                    1 => {
                        // A single-element tuple is just the element itself
                        formats[0].write(w, lang)
                    }
                    2 => {
                        write!(w, "Pair<")?;
                        formats[0].write(w, lang)?;
                        write!(w, ", ")?;
                        formats[1].write(w, lang)?;
                        write!(w, ">")
                    }
                    3 => {
                        write!(w, "Triple<")?;
                        formats[0].write(w, lang)?;
                        write!(w, ", ")?;
                        formats[1].write(w, lang)?;
                        write!(w, ", ")?;
                        formats[2].write(w, lang)?;
                        write!(w, ">")
                    }
                    _ => {
                        // For larger tuples, we'll use a data class NTupleN
                        write!(w, "NTuple{len}<")?;
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
                write!(w, "List<")?;
                content.write(w, lang)?;
                write!(w, ">")
            }
        }
    }
}

/// Emits a Kotlin `data object` — used for unit structs and unit variants.
///
/// When `interface` is `Some`, the object implements it (i.e. it is a variant
/// inside a `sealed interface`). Encoding-specific body code (e.g. serialize /
/// deserialize methods) is delegated to plugins via the `type_body` hook.
fn data_object<W: IndentWrite>(
    w: &mut W,
    name: &str,
    interface: Option<&str>,
    doc: &Doc,
    lang: &Kotlin,
    variant_index: Option<usize>,
) -> Result<()> {
    doc.write(w, lang)?;

    write_plugin_annotations(w, name, lang)?;

    write!(w, "data object {name}")?;

    if let Some(interface) = interface {
        write!(w, ": {interface}")?;
    }

    // Plugin type body
    {
        let temp_name = QualifiedTypeName::root(name.to_string());
        let temp_format = ContainerFormat::UnitStruct(Doc::default());
        let temp_container = Container {
            name: &temp_name,
            format: &temp_format,
        };
        let variant_format = VariantFormat::Unit;
        let ctx = if let (Some(parent_name), Some(index)) = (interface, variant_index) {
            EmitContext::for_variant(
                &temp_container,
                VariantInfo {
                    name,
                    index,
                    format: &variant_format,
                    fields: &[],
                    parent_name,
                },
            )
        } else {
            EmitContext::top_level(&temp_container)
        };
        write_plugin_body(w, lang, &ctx)?;
    }

    Ok(())
}

/// Emits a Kotlin `data class` — used for structs (with fields), newtype
/// structs, tuple structs, and non-unit sealed-interface variants.
///
/// When `interface` is `Some`, the class implements it. Encoding-specific
/// body code (e.g. serialize / deserialize methods) is delegated to plugins
/// via the `type_body` hook.
fn data_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    interface: Option<&str>,
    fields: &[Named<Format>],
    doc: &Doc,
    lang: &Kotlin,
    variant_index: Option<usize>,
) -> Result<()> {
    doc.write(w, lang)?;

    write_plugin_annotations(w, name, lang)?;

    writeln!(w, "data class {name}(")?;

    w.indent();
    for field in fields {
        field.write(w, lang)?;
    }
    w.unindent();

    write!(w, ")")?;

    if let Some(interface) = interface {
        write!(w, " : {interface}")?;
    }

    // Plugin type body
    {
        let temp_name = QualifiedTypeName::root(name.to_string());
        let temp_format = ContainerFormat::Struct(fields.to_vec(), Doc::default());
        let temp_container = Container {
            name: &temp_name,
            format: &temp_format,
        };
        let variant_format = VariantFormat::Struct(fields.to_vec());
        let ctx = if let (Some(parent_name), Some(index)) = (interface, variant_index) {
            EmitContext::for_variant(
                &temp_container,
                VariantInfo {
                    name,
                    index,
                    format: &variant_format,
                    fields,
                    parent_name,
                },
            )
        } else {
            EmitContext::top_level(&temp_container)
        };
        write_plugin_body(w, lang, &ctx)?;
    }

    Ok(())
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

    write_plugin_annotations(w, name, lang)?;

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
        let ctx = EmitContext::top_level(container);
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

    write_plugin_annotations(w, name, lang)?;

    write!(w, "sealed interface {name} ")?;
    let mut w = w.block(Newlines::BOTH)?;

    // Plugin type body preamble (before variants)
    {
        let ctx = EmitContext::top_level(container);
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
        let ctx = EmitContext::top_level(container);
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

/// Emits plugin type annotations (e.g. `@Serializable`, `@SerialName`) for a
/// named type. Creates a temporary [`Container`] so that the plugin
/// [`EmitContext`] can be constructed without threading the real container
/// through every helper function.
fn write_plugin_annotations<W: IndentWrite>(w: &mut W, name: &str, lang: &Kotlin) -> Result<()> {
    if lang.plugins().is_empty() {
        return Ok(());
    }
    let temp_name = QualifiedTypeName::root(name.to_string());
    let temp_format = ContainerFormat::UnitStruct(Doc::default());
    let temp_container = Container {
        name: &temp_name,
        format: &temp_format,
    };
    let ctx = EmitContext::top_level(&temp_container);
    for plugin in lang.plugins() {
        for annotation in plugin.type_annotations(&ctx) {
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
