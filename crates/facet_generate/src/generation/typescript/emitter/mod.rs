//! AST-to-TypeScript source rendering.
//!
//! This module implements [`Emitter<TypeScript>`](super::super::Emitter) for
//! each node type in the format AST, turning abstract type descriptions into
//! idiomatic TypeScript code.
//!
//! # Emitter implementations
//!
//! | AST node | TypeScript output |
//! |---|---|
//! | [`Module`] | `import` statements, type aliases, feature helpers |
//! | [`Container`] | `export class` or `export abstract class` + variant subclasses |
//! | [`Named<Format>`](Named) | `public` property declaration |
//! | [`Format`] | Inline type expression (`number`, `string`, `Array<T>`, …) |
//! | [`Doc`] | `///` doc comments |
//! | `(Named<VariantFormat>, …)` | Enum variant subclass extending the abstract base |
//!
//! # TypeScript type mapping
//!
//! The [`Format`] emitter maps Rust/reflection types to TypeScript equivalents
//! via type aliases — for example `I32` → `number` (via `type int32 = number`),
//! `Str` → `string`, `Seq(T)` → `T[]` (via `type Seq<T> = T[]`),
//! `Option(T)` → `Optional<T>` (i.e. `T | null`), `Map(K,V)` → `Map<K, V>`,
//! tuples → `[A, B]` (via `Tuple<[…]>`), fixed-size arrays → `ListTuple<[T]>`.
//!
//! # Plugin-dependent output
//!
//! The [`TypeScript`] language tag carries a list of [`EmitterPlugin`]s.
//! All encoding-specific behaviour — serialize / deserialize methods and
//! feature helper snippets — is delegated to those plugins. With no plugins,
//! only plain type declarations are emitted.
//!
//! # Plugins
//!
//! - [`BincodePlugin`](crate::generation::bincode::BincodePlugin) supplies
//!   `serialize` / `deserialize` methods and the Bincode feature helpers.
//! - [`JsonPlugin`](crate::generation::json::JsonPlugin) supplies static
//!   `toJson` / `fromJson` and `jsonSerialize` / `jsonDeserialize` methods
//!   (functions beside an enum) that match `serde_json`.
//! - With no plugins, only plain type declarations are emitted.

#[cfg(test)]
use std::collections::BTreeSet;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{Result, Write},
    sync::Arc,
};

use heck::{ToLowerCamelCase, ToUpperCamelCase};

use super::naming::{self, builtin};
use crate::{
    generation::{
        CodeGeneratorConfig, Container, Emitter, PackageLocation,
        indent::{IndentConfig, IndentWrite, IndentedWriter, Newlines},
        module::Module,
        naming::qualify_helper,
        plugin::{EmitContext, EmitterPlugin, collect_from_plugins},
    },
    reflection::format::{
        ContainerFormat, Doc, EnumTagging, Format, Named, QualifiedTypeName, VariantFormat,
    },
};

/// Language tag for TypeScript code generation.
///
/// Carries a plugin list that controls all encoding-specific behaviour
/// (serialize/deserialize methods, feature helpers, imports). Use
/// [`with_plugin`](Self::with_plugin) to add plugins.
#[derive(Debug, Clone)]
pub struct TypeScript {
    pub(crate) config: CodeGeneratorConfig,
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Self>>>,
}

impl TypeScript {
    /// Create a TypeScript language tag with no plugins.
    ///
    /// To add plugins, call [`with_plugin`](Self::with_plugin) after construction.
    #[must_use]
    pub fn new(config: &CodeGeneratorConfig, _registry: &crate::Registry) -> Self {
        Self {
            config: config.clone(),
            plugins: vec![],
        }
    }

    /// Access the config.
    #[must_use]
    pub const fn config(&self) -> &CodeGeneratorConfig {
        &self.config
    }

    /// Access the plugin list.
    #[must_use]
    pub fn plugins(&self) -> &[Arc<dyn EmitterPlugin<Self>>] {
        &self.plugins
    }

    /// Add a plugin to this language tag, returning the modified tag.
    ///
    /// Plugins are invoked in the order they were added.
    #[must_use]
    pub fn with_plugin(mut self, plugin: Arc<dyn EmitterPlugin<Self>>) -> Self {
        self.plugins.push(plugin);
        self
    }
}

impl Module {
    fn ts_namespace_import_path(&self, namespace: &str) -> String {
        self.config().external_packages.get(namespace).map_or_else(
            || format!("./{namespace}"),
            |path| match &path.location {
                PackageLocation::Path(_) => {
                    let name = &path.for_namespace;
                    path.module_name
                        .as_ref()
                        .map_or_else(|| name.clone(), |mod_name| format!("{name}/{mod_name}"))
                }
                PackageLocation::Url(_) => path.for_namespace.clone(),
            },
        )
    }
}

impl Emitter<TypeScript> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &TypeScript) -> Result<()> {
        let CodeGeneratorConfig {
            referenced_namespaces,
            used_format_types,
            ..
        } = self.config();

        // Plugin imports (e.g. `import type { Serializer, Deserializer }` from the
        // bincode or json plugin).
        for import in collect_from_plugins(lang.plugins(), |p| p.imports(self.config())) {
            writeln!(w, "{import}")?;
        }

        // Write namespace imports (e.g. `import * as Foo from "./foo";`)
        let mut import_paths: BTreeMap<String, String> = BTreeMap::new();
        for namespace in referenced_namespaces {
            let import_path = self.ts_namespace_import_path(namespace);
            import_paths.insert(namespace.to_upper_camel_case(), import_path);
        }
        for (namespace, path) in import_paths {
            writeln!(w, r#"import * as {namespace} from "{path}";"#)?;
        }

        // Write type aliases (e.g. `type bool = boolean;`)
        let alias_map = BTreeMap::from(TYPE_ALIASES);
        let aliases: Vec<String> = used_format_types
            .iter()
            .filter_map(|k| {
                alias_map.get(k.as_str()).map(|s| {
                    qualify_helper(s, naming::QUALIFIED, |name| {
                        naming::shadows(name, self.config())
                    })
                    .into_owned()
                })
            })
            .collect();
        if !aliases.is_empty() {
            writeln!(w, "{}", aliases.join("\n"))?;
        }

        // Plugin module helpers (feature helper snippets).
        for plugin in lang.plugins() {
            plugin.module_helpers(w, self.config())?;
        }

        Ok(())
    }
}

impl Emitter<TypeScript> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: &TypeScript) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }

        Ok(())
    }
}

impl Emitter<TypeScript> for Container<'_> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &TypeScript) -> Result<()> {
        let Container {
            name: qualified_name,
            format,
            ..
        } = self;
        let name = &qualified_name.name;

        if let ContainerFormat::Enum(variants, tagging, doc) = format {
            // `output_enum_container` runs the after-type hook itself, after
            // the union type, the constructors and the match helper.
            return output_enum_container(w, self, name, variants, tagging, doc, lang);
        }

        let (fields, doc): (Vec<Named<Format>>, &Doc) = match format {
            ContainerFormat::UnitStruct(doc) => (vec![], doc),
            ContainerFormat::NewTypeStruct(format, doc) => {
                (vec![Named::new(format.as_ref(), "value".to_string())], doc)
            }
            ContainerFormat::TupleStruct(formats, doc) => (
                formats
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Named::new(f, format!("field{i}")))
                    .collect(),
                doc,
            ),
            ContainerFormat::Struct(fields, doc) => (fields.clone(), doc),
            ContainerFormat::Enum(_, _, _) => unreachable!("handled above"),
        };

        let ctx = EmitContext::top_level(self, &lang.config);
        output_struct_or_variant(w, &ctx, name, &fields, doc, lang)?;

        // Plugin after-type hook — fires once per top-level type.
        for plugin in lang.plugins() {
            plugin.after_type(w as &mut dyn IndentWrite, &ctx)?;
        }

        Ok(())
    }
}

impl Emitter<TypeScript> for Format {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &TypeScript) -> Result<()> {
        match self {
            Self::TypeName(type_) => {
                write!(
                    w,
                    "{}",
                    type_.format(ToUpperCamelCase::to_upper_camel_case, ".")
                )
            }
            Self::Unit => write!(w, "unit"),
            Self::Bool => write!(w, "bool"),
            Self::I8 => write!(w, "int8"),
            Self::I16 => write!(w, "int16"),
            Self::I32 => write!(w, "int32"),
            Self::I64 => write!(w, "int64"),
            Self::I128 => write!(w, "int128"),
            Self::U8 => write!(w, "uint8"),
            Self::U16 => write!(w, "uint16"),
            Self::U32 => write!(w, "uint32"),
            Self::U64 => write!(w, "uint64"),
            Self::U128 => write!(w, "uint128"),
            Self::F32 => write!(w, "float32"),
            Self::F64 => write!(w, "float64"),
            Self::Char => write!(w, "char"),
            Self::Str => write!(w, "str"),
            Self::Bytes => write!(w, "bytes"),
            Self::Uuid => write!(w, "Uuid"),

            Self::Option(format) => {
                write!(w, "Optional<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Self::Seq(format) | Self::Set(format) => {
                write!(w, "Seq<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Self::Map { key, value } => {
                write!(w, "{}<", builtin("Map", &lang.config))?;
                key.write(w, lang)?;
                write!(w, ",")?;
                value.write(w, lang)?;
                write!(w, ">")
            }
            Self::Tuple(formats) => {
                write!(w, "Tuple<[")?;
                for (i, f) in formats.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    f.write(w, lang)?;
                }
                write!(w, "]>")
            }
            Self::TupleArray { content, .. } => {
                write!(w, "ListTuple<[")?;
                content.write(w, lang)?;
                write!(w, "]>")
            }
            Self::Variable(_) => panic!("unexpected value"),
        }
    }
}

impl Emitter<TypeScript> for Named<Format> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &TypeScript) -> Result<()> {
        write!(w, "public {}: ", naming::property_key(&self.name))?;
        self.value.write(w, lang)
    }
}

/// Render `format` as the TypeScript type expression the emitter would use
/// for a property of that type — for example `int32`, `Seq<str>`,
/// `Optional<Foo>`, `Map<str,Bar>`, or `Other.Child` for a type in another
/// namespace.
///
/// The type names in `format` must be in the emitter's spelling: a format
/// from [`EmitContext`] already is, and a name a plugin knows from elsewhere
/// goes through [`requalify`] first.
///
/// `config` supplies the set of type names the module declares, so a global a
/// declaration shadows (`Map`, …) is reached through `globalThis`.
#[must_use]
pub fn render_type(format: &Format, config: &CodeGeneratorConfig) -> String {
    let lang = TypeScript {
        config: config.clone(),
        plugins: vec![],
    };
    quote_type(format, &lang)
}

/// The spelling the emitter gives a reference to `name`, a type name in
/// registry spelling, from the module `config` describes.
///
/// Before emitting a module the generator rewrites every type reference in its
/// registry with this rule: a type of the module's own is written bare
/// (`Named("kv")/Row` → `Row` inside `kv`), and a ROOT type seen from a
/// namespaced module is reached through the root package's namespace import
/// (`Root/Event` → `Named("example")/Event`, rendered `Example.Event`, inside
/// `kv` under root package `example`), when the config knows the root
/// package ([`CodeGeneratorConfig::parent`], which the installer sets).
/// Everything else is unchanged.
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
/// [`write_serialize_value`](crate::generation::bincode::typescript::write_serialize_value),
/// [`CodeGeneratorConfig::is_enum`] /
/// [`is_unit_enum`](CodeGeneratorConfig::is_unit_enum), or any other function
/// that expects the emitter's spelling. Without it an enum lookup misses, and
/// an enum is serialized as though it were a class.
///
/// To requalify every type name inside a format, use [`requalify_format`].
///
/// Apply it once, to a registry-spelled name: it is not idempotent. Inside a
/// namespaced module a second pass would qualify the module's own types,
/// which the first made bare, with the root package.
#[must_use]
pub fn requalify(config: &CodeGeneratorConfig, name: &QualifiedTypeName) -> QualifiedTypeName {
    super::generator::TypeScriptCodeGenerator::requalify(config, name)
}

/// Requalifies every type name in `format` with [`requalify`], as the
/// generator does to each container of the module before emitting it.
///
/// Apply it exactly once, to a format in registry spelling such as one from
/// [`RegistryBuilder::format_of`](crate::reflection::RegistryBuilder::format_of).
/// The formats in [`EmitContext`] are already requalified; requalifying one
/// again qualifies, inside a namespaced module, the module's own types with the
/// root package.
///
/// ```
/// use facet_generate::{
///     generation::{CodeGeneratorConfig, typescript},
///     reflection::format::{Format, QualifiedTypeName},
/// };
///
/// let mut config = CodeGeneratorConfig::new("kv".to_string());
/// config.parent = Some("example".to_string());
///
/// let event = QualifiedTypeName::root("Event".to_string());
/// let mut format = Format::Option(Box::new(Format::TypeName(event)));
/// typescript::requalify_format(&config, &mut format);
///
/// assert_eq!(typescript::render_type(&format, &config), "Optional<Example.Event>");
/// ```
pub fn requalify_format(config: &CodeGeneratorConfig, format: &mut Format) {
    super::generator::TypeScriptCodeGenerator::requalify_type_names(config, format);
}

/// The TypeScript binding name the emitter gives to a constructor parameter
/// or a `const` local derived from `name`.
///
/// Reserved words are illegal as binding identifiers and cannot be quoted, so
/// they are renamed with a trailing underscore (`default` → `default_`), and
/// each character a name that is no identifier at all cannot hold becomes an
/// underscore (`with-dash` → `with_dash`).
/// Property and wire names are *not* renamed — they stay identical to the
/// Rust names — so a renamed binding is paired with an explicit
/// `this.name = name_;` assignment or a `name: name_` object entry.
///
/// Plugins that emit a parameter or a `const` declaration from a field name
/// should route it through this so the result matches the emitter.
#[must_use]
pub fn param_name(name: &str) -> Cow<'_, str> {
    if naming::is_identifier(name) {
        return naming::RULES.escape(name);
    }
    let mut binding: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '$' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if !binding.starts_with(|c: char| c.is_alphabetic() || c == '_' || c == '$') {
        binding.insert(0, '_');
    }
    Cow::Owned(binding)
}

/// Returns `true` if `name` is a TypeScript reserved word, and therefore
/// illegal as a binding identifier.
#[must_use]
pub fn is_reserved_word(name: &str) -> bool {
    naming::RULES.is_reserved(name)
}

/// Render a type expression to a string (used for constructor argument types).
fn quote_type(format: &Format, lang: &TypeScript) -> String {
    let mut buf = Vec::new();
    let mut w = IndentedWriter::new(&mut buf, IndentConfig::Space(0));
    format
        .write(&mut w, lang)
        .expect("writing to Vec should not fail");
    String::from_utf8(buf).expect("type expression should be valid UTF-8")
}

fn output_struct_or_variant<W: IndentWrite>(
    w: &mut W,
    ctx: &EmitContext<'_>,
    name: &str,
    fields: &[Named<Format>],
    doc: &Doc,
    lang: &TypeScript,
) -> Result<()> {
    writeln!(w)?;
    doc.write(w, lang)?;
    write!(w, "export class {name} ")?;
    let mut w = w.block(Newlines::BOTH)?;

    // A field whose name is a reserved word, or no identifier at all, cannot
    // be a parameter property: the parameter is a binding identifier. When
    // any field needs renaming the
    // whole class switches to the explicit style — every field declared, every
    // parameter plain, every assignment written in the constructor body — so
    // that declaration order and property-insertion order still match the
    // registry. A class with no reserved field name keeps its parameter
    // properties and is emitted exactly as before.
    let explicit = fields.iter().any(|f| param_name(&f.name) != f.name);

    if explicit {
        for field in fields {
            writeln!(
                w,
                "public {}: {};",
                naming::property_key(&field.name),
                quote_type(&field.value, lang)
            )?;
        }
        writeln!(w)?;
    }

    let args: Vec<String> = fields
        .iter()
        .map(|f| {
            let type_str = quote_type(&f.value, lang);
            if explicit {
                format!("{}: {}", param_name(&f.name), type_str)
            } else {
                format!("public {}: {}", f.name, type_str)
            }
        })
        .collect();
    let args = args.join(", ");
    write!(w, "constructor ({args}) ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        if explicit {
            for field in fields {
                writeln!(
                    w,
                    "{} = {};",
                    naming::member("this", &field.name),
                    param_name(&field.name)
                )?;
            }
        }
    }

    for plugin in lang.plugins() {
        plugin.type_body(&mut w as &mut dyn IndentWrite, ctx)?;
    }

    Ok(())
}

fn tag_field_name(tagging: &EnumTagging) -> &str {
    match tagging {
        EnumTagging::External => "kind",
        EnumTagging::Internal { tag } | EnumTagging::Adjacent { tag, .. } => tag.as_str(),
    }
}

fn write_variant_type_expr<W: std::io::Write>(
    w: &mut W,
    variant_name: &str,
    variant: &VariantFormat,
    tag_field: &str,
    content_field: Option<&str>,
    is_internal: bool,
    lang: &TypeScript,
) -> Result<()> {
    match variant {
        VariantFormat::Unit => {
            write!(w, r#"{{ {tag_field}: "{variant_name}" }}"#)?;
        }
        VariantFormat::NewType(inner) => {
            let type_str = quote_type(inner, lang);
            if let Some(content) = content_field {
                write!(
                    w,
                    r#"{{ {tag_field}: "{variant_name}"; {content}: {type_str} }}"#
                )?;
            } else if is_internal && matches!(inner.as_ref(), Format::TypeName(_)) {
                write!(w, r#"{{ {tag_field}: "{variant_name}" }} & {type_str}"#)?;
            } else {
                write!(
                    w,
                    r#"{{ {tag_field}: "{variant_name}"; value: {type_str} }}"#
                )?;
            }
        }
        VariantFormat::Tuple(formats) => {
            if let Some(content) = content_field {
                let types: Vec<String> = formats.iter().map(|f| quote_type(f, lang)).collect();
                write!(
                    w,
                    r#"{{ {tag_field}: "{variant_name}"; {content}: [{}] }}"#,
                    types.join(", ")
                )?;
            } else {
                write!(w, r#"{{ {tag_field}: "{variant_name}""#)?;
                for (i, f) in formats.iter().enumerate() {
                    write!(w, "; field{i}: {}", quote_type(f, lang))?;
                }
                write!(w, " }}")?;
            }
        }
        VariantFormat::Struct(fields) => {
            if let Some(content) = content_field {
                write!(w, r#"{{ {tag_field}: "{variant_name}"; {content}: {{ "#)?;
                for field in fields {
                    write!(
                        w,
                        "{}: {}; ",
                        naming::property_key(&field.name),
                        quote_type(&field.value, lang)
                    )?;
                }
                write!(w, "}} }}")?;
            } else {
                write!(w, r#"{{ {tag_field}: "{variant_name}""#)?;
                for field in fields {
                    write!(
                        w,
                        "; {}: {}",
                        naming::property_key(&field.name),
                        quote_type(&field.value, lang)
                    )?;
                }
                write!(w, " }}")?;
            }
        }
        VariantFormat::Variable(_) => panic!("unexpected variable format"),
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_variant_constructor<W: std::io::Write>(
    w: &mut W,
    enum_name: &str,
    variant_name: &str,
    variant: &VariantFormat,
    tag_field: &str,
    content_field: Option<&str>,
    is_internal: bool,
    lang: &TypeScript,
) -> Result<()> {
    let fn_name = format!(
        "{}{}",
        enum_name.to_lower_camel_case(),
        variant_name.to_upper_camel_case(),
    );

    let (params_str, object_str) = match variant {
        VariantFormat::Unit => (
            String::new(),
            format!(r#"{{ {tag_field}: "{variant_name}" }}"#),
        ),
        VariantFormat::NewType(inner) => {
            let type_str = quote_type(inner, lang);
            let obj = if let Some(content) = content_field {
                format!(r#"{{ {tag_field}: "{variant_name}", {content}: value }}"#)
            } else if is_internal && matches!(inner.as_ref(), Format::TypeName(_)) {
                format!(r#"{{ {tag_field}: "{variant_name}", ...value }}"#)
            } else {
                format!(r#"{{ {tag_field}: "{variant_name}", value }}"#)
            };
            (format!("value: {type_str}"), obj)
        }
        VariantFormat::Tuple(formats) => {
            let params: Vec<String> = formats
                .iter()
                .enumerate()
                .map(|(i, f)| format!("field{i}: {}", quote_type(f, lang)))
                .collect();
            let field_names: Vec<String> =
                (0..formats.len()).map(|i| format!("field{i}")).collect();
            let obj = if let Some(content) = content_field {
                format!(
                    r#"{{ {tag_field}: "{variant_name}", {content}: [{}] }}"#,
                    field_names.join(", ")
                )
            } else {
                format!(
                    r#"{{ {tag_field}: "{variant_name}", {} }}"#,
                    field_names.join(", ")
                )
            };
            (params.join(", "), obj)
        }
        VariantFormat::Struct(fields) => {
            let params: Vec<String> = fields
                .iter()
                .map(|f| format!("{}: {}", param_name(&f.name), quote_type(&f.value, lang)))
                .collect();
            // Object-literal shorthand is illegal for a renamed binding, so
            // those fields are written out in full (`default: default_`).
            let field_names: Vec<String> = fields
                .iter()
                .map(|f| {
                    let binding = param_name(&f.name);
                    if binding == f.name {
                        f.name.clone()
                    } else {
                        format!("{}: {binding}", naming::property_key(&f.name))
                    }
                })
                .collect();
            let obj = if let Some(content) = content_field {
                format!(
                    r#"{{ {tag_field}: "{variant_name}", {content}: {{ {} }} }}"#,
                    field_names.join(", ")
                )
            } else {
                format!(
                    r#"{{ {tag_field}: "{variant_name}", {} }}"#,
                    field_names.join(", ")
                )
            };
            (params.join(", "), obj)
        }
        VariantFormat::Variable(_) => panic!("unexpected variable format"),
    };

    writeln!(
        w,
        "export const {fn_name} = ({params_str}): {enum_name} => ({object_str});"
    )?;
    Ok(())
}

fn write_match_function<W: std::io::Write>(
    w: &mut W,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    tag_field: &str,
) -> Result<()> {
    writeln!(w, "export function match{name}<R>(value: {name}, cases: {{")?;
    for variant in variants.values() {
        let vname = &variant.name;
        let key = naming::property_key(vname);
        writeln!(
            w,
            r#"    {key}: (v: Extract<{name}, {{ {tag_field}: "{vname}" }}>) => R;"#
        )?;
    }
    writeln!(w, "}}): R {{")?;
    writeln!(
        w,
        r#"    return cases[value.{tag_field} as {name}["{tag_field}"]](value as never);"#
    )?;
    writeln!(w, "}}")
}

fn output_enum_container<W: IndentWrite>(
    w: &mut W,
    container: &Container<'_>,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    tagging: &EnumTagging,
    doc: &Doc,
    lang: &TypeScript,
) -> Result<()> {
    let tag_field = tag_field_name(tagging);
    let content_field: Option<&str> = match tagging {
        EnumTagging::Adjacent { content, .. } => Some(content.as_str()),
        _ => None,
    };
    let is_internal = matches!(tagging, EnumTagging::Internal { .. });

    writeln!(w)?;
    doc.write(w, lang)?;

    // 1. Union type alias
    write!(w, "export type {name} =")?;
    for variant in variants.values() {
        writeln!(w)?;
        write!(w, "    | ")?;
        write_variant_type_expr(
            w,
            &variant.name,
            &variant.value,
            tag_field,
            content_field,
            is_internal,
            lang,
        )?;
    }
    writeln!(w, ";")?;

    // 2. Constructor helpers
    for variant in variants.values() {
        writeln!(w)?;
        write_variant_constructor(
            w,
            name,
            &variant.name,
            &variant.value,
            tag_field,
            content_field,
            is_internal,
            lang,
        )?;
    }

    // 3. Match function
    writeln!(w)?;
    write_match_function(w, name, variants, tag_field)?;

    // 4. Plugin after_type hook (for standalone serialize/deserialize functions)
    let ctx = EmitContext::top_level(container, &lang.config);
    for plugin in lang.plugins() {
        plugin.after_type(w as &mut dyn IndentWrite, &ctx)?;
    }

    Ok(())
}

const TYPE_ALIASES: [(&str, &str); 21] = [
    ("unit", "type unit = null;"),
    ("bool", "type bool = boolean;"),
    ("int8", "type int8 = number;"),
    ("int16", "type int16 = number;"),
    ("int32", "type int32 = number;"),
    ("int64", "type int64 = bigint;"),
    ("int128", "type int128 = bigint;"),
    ("uint8", "type uint8 = number;"),
    ("uint16", "type uint16 = number;"),
    ("uint32", "type uint32 = number;"),
    ("uint64", "type uint64 = bigint;"),
    ("uint128", "type uint128 = bigint;"),
    ("float32", "type float32 = number;"),
    ("float64", "type float64 = number;"),
    ("char", "type char = string;"),
    ("str", "type str = string;"),
    ("bytes", "type bytes = Uint8Array;"),
    ("option", "type Optional<T> = T | null;"),
    ("seq", "type Seq<T> = T[];"),
    ("tuple", "type Tuple<T extends any[]> = T;"),
    (
        "list_tuple",
        "type ListTuple<T extends any[]> = Tuple<T>[];",
    ),
];

#[cfg(test)]
fn format_type_aliases(input: &BTreeSet<String>) -> String {
    let map = BTreeMap::from(TYPE_ALIASES);
    input
        .iter()
        .filter_map(|k| map.get(k.as_str()).map(|s| (*s).to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
#[cfg(test)]
mod tests_bincode;
#[cfg(test)]
mod tests_json;
