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
//! The [`TypeScript`] language tag carries the active [`Encoding`] and
//! [`InstallTarget`]. Both JSON and Bincode use the same
//! `Serializer`/`Deserializer` interface pattern with hand-written
//! `serialize`/`deserialize` methods. When encoding is `None`, only plain
//! type declarations are emitted.
//!
//! # Feature helpers (`features/` directory)
//!
//! TypeScript has `Array`, `Set`, `Map`, and union types built in, but the
//! Serde `Serializer`/`Deserializer` runtime only handles primitives and
//! user-defined types (which get their own `serialize`/`deserialize` methods).
//! The feature helpers are TypeScript functions that bridge this gap — they
//! teach the serde runtime how to length-prefix and iterate over generic
//! containers.
//!
//! | Helper | What it provides | When included |
//! |---|---|---|
//! | `ArrayOfT.ts` | `serializeArray`/`deserializeArray` | Bincode or JSON + `Seq` type used |
//! | `SetOfT.ts` | `serializeSet`/`deserializeSet` | Bincode or JSON + `Set` type used |
//! | `MapOfT.ts` | `serializeMap`/`deserializeMap` | Bincode or JSON + `Map` type used |
//! | `OptionOfT.ts` | `serializeOption`/`deserializeOption` | Bincode or JSON + `Option` type used |
//! | `TupleArray.ts` | Fixed-size array support | `TupleArray` type used |
//!
//! These `.ts` snippets are embedded at compile time via `include_bytes!`
//! and written into the file header by the [`Module`] emitter when the
//! corresponding [`Feature`] flag is active (discovered automatically by
//! [`CodeGeneratorConfig::update_from`]).

#[cfg(test)]
use std::collections::BTreeSet;
use std::{
    collections::BTreeMap,
    io::{Result, Write},
};

use heck::ToUpperCamelCase;

use crate::{
    generation::{
        CodeGeneratorConfig, Container, Emitter, Encoding, Feature, PackageLocation,
        SERDE_NAMESPACE,
        indent::{IndentConfig, IndentWrite, IndentedWriter, Newlines},
        module::Module,
    },
    reflection::format::{ContainerFormat, Doc, Format, Named, VariantFormat},
};

use super::InstallTarget;

const FEATURE_LIST_OF_T: &[u8] = include_bytes!("features/ArrayOfT.ts");
const FEATURE_MAP_OF_T: &[u8] = include_bytes!("features/MapOfT.ts");
const FEATURE_OPTION_OF_T: &[u8] = include_bytes!("features/OptionOfT.ts");
const FEATURE_SET_OF_T: &[u8] = include_bytes!("features/SetOfT.ts");
const FEATURE_TUPLE_ARRAY: &[u8] = include_bytes!("features/TupleArray.ts");

/// Language tag for TypeScript code generation.
///
/// Carries the active [`Encoding`] (None / Bincode / Json) and
/// [`InstallTarget`] (Node / Deno) so emitter implementations can adapt
/// their output accordingly.
#[derive(Debug, Clone)]
pub struct TypeScript {
    pub encoding: Encoding,
    pub target: InstallTarget,
}

impl TypeScript {
    #[must_use]
    pub fn new(encoding: Encoding) -> Self {
        Self {
            encoding,
            target: InstallTarget::Node,
        }
    }

    /// Which `InstallTarget` to use (Node or Deno)
    #[must_use]
    pub fn with_target(mut self, target: InstallTarget) -> Self {
        self.target = target;
        self
    }
}

impl Module {
    fn ts_serde_import_path(&self, target: InstallTarget) -> String {
        let serde = target.serde_import_path();
        if let Some(path) = self.config().external_packages.get(SERDE_NAMESPACE) {
            match &path.location {
                PackageLocation::Path(_) => {
                    let name = &path.for_namespace;
                    if let Some(mod_name) = &path.module_name {
                        format!("{name}/{mod_name}")
                    } else {
                        name.clone()
                    }
                }
                PackageLocation::Url(_) => path.for_namespace.clone(),
            }
        } else {
            format!("./{serde}")
        }
    }

    fn ts_namespace_import_path(&self, namespace: &str) -> String {
        if let Some(path) = self.config().external_packages.get(namespace) {
            match &path.location {
                PackageLocation::Path(_) => {
                    let name = &path.for_namespace;
                    if let Some(mod_name) = &path.module_name {
                        format!("{name}/{mod_name}")
                    } else {
                        name.clone()
                    }
                }
                PackageLocation::Url(_) => path.for_namespace.clone(),
            }
        } else {
            format!("../{namespace}")
        }
    }
}

impl Emitter<TypeScript> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &TypeScript) -> Result<()> {
        let CodeGeneratorConfig {
            encoding,
            features,
            referenced_namespaces,
            used_format_types,
            ..
        } = self.config();

        if self.config().has_encoding() {
            let import_path = self.ts_serde_import_path(lang.target);
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

        if !encoding.is_none() {
            for feature in features {
                match feature {
                    Feature::ListOfT => {
                        writeln!(w)?;
                        w.write_all(FEATURE_LIST_OF_T)?;
                    }
                    Feature::OptionOfT => {
                        writeln!(w)?;
                        w.write_all(FEATURE_OPTION_OF_T)?;
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
                output_struct_or_variant(w, None, None, name, &[], doc, lang)
            }
            ContainerFormat::NewTypeStruct(format, doc) => {
                let fields = vec![Named::new(format.as_ref(), "value".to_string())];
                output_struct_or_variant(w, None, None, name, &fields, doc, lang)
            }
            ContainerFormat::TupleStruct(formats, doc) => {
                let fields: Vec<_> = formats
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Named::new(f, format!("field{i}")))
                    .collect();
                output_struct_or_variant(w, None, None, name, &fields, doc, lang)
            }
            ContainerFormat::Struct(fields, doc) => {
                output_struct_or_variant(w, None, None, name, fields, doc, lang)
            }
            ContainerFormat::Enum(variants, doc) => {
                output_enum_container(w, name, variants, doc, lang)
            }
        }
    }
}

impl Emitter<TypeScript> for Format {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &TypeScript) -> Result<()> {
        match self {
            Format::TypeName(type_) => {
                write!(
                    w,
                    "{}",
                    type_.format(ToUpperCamelCase::to_upper_camel_case, ".")
                )
            }
            Format::Unit => write!(w, "unit"),
            Format::Bool => write!(w, "bool"),
            Format::I8 => write!(w, "int8"),
            Format::I16 => write!(w, "int16"),
            Format::I32 => write!(w, "int32"),
            Format::I64 => write!(w, "int64"),
            Format::I128 => write!(w, "int128"),
            Format::U8 => write!(w, "uint8"),
            Format::U16 => write!(w, "uint16"),
            Format::U32 => write!(w, "uint32"),
            Format::U64 => write!(w, "uint64"),
            Format::U128 => write!(w, "uint128"),
            Format::F32 => write!(w, "float32"),
            Format::F64 => write!(w, "float64"),
            Format::Char => write!(w, "char"),
            Format::Str => write!(w, "str"),
            Format::Bytes => write!(w, "bytes"),

            Format::Option(format) => {
                write!(w, "Optional<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Format::Seq(format) | Format::Set(format) => {
                write!(w, "Seq<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Format::Map { key, value } => {
                write!(w, "Map<")?;
                key.write(w, lang)?;
                write!(w, ",")?;
                value.write(w, lang)?;
                write!(w, ">")
            }
            Format::Tuple(formats) => {
                write!(w, "Tuple<[")?;
                for (i, f) in formats.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    f.write(w, lang)?;
                }
                write!(w, "]>")
            }
            Format::TupleArray { content, .. } => {
                write!(w, "ListTuple<[")?;
                content.write(w, lang)?;
                write!(w, "]>")
            }
            Format::Variable(_) => panic!("unexpected value"),
        }
    }
}

impl Emitter<TypeScript> for Named<Format> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &TypeScript) -> Result<()> {
        write!(w, "public {}: ", &self.name)?;
        self.value.write(w, lang)
    }
}

/// Render a type expression to a string.
fn quote_type(format: &Format) -> String {
    let mut buf = Vec::new();
    let mut w = IndentedWriter::new(&mut buf, IndentConfig::Space(0));
    format
        .write(&mut w, &TypeScript::new(Encoding::None))
        .expect("writing to Vec should not fail");
    String::from_utf8(buf).expect("type expression should be valid UTF-8")
}

/// Write a serialize statement for the given format, using nested closures
/// for container types (matching the Kotlin/Swift pattern).
fn write_serialize<W: IndentWrite>(w: &mut W, value_expr: &str, format: &Format) -> Result<()> {
    match format {
        Format::TypeName(_) => writeln!(w, "{value_expr}.serialize(serializer);"),
        Format::Unit => writeln!(w, "serializer.serializeUnit({value_expr});"),
        Format::Bool => writeln!(w, "serializer.serializeBool({value_expr});"),
        Format::I8 => writeln!(w, "serializer.serializeI8({value_expr});"),
        Format::I16 => writeln!(w, "serializer.serializeI16({value_expr});"),
        Format::I32 => writeln!(w, "serializer.serializeI32({value_expr});"),
        Format::I64 => writeln!(w, "serializer.serializeI64({value_expr});"),
        Format::I128 => writeln!(w, "serializer.serializeI128({value_expr});"),
        Format::U8 => writeln!(w, "serializer.serializeU8({value_expr});"),
        Format::U16 => writeln!(w, "serializer.serializeU16({value_expr});"),
        Format::U32 => writeln!(w, "serializer.serializeU32({value_expr});"),
        Format::U64 => writeln!(w, "serializer.serializeU64({value_expr});"),
        Format::U128 => writeln!(w, "serializer.serializeU128({value_expr});"),
        Format::F32 => writeln!(w, "serializer.serializeF32({value_expr});"),
        Format::F64 => writeln!(w, "serializer.serializeF64({value_expr});"),
        Format::Char => writeln!(w, "serializer.serializeChar({value_expr});"),
        Format::Str => writeln!(w, "serializer.serializeStr({value_expr});"),
        Format::Bytes => writeln!(w, "serializer.serializeBytes({value_expr});"),

        Format::Option(inner) => {
            write!(
                w,
                "serializeOption({value_expr}, serializer, (value, serializer) => "
            )?;
            {
                let mut w = w.block(Newlines::OPEN)?;
                write_serialize(&mut w, "value", inner)?;
            }
            writeln!(w, ");")
        }

        Format::Seq(inner) => {
            write!(
                w,
                "serializeArray({value_expr}, serializer, (item, serializer) => "
            )?;
            {
                let mut w = w.block(Newlines::OPEN)?;
                write_serialize(&mut w, "item", inner)?;
            }
            writeln!(w, ");")
        }

        Format::Set(inner) => {
            write!(
                w,
                "serializeSet({value_expr}, serializer, (item, serializer) => "
            )?;
            {
                let mut w = w.block(Newlines::OPEN)?;
                write_serialize(&mut w, "item", inner)?;
            }
            writeln!(w, ");")
        }

        Format::Map { key, value } => {
            write!(
                w,
                "serializeMap({value_expr}, serializer, (key, value, serializer) => "
            )?;
            {
                let mut w = w.block(Newlines::OPEN)?;
                write_serialize(&mut w, "key", key)?;
                write_serialize(&mut w, "value", value)?;
            }
            writeln!(w, ");")
        }

        Format::Tuple(formats) => {
            for (i, fmt) in formats.iter().enumerate() {
                write_serialize(w, &format!("{value_expr}[{i}]"), fmt)?;
            }

            Ok(())
        }

        Format::TupleArray { content, .. } => {
            write!(
                w,
                "serializeTupleArray({value_expr}, serializer, (item, serializer) => "
            )?;
            {
                let mut w = w.block(Newlines::OPEN)?;
                write_serialize(&mut w, "item[0]", content)?;
            }
            writeln!(w, ");")
        }

        Format::Variable(_) => panic!("unexpected value"),
    }
}

/// Returns a simple deserialize expression string for primitive/named types.
fn deserialize_primitive_expr(format: &Format) -> String {
    match format {
        Format::TypeName(qualified_name) => format!(
            "{}.deserialize(deserializer)",
            qualified_name.format(ToUpperCamelCase::to_upper_camel_case, ".")
        ),
        Format::Unit => "deserializer.deserializeUnit()".to_string(),
        Format::Bool => "deserializer.deserializeBool()".to_string(),
        Format::I8 => "deserializer.deserializeI8()".to_string(),
        Format::I16 => "deserializer.deserializeI16()".to_string(),
        Format::I32 => "deserializer.deserializeI32()".to_string(),
        Format::I64 => "deserializer.deserializeI64()".to_string(),
        Format::I128 => "deserializer.deserializeI128()".to_string(),
        Format::U8 => "deserializer.deserializeU8()".to_string(),
        Format::U16 => "deserializer.deserializeU16()".to_string(),
        Format::U32 => "deserializer.deserializeU32()".to_string(),
        Format::U64 => "deserializer.deserializeU64()".to_string(),
        Format::U128 => "deserializer.deserializeU128()".to_string(),
        Format::F32 => "deserializer.deserializeF32()".to_string(),
        Format::F64 => "deserializer.deserializeF64()".to_string(),
        Format::Char => "deserializer.deserializeChar()".to_string(),
        Format::Str => "deserializer.deserializeStr()".to_string(),
        Format::Bytes => "deserializer.deserializeBytes()".to_string(),
        _ => panic!("deserialize_primitive_expr called with non-primitive format"),
    }
}

/// Returns true if the format is a primitive type or a named type (not a container).
fn is_primitive_or_named(format: &Format) -> bool {
    matches!(
        format,
        Format::TypeName(_)
            | Format::Unit
            | Format::Bool
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
    )
}

/// Write a deserialize statement for the given format, using nested closures
/// for container types (matching the Kotlin/Swift pattern).
///
/// When `field_name` is Some, emits `const <name> = <expr>;`.
/// When `field_name` is None, emits `return <expr>;`.
#[allow(clippy::too_many_lines)]
fn write_deserialize<W: IndentWrite>(
    w: &mut W,
    field_name: Option<&str>,
    format: &Format,
) -> Result<()> {
    match format {
        // Primitive and named types - single expression
        f if is_primitive_or_named(f) => {
            let expr = deserialize_primitive_expr(f);
            if let Some(name) = field_name {
                writeln!(w, "const {name} = {expr};")
            } else {
                writeln!(w, "return {expr};")
            }
        }

        // Container types - nested closures
        Format::Option(inner) => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeOption(deserializer, (deserializer) => "
                )?;
            } else {
                write!(
                    w,
                    "return deserializeOption(deserializer, (deserializer) => "
                )?;
            }
            {
                let mut w = w.block(Newlines::OPEN)?;
                write_deserialize(&mut w, None, inner)?;
            }
            writeln!(w, ");")
        }

        Format::Seq(inner) => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeArray(deserializer, (deserializer) => "
                )?;
            } else {
                write!(
                    w,
                    "return deserializeArray(deserializer, (deserializer) => "
                )?;
            }
            {
                let mut w = w.block(Newlines::OPEN)?;
                write_deserialize(&mut w, None, inner)?;
            }
            writeln!(w, ");")
        }

        Format::Set(inner) => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeSet(deserializer, (deserializer) => "
                )?;
            } else {
                write!(w, "return deserializeSet(deserializer, (deserializer) => ")?;
            }
            {
                let mut w = w.block(Newlines::OPEN)?;
                write_deserialize(&mut w, None, inner)?;
            }
            writeln!(w, ");")
        }

        Format::Map { key, value } => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeMap(deserializer, (deserializer) => "
                )?;
            } else {
                write!(w, "return deserializeMap(deserializer, (deserializer) => ")?;
            }
            {
                let mut w = w.block(Newlines::OPEN)?;
                // Key deserialization
                if is_primitive_or_named(key) {
                    let key_expr = deserialize_primitive_expr(key);
                    writeln!(w, "const key = {key_expr};")?;
                } else {
                    write_deserialize(&mut w, Some("key"), key)?;
                }
                // Value deserialization
                if is_primitive_or_named(value) {
                    let value_expr = deserialize_primitive_expr(value);
                    writeln!(w, "const value = {value_expr};")?;
                } else {
                    write_deserialize(&mut w, Some("value"), value)?;
                }
                writeln!(w, "return [key, value];")?;
            }
            writeln!(w, ");")
        }

        Format::Tuple(formats) => {
            // Deserialize each element into a temp variable, then build the tuple array
            for (i, f) in formats.iter().enumerate() {
                write_deserialize(w, Some(&format!("field{i}")), f)?;
            }
            let fields = (0..formats.len())
                .map(|i| format!("field{i}"))
                .collect::<Vec<_>>()
                .join(", ");
            let type_str = formats
                .iter()
                .map(quote_type)
                .collect::<Vec<_>>()
                .join(", ");
            if let Some(name) = field_name {
                writeln!(w, "const {name} = [{fields}] as [{type_str}];")
            } else {
                writeln!(w, "return [{fields}] as [{type_str}];")
            }
        }

        Format::TupleArray { content, size } => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeTupleArray(deserializer, {size}, (deserializer) => "
                )?;
            } else {
                write!(
                    w,
                    "return deserializeTupleArray(deserializer, {size}, (deserializer) => "
                )?;
            }
            {
                let mut w = w.block(Newlines::OPEN)?;
                write_deserialize(&mut w, Some("item"), content)?;
                writeln!(w, "return [item];")?;
            }
            writeln!(w, ");")
        }

        Format::Variable(_) => panic!("unexpected value"),
        _ => unreachable!(),
    }
}

fn output_struct_or_variant<W: IndentWrite>(
    w: &mut W,
    variant_base: Option<&str>,
    variant_index: Option<u32>,
    name: &str,
    fields: &[Named<Format>],
    doc: &Doc,
    lang: &TypeScript,
) -> Result<()> {
    let mut variant_base_name = String::new();

    writeln!(w)?;
    doc.write(w, lang)?;
    if let Some(base) = variant_base {
        write!(w, "export class {base}Variant{name} extends {base} ")?;
        variant_base_name = format!("{base}Variant");
    } else {
        write!(w, "export class {name} ")?;
    }
    let mut w = w.block(Newlines::BOTH)?;
    let args: Vec<String> = fields
        .iter()
        .map(|f| {
            let type_str = quote_type(&f.value);
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
    if lang.encoding.is_bincode() || lang.encoding.is_json() {
        writeln!(w)?;
        write!(w, "public serialize(serializer: Serializer): void ")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            if let Some(index) = variant_index {
                writeln!(w, "serializer.serializeVariantIndex({index});")?;
            }
            for field in fields {
                let value_expr = format!("this.{}", &field.name);
                write_serialize(&mut w, &value_expr, &field.value)?;
            }
        }
        writeln!(w)?;
        if variant_index.is_none() {
            write!(w, "static deserialize(deserializer: Deserializer): {name} ")?;
        } else {
            write!(
                w,
                "static load(deserializer: Deserializer): {variant_base_name}{name} ",
            )?;
        }
        {
            let mut w = w.block(Newlines::BOTH)?;
            for field in fields {
                write_deserialize(&mut w, Some(&field.name), &field.value)?;
            }
            writeln!(
                w,
                r"return new {0}{1}({2});",
                variant_base_name,
                name,
                fields
                    .iter()
                    .map(|f| f.name.clone())
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
        }
    }

    Ok(())
}

fn output_variant<W: IndentWrite>(
    w: &mut W,
    base: &str,
    index: u32,
    name: &str,
    variant: &VariantFormat,
    doc: &Doc,
    lang: &TypeScript,
) -> Result<()> {
    let fields = match variant {
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
    output_struct_or_variant(w, Some(base), Some(index), name, &fields, doc, lang)
}

fn output_enum_container<W: IndentWrite>(
    w: &mut W,
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
        if lang.encoding.is_bincode() || lang.encoding.is_json() {
            writeln!(w, "abstract serialize(serializer: Serializer): void;\n")?;
            write!(w, "static deserialize(deserializer: Deserializer): {name} ")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                writeln!(w, "const index = deserializer.deserializeVariantIndex();")?;
                write!(w, "switch (index) ")?;
                {
                    let mut w = w.block(Newlines::BOTH)?;
                    for (index, variant) in variants {
                        writeln!(
                            w,
                            "case {}: return {}Variant{}.load(deserializer);",
                            index, name, variant.name,
                        )?;
                    }
                    writeln!(
                        w,
                        "default: throw new Error(\"Unknown variant index for {name}: \" + index);",
                    )?;
                }
            }
        }
    }
    for (index, variant) in variants {
        output_variant(
            w,
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
