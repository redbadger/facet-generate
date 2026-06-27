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
//! - [`JsonPlugin`](crate::generation::json::JsonPlugin) supplies the same
//!   interface for JSON (the TypeScript Serializer/Deserializer API is
//!   identical for both encodings).
//! - With no plugins, only plain type declarations are emitted.

#[cfg(test)]
use std::collections::BTreeSet;
use std::{
    collections::BTreeMap,
    io::{Result, Write},
    sync::Arc,
};

use heck::{ToLowerCamelCase, ToUpperCamelCase};

use crate::{
    generation::{
        CodeGeneratorConfig, Container, Emitter, PackageLocation,
        indent::{IndentConfig, IndentWrite, IndentedWriter, Newlines},
        module::Module,
        plugin::{EmitContext, EmitterPlugin, collect_from_plugins},
    },
    reflection::format::{ContainerFormat, Doc, EnumTagging, Format, Named, VariantFormat},
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

        // Plugin imports (e.g. `import { Serializer, Deserializer }` from the
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
            .filter_map(|k| alias_map.get(k.as_str()).map(|s| (*s).to_string()))
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

        match format {
            ContainerFormat::UnitStruct(doc) => {
                let ctx = EmitContext::top_level(self, &lang.config);
                output_struct_or_variant(w, &ctx, name, &[], doc, lang)
            }
            ContainerFormat::NewTypeStruct(format, doc) => {
                let fields = vec![Named::new(format.as_ref(), "value".to_string())];
                let ctx = EmitContext::top_level(self, &lang.config);
                output_struct_or_variant(w, &ctx, name, &fields, doc, lang)
            }
            ContainerFormat::TupleStruct(formats, doc) => {
                let fields: Vec<_> = formats
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Named::new(f, format!("field{i}")))
                    .collect();
                let ctx = EmitContext::top_level(self, &lang.config);
                output_struct_or_variant(w, &ctx, name, &fields, doc, lang)
            }
            ContainerFormat::Struct(fields, doc) => {
                let ctx = EmitContext::top_level(self, &lang.config);
                output_struct_or_variant(w, &ctx, name, fields, doc, lang)
            }
            ContainerFormat::Enum(variants, tagging, doc) => {
                output_enum_container(w, self, name, variants, tagging, doc, lang)
            }
        }
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
                write!(w, "Map<")?;
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
        write!(w, "public {}: ", &self.name)?;
        self.value.write(w, lang)
    }
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

    let args: Vec<String> = fields
        .iter()
        .map(|f| {
            let type_str = quote_type(&f.value, lang);
            format!("public {}: {}", &f.name, type_str)
        })
        .collect();
    let args = args.join(", ");
    write!(w, "constructor ({args}) ")?;
    {
        let _w = w.block(Newlines::BOTH)?;
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
                    write!(w, "{}: {}; ", field.name, quote_type(&field.value, lang))?;
                }
                write!(w, "}} }}")?;
            } else {
                write!(w, r#"{{ {tag_field}: "{variant_name}""#)?;
                for field in fields {
                    write!(w, "; {}: {}", field.name, quote_type(&field.value, lang))?;
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
                .map(|f| format!("{}: {}", f.name, quote_type(&f.value, lang)))
                .collect();
            let field_names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
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
        writeln!(
            w,
            r#"    {vname}: (v: Extract<{name}, {{ {tag_field}: "{vname}" }}>) => R;"#
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
