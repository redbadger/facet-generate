use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

use heck::ToUpperCamelCase;

use crate::{
    Registry,
    generation::{
        Encoding, PackageLocation, common, indent::IndentWrite, typescript::CodeGenerator,
    },
    reflection::format::{
        ContainerFormat, Doc, Format, FormatHolder as _, Named, Namespace, VariantFormat,
    },
};

use super::InstallTarget;

/// Shared state for the code generation of a TypeScript source file.
pub(crate) struct TypeScriptEmitter<'a, T> {
    /// Generator.
    pub(crate) generator: &'a CodeGenerator<'a>,
    namespaces_used: BTreeSet<Namespace>,
    types_used: BTreeSet<String>,
    _data: PhantomData<T>,
}

impl<'a, T> TypeScriptEmitter<'a, T>
where
    T: IndentWrite,
{
    pub fn new(generator: &'a CodeGenerator<'a>) -> Self {
        Self {
            generator,
            namespaces_used: BTreeSet::new(),
            types_used: BTreeSet::new(),
            _data: PhantomData,
        }
    }

    pub fn output_preamble(&mut self, out: &mut T) -> std::io::Result<()> {
        if self.generator.config.has_encoding() {
            let (serde, bcs) = match self.generator.target {
                InstallTarget::Node => ("serde", "bcs"),
                InstallTarget::Deno => ("serde/mod.ts", "bcs/mod.ts"),
            };
            let import_path =
                if let Some(path) = self.generator.config.external_packages.get("serde") {
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
                };
            writeln!(
                out,
                r#"import {{ Serializer, Deserializer }} from "{import_path}";"#
            )?;
            if let Encoding::Bcs = self.generator.config.encoding {
                writeln!(
                    out,
                    r#"import {{ BcsSerializer, BcsDeserializer }} from "./{bcs}";"#
                )?;
            }
        }

        let mut import_paths: BTreeMap<String, String> = BTreeMap::new();

        for namespace in self.namespaces_used.iter().filter_map(|ns| match ns {
            Namespace::Root => None,
            Namespace::Named(name) => Some(name),
        }) {
            let import_path =
                if let Some(path) = self.generator.config.external_packages.get(namespace) {
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
                };

            import_paths.insert(namespace.to_upper_camel_case(), import_path);
        }

        for (namespace, path) in import_paths {
            writeln!(out, r#"import * as {namespace} from "{path}";"#,)?;
        }

        let aliases = format_type_aliases(&self.types_used);
        writeln!(out, "{aliases}")?;

        Ok(())
    }

    fn output_comment(&mut self, out: &mut T, name: &str) -> std::io::Result<()> {
        let path = vec![name.to_string()];
        if let Some(doc) = self.generator.config.comments.get(&path) {
            let text = textwrap::indent(doc, " * ").replace("\n\n", "\n *\n");
            writeln!(out, "/**\n{text} */")?;
        }
        Ok(())
    }

    fn quote_typename(&self, name: &str) -> String {
        self.generator
            .external_qualified_names
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string())
    }

    fn quote_type(&mut self, format: &Format) -> String {
        let (quoted, type_) = match format {
            Format::TypeName(type_) => {
                if !self.namespaces_used.contains(&type_.namespace) {
                    self.namespaces_used.insert(type_.namespace.clone());
                }
                (self.quote_typename(&type_.name), "")
            }
            Format::Unit => ("unit".into(), "unit"),
            Format::Bool => ("bool".into(), "bool"),
            Format::I8 => ("int8".into(), "int8"),
            Format::I16 => ("int16".into(), "int16"),
            Format::I32 => ("int32".into(), "int32"),
            Format::I64 => ("int64".into(), "int64"),
            Format::I128 => ("int128".into(), "int128"),
            Format::U8 => ("uint8".into(), "uint8"),
            Format::U16 => ("uint16".into(), "uint16"),
            Format::U32 => ("uint32".into(), "uint32"),
            Format::U64 => ("uint64".into(), "uint64"),
            Format::U128 => ("uint128".into(), "uint128"),
            Format::F32 => ("float32".into(), "float32"),
            Format::F64 => ("float64".into(), "float64"),
            Format::Char => ("char".into(), "char"),
            Format::Str => ("str".into(), "str"),
            Format::Bytes => ("bytes".into(), "bytes"),

            Format::Option(format) => (format!("Optional<{}>", self.quote_type(format)), "option"),
            Format::Seq(format) | Format::Set(format) => {
                (format!("Seq<{}>", self.quote_type(format)), "seq")
            }
            Format::Map { key, value } => (
                format!("Map<{},{}>", self.quote_type(key), self.quote_type(value)),
                "map",
            ),
            Format::Tuple(formats) => (
                format!("Tuple<[{}]>", self.quote_types(formats, ", ")),
                "tuple",
            ),
            Format::TupleArray {
                content,
                size: _size,
            } => (
                format!("ListTuple<[{}]>", self.quote_type(content)),
                "list_tuple",
            ),
            Format::Variable(_) => panic!("unexpected value"),
        };
        self.types_used.insert(type_.to_string());

        quoted
    }

    fn quote_types(&mut self, formats: &[Format], sep: &str) -> String {
        formats
            .iter()
            .map(|f| self.quote_type(f))
            .collect::<Vec<_>>()
            .join(sep)
    }

    pub fn output_helpers(&mut self, out: &mut T, registry: &Registry) -> std::io::Result<()> {
        let mut subtypes = BTreeMap::new();
        for format in registry.values() {
            format
                .visit(&mut |f| {
                    if Self::needs_helper(f) {
                        subtypes.insert(common::mangle_type(f), f.clone());
                    }
                    Ok(())
                })
                .unwrap();
        }

        writeln!(out, "export class Helpers {{")?;
        out.indent();
        for (mangled_name, subtype) in &subtypes {
            self.output_serialization_helper(out, mangled_name, subtype)?;
            self.output_deserialization_helper(out, mangled_name, subtype)?;
        }
        out.unindent();
        writeln!(out, "}}")?;
        writeln!(out)
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

    #[allow(clippy::unused_self)]
    fn quote_serialize_value(&self, value: &str, format: &Format, use_this: bool) -> String {
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

    fn quote_deserialize(&self, format: &Format) -> String {
        match format {
            Format::TypeName(qualified_name) => format!(
                "{}.deserialize(deserializer)",
                self.quote_typename(&qualified_name.name)
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

    fn output_serialization_helper(
        &mut self,
        out: &mut T,
        name: &str,
        format0: &Format,
    ) -> std::io::Result<()> {
        let type_ = self.quote_type(format0);
        let name = name.to_upper_camel_case();
        write!(
            out,
            "static serialize{name}(value: {type_}, serializer: Serializer): void {{",
        )?;
        out.indent();
        match format0 {
            Format::Option(format) => {
                write!(
                    out,
                    r"
if (value) {{
    serializer.serializeOptionTag(true);
    {}
}} else {{
    serializer.serializeOptionTag(false);
}}
",
                    self.quote_serialize_value("value", format, false)
                )?;
            }

            Format::Seq(format) | Format::Set(format) => {
                let type_ = self.quote_type(format);
                let item = self.quote_serialize_value("item", format, false);
                write!(
                    out,
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
                    out,
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
                    self.quote_serialize_value("k", key, false),
                    self.quote_serialize_value("v", value, false)
                )?;
            }

            Format::Tuple(format_list) => {
                writeln!(out)?;
                for (index, format) in format_list.iter().enumerate() {
                    let expr = format!("value[{index}]");
                    writeln!(out, "{}", self.quote_serialize_value(&expr, format, false))?;
                }
            }

            Format::TupleArray {
                content,
                size: _size,
            } => {
                write!(
                    out,
                    r"
value.forEach((item) =>{{
    {}
}});
",
                    self.quote_serialize_value("item[0]", content, false)
                )?;
            }

            _ => panic!("unexpected case"),
        }
        out.unindent();
        writeln!(out, "}}\n")
    }

    #[allow(clippy::too_many_lines)]
    fn output_deserialization_helper(
        &mut self,
        out: &mut T,
        name: &str,
        format0: &Format,
    ) -> std::io::Result<()> {
        let name = name.to_upper_camel_case();
        let type_ = self.quote_type(format0);
        write!(
            out,
            "static deserialize{name}(deserializer: Deserializer): {type_} {{",
        )?;
        out.indent();
        match format0 {
            Format::Option(format) => {
                write!(
                    out,
                    r"
const tag = deserializer.deserializeOptionTag();
if (!tag) {{
    return null;
}} else {{
    return {};
}}
",
                    self.quote_deserialize(format),
                )?;
            }

            Format::Seq(format) | Format::Set(format) => {
                let format0 = self.quote_type(format0);
                write!(
                    out,
                    r"
const length = deserializer.deserializeLen();
const list: {format0} = [];
for (let i = 0; i < length; i++) {{
    list.push({});
}}
return list;
",
                    self.quote_deserialize(format)
                )?;
            }

            Format::Map { key, value } => {
                let key_type = self.quote_type(key);
                let value_type = self.quote_type(value);
                write!(
                    out,
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
                    self.quote_deserialize(key),
                    self.quote_deserialize(value),
                )?;
            }

            Format::Tuple(format_list) => {
                write!(
                    out,
                    r"
return [{}
];
",
                    format_list
                        .iter()
                        .map(|f| format!("\n    {}", self.quote_deserialize(f)))
                        .collect::<Vec<_>>()
                        .join(",")
                )?;
            }

            Format::TupleArray { content, size } => {
                let format0 = self.quote_type(format0);
                let content = self.quote_deserialize(content);
                write!(
                    out,
                    r"
const list: {format0} = [];
for (let i = 0; i < {size}; i++) {{
    list.push([{content}]);
}}
return list;
",
                )?;
            }

            _ => panic!("unexpected case"),
        }
        out.unindent();
        writeln!(out, "}}\n")
    }

    fn output_variant(
        &mut self,
        out: &mut T,
        base: &str,
        index: u32,
        name: &str,
        variant: &VariantFormat,
    ) -> std::io::Result<()> {
        let fields = match variant {
            VariantFormat::Unit => Vec::new(),
            VariantFormat::NewType(format) => vec![Named {
                name: "value".to_string(),
                doc: Doc::new(),
                value: format.as_ref().clone(),
            }],
            VariantFormat::Tuple(formats) => formats
                .iter()
                .enumerate()
                .map(|(i, f)| Named {
                    name: format!("field{i}"),
                    doc: Doc::new(),
                    value: f.clone(),
                })
                .collect(),
            VariantFormat::Struct(fields) => fields.clone(),
            VariantFormat::Variable(_) => panic!("incorrect value"),
        };
        self.output_struct_or_variant_container(out, Some(base), Some(index), name, &fields)
    }

    fn output_variants(
        &mut self,
        out: &mut T,
        base: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> std::io::Result<()> {
        for (index, variant) in variants {
            self.output_variant(out, base, *index, &variant.name, &variant.value)?;
        }
        Ok(())
    }

    fn output_struct_or_variant_container(
        &mut self,
        out: &mut T,
        variant_base: Option<&str>,
        variant_index: Option<u32>,
        name: &str,
        fields: &[Named<Format>],
    ) -> std::io::Result<()> {
        let mut variant_base_name = String::new();

        // Beginning of class
        if let Some(base) = variant_base {
            writeln!(out)?;
            self.output_comment(out, name)?;
            writeln!(out, "export class {base}Variant{name} extends {base} {{")?;
            variant_base_name = format!("{base}Variant");
        } else {
            self.output_comment(out, name)?;
            writeln!(out, "export class {name} {{")?;
        }
        out.indent();
        if !fields.is_empty() {
            writeln!(out)?;
        }
        // Constructor.
        let args = fields
            .iter()
            .map(|f| format!("public {}: {}", &f.name, self.quote_type(&f.value)))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(out, "constructor ({args}) {{")?;
        out.indent();
        if let Some(_base) = variant_base {
            writeln!(out, "super();")?;
        }
        out.unindent();
        writeln!(out, "}}\n")?;
        // Serialize
        if self.generator.config.has_encoding() {
            writeln!(out, "public serialize(serializer: Serializer): void {{",)?;
            out.indent();
            if let Some(index) = variant_index {
                writeln!(out, "serializer.serializeVariantIndex({index});")?;
            }
            for field in fields {
                writeln!(
                    out,
                    "{}",
                    self.quote_serialize_value(&field.name, &field.value, true)
                )?;
            }
            out.unindent();
            writeln!(out, "}}\n")?;
        }
        // Deserialize (struct) or Load (variant)
        if self.generator.config.has_encoding() {
            if variant_index.is_none() {
                writeln!(
                    out,
                    "static deserialize(deserializer: Deserializer): {name} {{",
                )?;
            } else {
                writeln!(
                    out,
                    "static load(deserializer: Deserializer): {variant_base_name}{name} {{",
                )?;
            }
            out.indent();
            for field in fields {
                writeln!(
                    out,
                    "const {} = {};",
                    field.name,
                    self.quote_deserialize(&field.value)
                )?;
            }
            writeln!(
                out,
                r"return new {0}{1}({2});",
                variant_base_name,
                name,
                fields
                    .iter()
                    .map(|f| f.name.clone())
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
            out.unindent();
            writeln!(out, "}}\n")?;
        }
        out.unindent();
        writeln!(out, "}}")
    }

    fn output_enum_container(
        &mut self,
        out: &mut T,
        name: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> std::io::Result<()> {
        self.output_comment(out, name)?;
        writeln!(out, "export abstract class {name} {{")?;
        out.indent();
        if self.generator.config.has_encoding() {
            writeln!(out, "abstract serialize(serializer: Serializer): void;\n")?;
            write!(
                out,
                "static deserialize(deserializer: Deserializer): {name} {{"
            )?;
            out.indent();
            writeln!(
                out,
                r"
const index = deserializer.deserializeVariantIndex();
switch (index) {{",
            )?;
            out.indent();
            for (index, variant) in variants {
                writeln!(
                    out,
                    "case {}: return {}Variant{}.load(deserializer);",
                    index, name, variant.name,
                )?;
            }
            writeln!(
                out,
                "default: throw new Error(\"Unknown variant index for {name}: \" + index);",
            )?;
            out.unindent();
            writeln!(out, "}}")?;
            out.unindent();
            writeln!(out, "}}")?;
        }
        out.unindent();
        writeln!(out, "}}\n")?;
        self.output_variants(out, name, variants)?;
        Ok(())
    }

    pub fn output_container(
        &mut self,
        out: &mut T,
        name: &str,
        format: &ContainerFormat,
    ) -> std::io::Result<()> {
        let fields = match format {
            ContainerFormat::UnitStruct(_doc) => Vec::new(),
            ContainerFormat::NewTypeStruct(format, _doc) => vec![Named {
                name: "value".to_string(),
                doc: Doc::new(),
                value: format.as_ref().clone(),
            }],
            ContainerFormat::TupleStruct(formats, _doc) => formats
                .iter()
                .enumerate()
                .map(|(i, f)| Named {
                    name: format!("field{i}"),
                    doc: Doc::new(),
                    value: f.clone(),
                })
                .collect::<Vec<_>>(),
            ContainerFormat::Struct(fields, _doc) => fields.clone(),
            ContainerFormat::Enum(variants, _doc) => {
                self.output_enum_container(out, name, variants)?;
                return Ok(());
            }
        };
        self.output_struct_or_variant_container(out, None, None, name, &fields)
    }
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

fn format_type_aliases(input: &BTreeSet<String>) -> String {
    let map = BTreeMap::from(TYPE_ALIASES);
    input
        .iter()
        .filter_map(|k| map.get(k.as_str()).map(|s| (*s).to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_type_aliases() {
        let input = BTreeSet::from([
            "bool".to_string(),
            "bytes".to_string(),
            "char".to_string(),
            "float32".to_string(),
            "float64".to_string(),
            "int128".to_string(),
            "int16".to_string(),
            "int32".to_string(),
            "int64".to_string(),
            "int8".to_string(),
            "list_tuple".to_string(),
            "option".to_string(),
            "seq".to_string(),
            "str".to_string(),
            "tuple".to_string(),
            "uint128".to_string(),
            "uint16".to_string(),
            "uint32".to_string(),
            "uint64".to_string(),
            "uint8".to_string(),
            "unit".to_string(),
        ]);
        let actual = format_type_aliases(&input);
        insta::assert_snapshot!(&actual, @r"
        type bool = boolean;
        type bytes = Uint8Array;
        type char = string;
        type float32 = number;
        type float64 = number;
        type int128 = bigint;
        type int16 = number;
        type int32 = number;
        type int64 = bigint;
        type int8 = number;
        type ListTuple<T extends any[]> = Tuple<T>[];
        type Optional<T> = T | null;
        type Seq<T> = T[];
        type str = string;
        type Tuple<T extends any[]> = T;
        type uint128 = bigint;
        type uint16 = number;
        type uint32 = number;
        type uint64 = bigint;
        type uint8 = number;
        type unit = null;
        ");
    }
}
