use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Result, Write},
};

use heck::ToUpperCamelCase;

use crate::{
    Registry,
    generation::{
        Container, Emitter, Encoding, PackageLocation, common, indent::IndentWrite, module::Module,
    },
    reflection::format::{
        ContainerFormat, Doc, Format, FormatHolder as _, Named, Namespace, VariantFormat,
    },
};

use super::InstallTarget;

/// Language tag for TypeScript code generation.
#[derive(Debug, Clone, Copy)]
pub struct TypeScript {
    pub encoding: Encoding,
    pub target: InstallTarget,
}

impl TypeScript {
    pub fn new(encoding: Encoding, target: InstallTarget) -> Self {
        Self { encoding, target }
    }
}

impl Module {
    fn ts_serde_import_path(&self, target: InstallTarget) -> String {
        let serde = target.serde_import_path();
        if let Some(path) = self.config().external_packages.get("serde") {
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
    fn write<W: IndentWrite>(&self, w: &mut W, lang: TypeScript) -> Result<()> {
        if self.config().has_encoding() {
            let import_path = self.ts_serde_import_path(lang.target);
            writeln!(
                w,
                r#"import {{ Serializer, Deserializer }} from "{import_path}";"#
            )?;
        }

        Ok(())
    }
}

/// Collect namespaces actually referenced in the registry.
pub(crate) fn collect_used_namespaces(registry: &Registry) -> BTreeSet<String> {
    let mut namespaces = BTreeSet::new();
    for format in registry.values() {
        format
            .visit(&mut |f| {
                if let Format::TypeName(qualified_name) = f {
                    if let Namespace::Named(ns) = &qualified_name.namespace {
                        namespaces.insert(ns.clone());
                    }
                }
                Ok(())
            })
            .unwrap();
    }
    namespaces
}

/// Write namespace import statements for the given namespaces.
pub(crate) fn write_namespace_imports<W: Write>(
    w: &mut W,
    module: &Module,
    namespaces: &BTreeSet<String>,
) -> Result<()> {
    let mut import_paths: BTreeMap<String, String> = BTreeMap::new();

    for namespace in namespaces {
        let import_path = module.ts_namespace_import_path(namespace);
        import_paths.insert(namespace.to_upper_camel_case(), import_path);
    }

    for (namespace, path) in import_paths {
        writeln!(w, r#"import * as {namespace} from "{path}";"#)?;
    }

    Ok(())
}

impl Emitter<TypeScript> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: TypeScript) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }
        Ok(())
    }
}

impl Emitter<TypeScript> for Container<'_> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: TypeScript) -> Result<()> {
        let Container {
            name: qualified_name,
            format,
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
    fn write<W: IndentWrite>(&self, w: &mut W, lang: TypeScript) -> Result<()> {
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
    fn write<W: IndentWrite>(&self, w: &mut W, lang: TypeScript) -> Result<()> {
        write!(w, "public {}: ", &self.name)?;
        self.value.write(w, lang)
    }
}

/// Render a type expression to a string.
fn quote_type(format: &Format) -> String {
    let mut buf = Vec::new();
    let mut w = crate::generation::indent::IndentedWriter::new(
        &mut buf,
        crate::generation::indent::IndentConfig::Space(0),
    );
    format
        .write(&mut w, TypeScript::new(Encoding::None, InstallTarget::Node))
        .expect("writing to Vec should not fail");
    String::from_utf8(buf).expect("type expression should be valid UTF-8")
}

fn quote_serialize_value(value: &str, format: &Format, use_this: bool) -> String {
    let this_str = if use_this { "this." } else { "" };

    match format {
        Format::TypeName(_) => {
            format!("{this_str}{value}.serialize(serializer);")
        }
        Format::Unit => format!("serializer.serializeUnit({this_str}{value});"),
        Format::Bool => format!("serializer.serializeBool({this_str}{value});"),
        Format::I8 => format!("serializer.serializeI8({this_str}{value});"),
        Format::I16 => format!("serializer.serializeI16({this_str}{value});"),
        Format::I32 => format!("serializer.serializeI32({this_str}{value});"),
        Format::I64 => format!("serializer.serializeI64({this_str}{value});"),
        Format::I128 => format!("serializer.serializeI128({this_str}{value});"),
        Format::U8 => format!("serializer.serializeU8({this_str}{value});"),
        Format::U16 => format!("serializer.serializeU16({this_str}{value});"),
        Format::U32 => format!("serializer.serializeU32({this_str}{value});"),
        Format::U64 => format!("serializer.serializeU64({this_str}{value});"),
        Format::U128 => format!("serializer.serializeU128({this_str}{value});"),
        Format::F32 => format!("serializer.serializeF32({this_str}{value});"),
        Format::F64 => format!("serializer.serializeF64({this_str}{value});"),
        Format::Char => format!("serializer.serializeChar({this_str}{value});"),
        Format::Str => format!("serializer.serializeStr({this_str}{value});"),
        Format::Bytes => format!("serializer.serializeBytes({this_str}{value});"),
        _ => format!(
            "Helpers.serialize{}({}{}, serializer);",
            common::mangle_type(format).to_upper_camel_case(),
            this_str,
            value
        ),
    }
}

fn quote_deserialize(format: &Format) -> String {
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
        _ => format!(
            "Helpers.deserialize{}(deserializer)",
            common::mangle_type(format).to_upper_camel_case(),
        ),
    }
}

fn output_struct_or_variant<W: IndentWrite>(
    w: &mut W,
    variant_base: Option<&str>,
    variant_index: Option<u32>,
    name: &str,
    fields: &[Named<Format>],
    doc: &Doc,
    lang: TypeScript,
) -> Result<()> {
    let mut variant_base_name = String::new();

    doc.write(w, lang)?;

    if let Some(base) = variant_base {
        writeln!(w, "export class {base}Variant{name} extends {base} {{")?;
        variant_base_name = format!("{base}Variant");
    } else {
        writeln!(w, "export class {name} {{")?;
    }
    w.indent();
    if !fields.is_empty() {
        writeln!(w)?;
    }
    let args: Vec<String> = fields
        .iter()
        .map(|f| {
            let type_str = quote_type(&f.value);
            format!("public {}: {}", &f.name, type_str)
        })
        .collect();
    let args = args.join(", ");
    writeln!(w, "constructor ({args}) {{")?;
    w.indent();
    if variant_base.is_some() {
        writeln!(w, "super();")?;
    }
    w.unindent();
    writeln!(w, "}}\n")?;
    if lang.encoding.is_bincode() || lang.encoding.is_json() {
        writeln!(w, "public serialize(serializer: Serializer): void {{")?;
        w.indent();
        if let Some(index) = variant_index {
            writeln!(w, "serializer.serializeVariantIndex({index});")?;
        }
        for field in fields {
            writeln!(
                w,
                "{}",
                quote_serialize_value(&field.name, &field.value, true)
            )?;
        }
        w.unindent();
        writeln!(w, "}}\n")?;
    }
    if lang.encoding.is_bincode() || lang.encoding.is_json() {
        if variant_index.is_none() {
            writeln!(
                w,
                "static deserialize(deserializer: Deserializer): {name} {{",
            )?;
        } else {
            writeln!(
                w,
                "static load(deserializer: Deserializer): {variant_base_name}{name} {{",
            )?;
        }
        w.indent();
        for field in fields {
            writeln!(
                w,
                "const {} = {};",
                field.name,
                quote_deserialize(&field.value)
            )?;
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
        w.unindent();
        writeln!(w, "}}\n")?;
    }
    w.unindent();
    writeln!(w, "}}")
}

fn output_variant<W: IndentWrite>(
    w: &mut W,
    base: &str,
    index: u32,
    name: &str,
    variant: &VariantFormat,
    doc: &Doc,
    lang: TypeScript,
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
    lang: TypeScript,
) -> Result<()> {
    doc.write(w, lang)?;
    writeln!(w, "export abstract class {name} {{")?;
    w.indent();
    if lang.encoding.is_bincode() || lang.encoding.is_json() {
        writeln!(w, "abstract serialize(serializer: Serializer): void;\n")?;
        write!(
            w,
            "static deserialize(deserializer: Deserializer): {name} {{"
        )?;
        w.indent();
        writeln!(
            w,
            r"
const index = deserializer.deserializeVariantIndex();
switch (index) {{",
        )?;
        w.indent();
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
        w.unindent();
        writeln!(w, "}}")?;
        w.unindent();
        writeln!(w, "}}")?;
    }
    w.unindent();
    writeln!(w, "}}\n")?;
    for (index, variant) in variants {
        writeln!(w)?;
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

fn needs_helper(format: &Format) -> bool {
    matches!(
        format,
        Format::Option(_)
            | Format::Seq(_)
            | Format::Set(_)
            | Format::Map { .. }
            | Format::Tuple(_)
            | Format::TupleArray { .. }
    )
}

pub(crate) fn output_helpers<W: IndentWrite>(
    w: &mut W,
    registry: &Registry,
    _lang: TypeScript,
) -> Result<()> {
    let mut subtypes = BTreeMap::new();
    for format in registry.values() {
        format
            .visit(&mut |f| {
                if needs_helper(f) {
                    subtypes.insert(common::mangle_type(f), f.clone());
                }
                Ok(())
            })
            .unwrap();
    }

    writeln!(w, "export class Helpers {{")?;
    w.indent();
    for (mangled_name, subtype) in &subtypes {
        output_serialization_helper(w, mangled_name, subtype)?;
        output_deserialization_helper(w, mangled_name, subtype)?;
    }
    w.unindent();
    writeln!(w, "}}")?;
    writeln!(w)
}

fn output_serialization_helper<W: IndentWrite>(
    w: &mut W,
    name: &str,
    format0: &Format,
) -> Result<()> {
    let type_ = quote_type(format0);
    let name = name.to_upper_camel_case();
    write!(
        w,
        "static serialize{name}(value: {type_}, serializer: Serializer): void {{",
    )?;
    w.indent();
    match format0 {
        Format::Option(format) => {
            write!(
                w,
                r"
if (value) {{
    serializer.serializeOptionTag(true);
    {}
}} else {{
    serializer.serializeOptionTag(false);
}}
",
                quote_serialize_value("value", format, false)
            )?;
        }

        Format::Seq(format) | Format::Set(format) => {
            let type_ = quote_type(format);
            let item = quote_serialize_value("item", format, false);
            write!(
                w,
                r"
serializer.serializeLen(value.length);
value.forEach((item: {type_}) => {{
    {item}
}});
"
            )?;
        }

        Format::Map { key, value } => {
            write!(
                w,
                r"
serializer.serializeLen(value.size);
const offsets: number[] = [];
for (const [k, v] of value.entries()) {{
  offsets.push(serializer.getBufferOffset());
  {}
  {}
}}
serializer.sortMapEntries(offsets);
",
                quote_serialize_value("k", key, false),
                quote_serialize_value("v", value, false)
            )?;
        }

        Format::Tuple(format_list) => {
            writeln!(w)?;
            for (index, format) in format_list.iter().enumerate() {
                let expr = format!("value[{index}]");
                writeln!(w, "{}", quote_serialize_value(&expr, format, false))?;
            }
        }

        Format::TupleArray { content, .. } => {
            write!(
                w,
                r"
value.forEach((item) =>{{
    {}
}});
",
                quote_serialize_value("item[0]", content, false)
            )?;
        }

        _ => panic!("unexpected case"),
    }
    w.unindent();
    writeln!(w, "}}\n")
}

#[allow(clippy::too_many_lines)]
fn output_deserialization_helper<W: IndentWrite>(
    w: &mut W,
    name: &str,
    format0: &Format,
) -> Result<()> {
    let name = name.to_upper_camel_case();
    let type_ = quote_type(format0);
    write!(
        w,
        "static deserialize{name}(deserializer: Deserializer): {type_} {{",
    )?;
    w.indent();
    match format0 {
        Format::Option(format) => {
            write!(
                w,
                r"
const tag = deserializer.deserializeOptionTag();
if (!tag) {{
    return null;
}} else {{
    return {};
}}
",
                quote_deserialize(format),
            )?;
        }

        Format::Seq(format) | Format::Set(format) => {
            let format0_str = quote_type(format0);
            write!(
                w,
                r"
const length = deserializer.deserializeLen();
const list: {format0_str} = [];
for (let i = 0; i < length; i++) {{
    list.push({});
}}
return list;
",
                quote_deserialize(format)
            )?;
        }

        Format::Map { key, value } => {
            let key_type = quote_type(key);
            let value_type = quote_type(value);
            write!(
                w,
                r"
const length = deserializer.deserializeLen();
const obj = new Map<{key_type}, {value_type}>();
let previousKeyStart = 0;
let previousKeyEnd = 0;
for (let i = 0; i < length; i++) {{
    const keyStart = deserializer.getBufferOffset();
    const key = {0};
    const keyEnd = deserializer.getBufferOffset();
    if (i > 0) {{
        deserializer.checkThatKeySlicesAreIncreasing(
            [previousKeyStart, previousKeyEnd],
            [keyStart, keyEnd]);
    }}
    previousKeyStart = keyStart;
    previousKeyEnd = keyEnd;
    const value = {1};
    obj.set(key, value);
}}
return obj;
",
                quote_deserialize(key),
                quote_deserialize(value),
            )?;
        }

        Format::Tuple(format_list) => {
            write!(
                w,
                r"
return [{}
];
",
                format_list
                    .iter()
                    .map(|f| format!("\n    {}", quote_deserialize(f)))
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
        }

        Format::TupleArray { content, size } => {
            let format0_str = quote_type(format0);
            let content = quote_deserialize(content);
            write!(
                w,
                r"
const list: {format0_str} = [];
for (let i = 0; i < {size}; i++) {{
    list.push([{content}]);
}}
return list;
",
            )?;
        }

        _ => panic!("unexpected case"),
    }
    w.unindent();
    writeln!(w, "}}\n")
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

pub(crate) fn collect_type_alias_keys(registry: &Registry) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for format in registry.values() {
        format
            .visit(&mut |f| {
                let key = match f {
                    Format::Unit => "unit",
                    Format::Bool => "bool",
                    Format::I8 => "int8",
                    Format::I16 => "int16",
                    Format::I32 => "int32",
                    Format::I64 => "int64",
                    Format::I128 => "int128",
                    Format::U8 => "uint8",
                    Format::U16 => "uint16",
                    Format::U32 => "uint32",
                    Format::U64 => "uint64",
                    Format::U128 => "uint128",
                    Format::F32 => "float32",
                    Format::F64 => "float64",
                    Format::Char => "char",
                    Format::Str => "str",
                    Format::Bytes => "bytes",
                    Format::Option(_) => "option",
                    Format::Seq(_) | Format::Set(_) => "seq",
                    Format::Map { .. } => "map",
                    Format::Tuple(_) => "tuple",
                    Format::TupleArray { .. } => "list_tuple",
                    Format::TypeName(_) | Format::Variable(_) => "",
                };
                if !key.is_empty() {
                    keys.insert(key.to_string());
                }
                Ok(())
            })
            .unwrap();
    }
    keys
}

pub(crate) fn write_type_aliases<W: Write>(
    w: &mut W,
    type_alias_keys: &BTreeSet<String>,
) -> Result<()> {
    let map = BTreeMap::from(TYPE_ALIASES);
    let aliases: String = type_alias_keys
        .iter()
        .filter_map(|k| map.get(k.as_str()).map(|s| (*s).to_string()))
        .collect::<Vec<_>>()
        .join("\n");
    writeln!(w, "{aliases}")
}

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
