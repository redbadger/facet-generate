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
//! # Encoding-dependent output
//!
//! The [`TypeScript`] language tag carries the active [`Encoding`] and a list
//! of [`EmitterPlugin`]s. All encoding-specific behaviour — serialize /
//! deserialize methods and feature helper snippets — is delegated to those
//! plugins. With no plugins (`Encoding::None`), only plain type declarations
//! are emitted.
//!
//! # Plugins
//!
//! - [`BincodePlugin`](crate::generation::bincode::BincodePlugin) supplies
//!   `serialize` / `deserialize` methods and the Bincode feature helpers.
//! - [`JsonPlugin`](crate::generation::json::JsonPlugin) supplies the same
//!   interface for JSON (the TypeScript Serializer/Deserializer API is
//!   identical for both encodings).
//! - With no plugins (`Encoding::None`), only plain type declarations are
//!   emitted.

#[cfg(test)]
use std::collections::BTreeSet;
use std::{
    collections::BTreeMap,
    io::{Result, Write},
    sync::Arc,
};

use heck::ToUpperCamelCase;

use crate::{
    generation::{
        CodeGeneratorConfig, Container, Emitter, Encoding, PackageLocation, SERDE_NAMESPACE,
        bincode::BincodePlugin,
        indent::{IndentConfig, IndentWrite, IndentedWriter, Newlines},
        json::JsonPlugin,
        module::Module,
        plugin::{BuildPluginsFor, EmitContext, EmitterPlugin, VariantInfo},
    },
    reflection::format::{ContainerFormat, Doc, Format, Named, VariantFormat},
};

/// Language tag for TypeScript code generation.
///
/// Carries a plugin list that controls all encoding-specific behaviour
/// (serialize/deserialize methods, feature helpers, imports). Build the
/// standard plugin list from the config encoding via
/// [`BuildPluginsFor<TypeScript>`], or supply custom plugins with
/// [`with_plugin`](Self::with_plugin).
#[derive(Debug, Clone)]
pub struct TypeScript {
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<Self>>>,
}

impl TypeScript {
    /// Create a TypeScript language tag using the plugins derived from
    /// `config.encoding` via [`BuildPluginsFor<TypeScript>`].
    #[must_use]
    pub fn new(config: &CodeGeneratorConfig, _registry: &crate::Registry) -> Self {
        Self {
            plugins: config.build_plugins_for(),
        }
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

impl BuildPluginsFor<TypeScript> for CodeGeneratorConfig {
    fn build_plugins_for(&self) -> Vec<Arc<dyn EmitterPlugin<TypeScript>>> {
        match self.encoding {
            Encoding::Bincode => vec![Arc::new(BincodePlugin)],
            Encoding::Json => vec![Arc::new(JsonPlugin)],
            Encoding::None => vec![],
        }
    }
}

impl Module {
    fn ts_serde_import_path(&self) -> String {
        self.config()
            .external_packages
            .get(SERDE_NAMESPACE)
            .map_or_else(
                || "./serde".to_string(),
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

    fn ts_namespace_import_path(&self, namespace: &str) -> String {
        self.config().external_packages.get(namespace).map_or_else(
            || format!("../{namespace}"),
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

        if self.config().has_encoding() {
            let import_path = self.ts_serde_import_path();
            writeln!(
                w,
                r#"import {{ Serializer, Deserializer }} from "{import_path}";"#
            )?;
        }

        // Write namespace imports (e.g. `import * as Foo from "../foo";`)
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
                let ctx = EmitContext::top_level(self);
                output_struct_or_variant(w, &ctx, name, &[], doc, lang)
            }
            ContainerFormat::NewTypeStruct(format, doc) => {
                let fields = vec![Named::new(format.as_ref(), "value".to_string())];
                let ctx = EmitContext::top_level(self);
                output_struct_or_variant(w, &ctx, name, &fields, doc, lang)
            }
            ContainerFormat::TupleStruct(formats, doc) => {
                let fields: Vec<_> = formats
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Named::new(f, format!("field{i}")))
                    .collect();
                let ctx = EmitContext::top_level(self);
                output_struct_or_variant(w, &ctx, name, &fields, doc, lang)
            }
            ContainerFormat::Struct(fields, doc) => {
                let ctx = EmitContext::top_level(self);
                output_struct_or_variant(w, &ctx, name, fields, doc, lang)
            }
            ContainerFormat::Enum(variants, doc) => {
                output_enum_container(w, self, name, variants, doc, lang)
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
    let variant_base = ctx.variant.as_ref().map(|v| v.parent_name);

    writeln!(w)?;
    doc.write(w, lang)?;
    if let Some(base) = variant_base {
        write!(w, "export class {base}Variant{name} extends {base} ")?;
    } else {
        write!(w, "export class {name} ")?;
    }
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
        let mut w = w.block(Newlines::BOTH)?;
        if variant_base.is_some() {
            writeln!(w, "super();")?;
        }
    }

    // Plugin type bodies (serialize / deserialize methods).
    for plugin in lang.plugins() {
        plugin.type_body(&mut w as &mut dyn IndentWrite, ctx)?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn output_variant<W: IndentWrite>(
    w: &mut W,
    parent: &Container<'_>,
    base: &str,
    index: u32,
    name: &str,
    variant: &VariantFormat,
    doc: &Doc,
    lang: &TypeScript,
) -> Result<()> {
    let fields: Vec<Named<Format>> = match variant {
        VariantFormat::Unit => Vec::new(),
        VariantFormat::NewType(format) => {
            vec![Named::new(format.as_ref(), "value".to_string())]
        }
        VariantFormat::Tuple(formats) => formats
            .iter()
            .enumerate()
            .map(|(i, f)| Named::new(f, format!("field{i}")))
            .collect(),
        VariantFormat::Struct(fields) => fields.clone(),
        VariantFormat::Variable(_) => panic!("incorrect value"),
    };

    let variant_info = VariantInfo {
        name,
        index: index as usize,
        format: variant,
        fields: &fields,
        parent_name: base,
    };
    let ctx = EmitContext::for_variant(parent, variant_info);
    output_struct_or_variant(w, &ctx, name, &fields, doc, lang)
}

fn output_enum_container<W: IndentWrite>(
    w: &mut W,
    container: &Container<'_>,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    lang: &TypeScript,
) -> Result<()> {
    writeln!(w)?;
    doc.write(w, lang)?;
    write!(w, "export abstract class {name} ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        // Plugin type bodies (abstract serialize + static deserialize switch).
        let ctx = EmitContext::top_level(container);
        for plugin in lang.plugins() {
            plugin.type_body(&mut w as &mut dyn IndentWrite, &ctx)?;
        }
    }
    for (index, variant) in variants {
        output_variant(
            w,
            container,
            name,
            *index,
            &variant.name,
            &variant.value,
            &variant.doc,
            lang,
        )?;
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
