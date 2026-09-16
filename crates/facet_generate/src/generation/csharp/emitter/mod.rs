//! AST-to-C# source rendering.
//!
//! This module implements [`Emitter<CSharp>`](super::super::Emitter) for each
//! node type in the format AST, turning abstract type descriptions into
//! idiomatic C# code.
//!
//! # Emitter implementations
//!
//! | AST node | C# output |
//! |---|---|
//! | [`Module`] | `using` directives, file-scoped `namespace` declaration |
//! | [`Container`] | `sealed record`, `partial class : ObservableObject`, `public enum`, or `abstract record` + `sealed record` variant hierarchy |
//! | [`Named<Format>`](Named) | `[ObservableProperty]` private field (+ `[JsonPropertyName]` for JSON) |
//! | [`Format`] | Inline type expression (`int`, `string`, `ObservableCollection<T>`, …) |
//! | [`Doc`] | `///` XML doc comments |
//!
//! # C# type mapping
//!
//! The [`Format`] emitter maps Rust/reflection types to C# equivalents:
//! `I32` → `int`, `I64` → `long`, `U8` → `byte`, `Str` → `string`,
//! `Bytes` → `byte[]`, `Seq(T)` → `ObservableCollection<T>`,
//! `Set(T)` → `HashSet<T>`, `Map(K,V)` → `Dictionary<K, V>`,
//! `Option(T)` → `T?`, tuples → `(T1, T2, …)`, `TupleArray` → `T[]`,
//! `Unit` → `Unit` (custom readonly record struct).
//!
//! # Plugin-dependent output
//!
//! The [`CSharp`] language tag carries a list of [`EmitterPlugin`]s. All
//! encoding-specific behaviour is delegated to those plugins — the emitter
//! itself contains no encoding checks. For example:
//!
//! - `JsonPlugin` supplies `System.Text.Json` annotations (`[JsonPropertyName]`,
//!   `[JsonPolymorphic]`, `[JsonDerivedType]`, `[JsonConverter]`) plus
//!   `JsonSerde` static helper methods.
//! - `BincodePlugin` supplies `IFacetSerializable`/`IFacetDeserializable<T>`
//!   interface implementations with `Serialize`/`Deserialize` methods and
//!   `BincodeSerialize`/`BincodeDeserialize` wrappers.
//! - With no plugins, only plain MVVM type declarations are emitted.
//!
//! # Feature helpers via `FacetHelpers.cs`
//!
//! Like Kotlin, Swift, and TypeScript, C# uses reusable helper functions for
//! serializing/deserializing generic container types (collections, maps,
//! options, arrays). Instead of per-module snippet files (as the other
//! languages use), C# places these helpers in a single shared runtime file
//! (`Facet/Runtime/Bincode/FacetHelpers.cs`). This works because C#
//! file-scoped namespaces and `using` directives make a helper class in
//! `Facet.Runtime.Bincode` accessible from any generated namespace without
//! duplication. The generated code calls helpers with lambdas rather than
//! emitting inline loops, e.g.:
//!
//! ```csharp
//! FacetHelpers.SerializeCollection(Items, serializer, (item, s) => s.SerializeStr(item));
//! var items = FacetHelpers.DeserializeList(deserializer, d => d.DeserializeStr());
//! ```

use super::naming::builtin;
use std::{
    borrow::Cow,
    io::{Result, Write},
    sync::Arc,
};

use crate::Registry;

use heck::{ToLowerCamelCase, ToUpperCamelCase};

use crate::{
    generation::{
        CodeGeneratorConfig, Container, Emitter,
        indent::{IndentWrite, Newlines},
        module::Module,
        plugin::{EmitContext, EmitterPlugin, any_plugin, collect_from_plugins},
    },
    reflection::format::{
        ContainerFormat, Doc, Format, Named, Namespace, QualifiedTypeName, VariantFormat,
    },
};

/// Language tag for C#.
///
/// Passed to every [`Emitter`](super::super::Emitter) implementation.
/// All encoding-specific behaviour is delegated to plugins — the emitter
/// itself contains no encoding checks.
#[derive(Debug, Clone)]
pub struct CSharp {
    /// The code-generator configuration (namespace, feature flags, etc.).
    pub(crate) config: CodeGeneratorConfig,
    /// Plugins to apply during code generation.
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Self>>>,
}

impl CSharp {
    /// Create a [`CSharp`] language tag with no plugins.
    ///
    /// To add plugins, call [`with_plugin`](Self::with_plugin) after construction.
    #[must_use]
    pub fn new(config: &CodeGeneratorConfig, _registry: &Registry) -> Self {
        Self {
            config: config.clone(),
            plugins: vec![],
        }
    }

    /// Access the code-generator configuration.
    #[must_use]
    pub const fn config(&self) -> &CodeGeneratorConfig {
        &self.config
    }

    /// Append a plugin to this language tag's plugin list.
    ///
    /// Useful for adding custom plugins on top of the defaults returned by
    /// [`new`](Self::new).
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

/// Write the module header — the `using` directives (built-in,
/// plugin-provided, and the caller's `extra_imports`, each a whole
/// `using …;` directive) and the file-scoped `namespace` declaration.
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
    lang: &CSharp,
    extra_imports: &[String],
) -> Result<()> {
    let CodeGeneratorConfig { module_name, .. } = config;
    writeln!(w, "using CommunityToolkit.Mvvm.ComponentModel;")?;
    writeln!(w, "using Facet.Runtime.Serde;")?;
    writeln!(w, "using System.Collections.Generic;")?;
    writeln!(w, "using System.Collections.ObjectModel;")?;
    // Plugin-provided using directives (e.g. Facet.Runtime.Json / Bincode).
    for plugin in lang.plugins() {
        for import in plugin.imports(config) {
            writeln!(w, "{import}")?;
        }
    }
    for import in extra_imports {
        writeln!(w, "{import}")?;
    }
    writeln!(w)?;
    writeln!(w, "namespace {};", namespace_name(module_name))
}

impl Emitter<CSharp> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &CSharp) -> Result<()> {
        write_module_header(w, self.config(), lang, &[])?;
        // Plugin module helpers (e.g. UuidSerde from BincodePlugin).
        // These are emitted per-module file rather than into a shared runtime
        // file because they reference types (e.g. Guid) that may not be in
        // scope in the shared runtime namespace.
        for plugin in lang.plugins() {
            plugin.module_helpers(w, self.config())?;
        }
        writeln!(w)
    }
}

impl Emitter<CSharp> for Container<'_> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &CSharp) -> Result<()> {
        let Container {
            name: QualifiedTypeName { namespace: _, name },
            format,
        } = self;

        match format {
            ContainerFormat::UnitStruct(doc) => write_sealed_record(w, self, name, doc, lang),
            ContainerFormat::NewTypeStruct(format, doc) => write_class(
                w,
                self,
                name,
                &[Named::new(format, "value".to_string())],
                doc,
                lang,
            ),
            ContainerFormat::TupleStruct(formats, doc) => {
                write_class(w, self, name, &named(formats), doc, lang)
            }
            ContainerFormat::Struct(fields, doc) => {
                if fields.is_empty() {
                    write_sealed_record(w, self, name, doc, lang)
                } else {
                    write_class(w, self, name, fields, doc, lang)
                }
            }
            ContainerFormat::Enum(variants, _, doc) => {
                let all_unit_variants = variants
                    .values()
                    .all(|variant| matches!(variant.value, VariantFormat::Unit));
                if all_unit_variants {
                    write_enum(w, self, name, variants, doc, lang)
                } else {
                    let variant_list: Vec<_> = variants.values().cloned().collect();
                    write_variant_record_hierarchy(w, self, name, &variant_list, doc, lang)
                }
            }
        }
    }
}

/// Scan the registry (if available) and collect the **names** of every enum
/// whose variants are all [`VariantFormat::Unit`] (C-style enums).
///
/// These enums are emitted as plain C# `enum` types rather than class
/// hierarchies, so their bincode serialization must go through a static
/// `{Enum}Bincode` helper class instead of instance methods.
///
/// We collect bare type names (`String`) rather than full
/// [`QualifiedTypeName`]s because `update_qualified_names` rewrites the
/// namespace on `Format::TypeName` references without touching registry keys.
/// Within a single module the bare name is unambiguous (types are grouped by
/// namespace via `module::split`).
impl Emitter<CSharp> for Named<Format> {
    /// Write a field declaration without plugin annotations.
    ///
    /// In the code-generation pipeline, fields inside `write_class` are
    /// rendered via `write_field` instead, which also calls
    /// `plugin.field_annotations()`. This impl is kept for completeness.
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &CSharp) -> Result<()> {
        self.doc.write(w, lang)?;
        writeln!(w, "[ObservableProperty]")?;
        writeln!(
            w,
            "private {} _{};",
            csharp_type(&self.value, &lang.config),
            self.name.to_lower_camel_case()
        )
    }
}

/// Write a single field declaration, including plugin-provided annotations
/// (e.g. `[JsonPropertyName]`).
fn write_field<W: IndentWrite>(
    w: &mut W,
    field: &Named<Format>,
    ctx: &EmitContext<'_>,
    lang: &CSharp,
) -> Result<()> {
    field.doc.write(w, lang)?;
    for annotation in collect_from_plugins(lang.plugins(), |p| p.field_annotations(field, ctx)) {
        writeln!(w, "{annotation}")?;
    }
    writeln!(w, "[ObservableProperty]")?;
    writeln!(
        w,
        "private {} _{};",
        csharp_type(&field.value, &lang.config),
        field.name.to_lower_camel_case()
    )
}

impl Emitter<CSharp> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: &CSharp) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }
        Ok(())
    }
}

fn write_sealed_record<W: IndentWrite>(
    w: &mut W,
    container: &Container<'_>,
    name: &str,
    doc: &Doc,
    lang: &CSharp,
) -> Result<()> {
    doc.write(w, lang)?;

    let record_name = name.to_upper_camel_case();
    let ctx = EmitContext::top_level(container, &lang.config);

    let conformances = collect_from_plugins(lang.plugins(), |p| p.type_conformances(&ctx));
    let conforms = if conformances.is_empty() {
        String::new()
    } else {
        format!(" : {}", conformances.join(", "))
    };

    if !any_plugin(lang.plugins(), |p| p.has_type_body(&ctx)) {
        writeln!(w, "public sealed record {record_name}{conforms};")?;
        return write_after_type(w, &ctx, lang);
    }

    write!(w, "public sealed record {record_name}{conforms} ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        for plugin in lang.plugins() {
            plugin.type_body(&mut w as &mut dyn IndentWrite, &ctx)?;
        }
    }

    write_after_type(w, &ctx, lang)
}

/// Run the plugin `after_type` hook for a top-level type, at the indentation
/// level of the type declaration itself.
fn write_after_type<W: IndentWrite>(w: &mut W, ctx: &EmitContext<'_>, lang: &CSharp) -> Result<()> {
    for plugin in lang.plugins() {
        plugin.after_type(w as &mut dyn IndentWrite, ctx)?;
    }
    Ok(())
}

fn write_class<W: IndentWrite>(
    w: &mut W,
    container: &Container<'_>,
    name: &str,
    fields: &[Named<Format>],
    doc: &Doc,
    lang: &CSharp,
) -> Result<()> {
    doc.write(w, lang)?;

    let class_name = name.to_upper_camel_case();
    let ctx = EmitContext::top_level(container, &lang.config);

    let conformances = collect_from_plugins(lang.plugins(), |p| p.type_conformances(&ctx));
    let conforms = if conformances.is_empty() {
        String::new()
    } else {
        format!(", {}", conformances.join(", "))
    };

    write!(
        w,
        "public partial class {class_name} : ObservableObject{conforms} "
    )?;

    let has_plugin_body = any_plugin(lang.plugins(), |p| p.has_type_body(&ctx));

    if fields.is_empty() && !has_plugin_body {
        let _ = w.block(Newlines::CLOSE)?;
        return write_after_type(w, &ctx, lang);
    }

    {
        let mut w = w.block(Newlines::BOTH)?;
        for field in fields {
            write_field(&mut w, field, &ctx, lang)?;
        }

        for plugin in lang.plugins() {
            if plugin.has_type_body(&ctx) {
                if !fields.is_empty() {
                    writeln!(w)?;
                }
                plugin.type_body(&mut w as &mut dyn IndentWrite, &ctx)?;
            }
        }
    }

    write_after_type(w, &ctx, lang)
}

fn write_enum<W: IndentWrite>(
    w: &mut W,
    container: &Container<'_>,
    name: &str,
    variants: &std::collections::BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    lang: &CSharp,
) -> Result<()> {
    let enum_name = name.to_upper_camel_case();
    let ctx = EmitContext::top_level(container, &lang.config);

    doc.write(w, lang)?;

    // Type annotations from plugins (e.g. [JsonConverter(typeof(JsonStringEnumConverter))]).
    for annotation in collect_from_plugins(lang.plugins(), |p| p.type_annotations(&ctx)) {
        writeln!(w, "{annotation}")?;
    }

    write!(w, "public enum {enum_name} ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        let len = variants.len();
        for (index, variant) in variants.values().enumerate() {
            variant.doc.write(&mut w, lang)?;
            write!(w, "{}", variant.name.to_upper_camel_case())?;
            if index + 1 < len {
                writeln!(w, ",")?;
            } else {
                writeln!(w)?;
            }
        }
    }

    // After-type content from plugins (e.g. {EnumName}Bincode static class).
    write_after_type(w, &ctx, lang)
}

fn write_variant_record_hierarchy<W: IndentWrite>(
    w: &mut W,
    container: &Container<'_>,
    name: &str,
    variants: &[Named<VariantFormat>],
    doc: &Doc,
    lang: &CSharp,
) -> Result<()> {
    let base_name = name.to_upper_camel_case();
    let ctx = EmitContext::top_level(container, &lang.config);

    doc.write(w, lang)?;

    // Type annotations from plugins (e.g. [JsonPolymorphic] + [JsonDerivedType(…)]).
    for annotation in collect_from_plugins(lang.plugins(), |p| p.type_annotations(&ctx)) {
        writeln!(w, "{annotation}")?;
    }

    // Type conformances from plugins (e.g. IFacetSerializable, IFacetDeserializable<T>).
    let conformances = collect_from_plugins(lang.plugins(), |p| p.type_conformances(&ctx));
    let conforms = if conformances.is_empty() {
        String::new()
    } else {
        format!(" : {}", conformances.join(", "))
    };

    // `partial` is required when bincode is active — variant partial records must
    // re-open the primary record declaration to add the Serialize override.
    let partial = if conformances.is_empty() {
        ""
    } else {
        " partial"
    };

    write!(w, "public abstract record {base_name}{conforms} ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        write_variant_records(&mut w, base_name.as_str(), variants, partial, lang)?;

        // Plugin type bodies (JSON helpers or Bincode abstract method +
        // partial record overrides).
        for plugin in lang.plugins() {
            plugin.type_body(&mut w as &mut dyn IndentWrite, &ctx)?;
        }
    }

    write_after_type(w, &ctx, lang)
}

/// Write the `sealed record` declaration for each variant of an
/// `abstract record` hierarchy.
fn write_variant_records<W: IndentWrite>(
    w: &mut W,
    base_name: &str,
    variants: &[Named<VariantFormat>],
    partial: &str,
    lang: &CSharp,
) -> Result<()> {
    for variant in variants {
        variant.doc.write(w, lang)?;
        let variant_name = variant.name.to_upper_camel_case();
        write!(w, "public sealed{partial} record {variant_name}")?;
        match &variant.value {
            VariantFormat::Unit => {
                writeln!(w, "() : {base_name};")?;
            }
            VariantFormat::NewType(inner) => {
                writeln!(
                    w,
                    "({} Value) : {};",
                    csharp_type(inner, &lang.config),
                    base_name
                )?;
            }
            VariantFormat::Tuple(values) => {
                write!(w, "(")?;
                for (index, format) in values.iter().enumerate() {
                    if index > 0 {
                        write!(w, ", ")?;
                    }
                    write!(w, "{} Field{}", csharp_type(format, &lang.config), index)?;
                }
                writeln!(w, ") : {base_name};")?;
            }
            VariantFormat::Struct(fields) => {
                write!(w, "(")?;
                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(w, ", ")?;
                    }
                    write!(
                        w,
                        "{} {}",
                        csharp_type(&field.value, &lang.config),
                        field.name.to_upper_camel_case()
                    )?;
                }
                writeln!(w, ") : {base_name};")?;
            }
            VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
        }
        writeln!(w)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Public helpers for plugin authors
// ---------------------------------------------------------------------------

/// Render `format` as the C# type expression the emitter would use for a
/// property of that type — for example `int`, `ObservableCollection<string>`,
/// `Foo?`, `Dictionary<string, Bar>`, or `Other.Child` for a type in another
/// namespace.
///
/// `config` supplies the set of type names the module declares, so a builtin
/// a declaration shadows (`HashSet`, `Dictionary`, …) is written with its
/// `global::` qualified name.
#[must_use]
pub fn render_type(format: &Format, config: &CodeGeneratorConfig) -> String {
    csharp_type(format, config)
}

/// Escapes an identifier when it is a reserved C# keyword.
///
/// C# verbatim identifiers preserve the identifier's spelling while allowing a
/// keyword to be used in declaration and reference positions. Contextual
/// keywords are intentionally omitted because the generated identifiers are
/// method locals, where those words are valid without escaping.
///
/// Plugins that generate parameter or local names from field or variant names
/// should route them through this so the result matches the emitter.
#[must_use]
pub fn escape_identifier(identifier: &str) -> Cow<'_, str> {
    super::naming::RULES.escape(identifier)
}

fn csharp_type(format: &Format, config: &CodeGeneratorConfig) -> String {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qualified_type_name) => format_qualified_type_name(qualified_type_name),
        Format::Unit => builtin("Unit", config).into_owned(),
        Format::Bool => "bool".to_string(),
        Format::I8 => "sbyte".to_string(),
        Format::I16 => "short".to_string(),
        Format::I32 => "int".to_string(),
        Format::I64 => "long".to_string(),
        Format::I128 => builtin("Int128", config).into_owned(),
        Format::U8 => "byte".to_string(),
        Format::U16 => "ushort".to_string(),
        Format::U32 => "uint".to_string(),
        Format::U64 => "ulong".to_string(),
        Format::U128 => builtin("UInt128", config).into_owned(),
        Format::F32 => "float".to_string(),
        Format::F64 => "double".to_string(),
        Format::Char => "char".to_string(),
        Format::Str => "string".to_string(),
        Format::Bytes => "byte[]".to_string(),
        Format::Uuid => builtin("Guid", config).into_owned(),
        Format::Option(inner) => format!("{}?", csharp_type(inner, config)),
        Format::Seq(inner) => format!(
            "{}<{}>",
            builtin("ObservableCollection", config),
            csharp_type(inner, config)
        ),
        Format::Set(inner) => format!(
            "{}<{}>",
            builtin("HashSet", config),
            csharp_type(inner, config)
        ),
        Format::Map { key, value } => {
            format!(
                "{}<{}, {}>",
                builtin("Dictionary", config),
                csharp_type(key, config),
                csharp_type(value, config)
            )
        }
        Format::Tuple(formats) => {
            if formats.is_empty() {
                return builtin("Unit", config).into_owned();
            }
            let values = formats
                .iter()
                .map(|f| csharp_type(f, config))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({values})")
        }
        Format::TupleArray { content, size: _ } => format!("{}[]", csharp_type(content, config)),
    }
}

fn format_qualified_type_name(qualified_type_name: &QualifiedTypeName) -> String {
    match &qualified_type_name.namespace {
        Namespace::Root => qualified_type_name.name.to_upper_camel_case(),
        Namespace::Named(namespace) => {
            format!(
                "{}.{}",
                namespace_name(namespace),
                qualified_type_name.name.to_upper_camel_case()
            )
        }
    }
}

fn namespace_name(namespace: &str) -> String {
    namespace
        .split('.')
        .map(str::to_upper_camel_case)
        .collect::<Vec<_>>()
        .join(".")
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
