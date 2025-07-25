use std::{collections::BTreeMap, io::Write};

use heck::ToUpperCamelCase;

use crate::{
    Registry,
    generation::{common, typescript::CodeGenerator},
    reflection::format::{ContainerFormat, Format, FormatHolder as _, Named, VariantFormat},
};

use super::super::indent::IndentedWriter;

/// Shared state for the code generation of a TypeScript source file.
pub(crate) struct TypeScriptEmitter<'a, T> {
    /// Writer.
    pub(crate) out: IndentedWriter<T>,
    /// Generator.
    pub(crate) generator: &'a CodeGenerator<'a>,
}

impl<T> TypeScriptEmitter<'_, T>
where
    T: Write,
{
    pub fn output_preamble(&mut self) -> std::io::Result<()> {
        writeln!(
            self.out,
            r"
import {{ Serializer, Deserializer }} from '../serde/mod.ts';
import {{ BcsSerializer, BcsDeserializer }} from '../bcs/mod.ts';
import {{ Optional, Seq, Tuple, ListTuple, unit, bool, int8, int16, int32, int64, int128, uint8, uint16, uint32, uint64, uint128, float32, float64, char, str, bytes }} from '../serde/mod.ts';
",
        )?;
        for namespace in &self.generator.namespaces_to_import {
            writeln!(
                self.out,
                "import * as {} from './{}';\n",
                namespace.to_upper_camel_case(),
                namespace
            )?;
        }

        Ok(())
    }

    fn quote_qualified_name(&self, name: &str) -> String {
        self.generator
            .external_qualified_names
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string())
    }

    fn output_comment(&mut self, name: &str) -> std::io::Result<()> {
        let path = vec![name.to_string()];
        if let Some(doc) = self.generator.config.comments.get(&path) {
            let text = textwrap::indent(doc, " * ").replace("\n\n", "\n *\n");
            writeln!(self.out, "/**\n{text} */")?;
        }
        Ok(())
    }

    fn quote_type(&self, format: &Format) -> String {
        use Format::{
            Bool, Bytes, Char, F32, F64, I8, I16, I32, I64, I128, Map, Option, Seq, Str, Tuple,
            TupleArray, TypeName, U8, U16, U32, U64, U128, Unit, Variable,
        };
        match format {
            TypeName(qualified_name) => self.quote_qualified_name(
                &qualified_name.to_legacy_string(ToUpperCamelCase::to_upper_camel_case),
            ),
            Unit => "unit".into(),
            Bool => "bool".into(),
            I8 => "int8".into(),
            I16 => "int16".into(),
            I32 => "int32".into(),
            I64 => "int64".into(),
            I128 => "int128".into(),
            U8 => "uint8".into(),
            U16 => "uint16".into(),
            U32 => "uint32".into(),
            U64 => "uint64".into(),
            U128 => "uint128".into(),
            F32 => "float32".into(),
            F64 => "float64".into(),
            Char => "char".into(),
            Str => "str".into(),
            Bytes => "bytes".into(),

            Option(format) => format!("Optional<{}>", self.quote_type(format)),
            Seq(format) => format!("Seq<{}>", self.quote_type(format)),
            Map { key, value } => {
                format!("Map<{},{}>", self.quote_type(key), self.quote_type(value))
            }
            Tuple(formats) => format!("Tuple<[{}]>", self.quote_types(formats, ", ")),
            TupleArray {
                content,
                size: _size,
            } => format!("ListTuple<[{}]>", self.quote_type(content),),
            Variable(_) => panic!("unexpected value"),
        }
    }

    fn quote_types(&self, formats: &[Format], sep: &str) -> String {
        formats
            .iter()
            .map(|f| self.quote_type(f))
            .collect::<Vec<_>>()
            .join(sep)
    }

    pub fn output_helpers(&mut self, registry: &Registry) -> std::io::Result<()> {
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

        writeln!(self.out, "export class Helpers {{")?;
        self.out.indent();
        for (mangled_name, subtype) in &subtypes {
            self.output_serialization_helper(mangled_name, subtype)?;
            self.output_deserialization_helper(mangled_name, subtype)?;
        }
        self.out.unindent();
        writeln!(self.out, "}}")?;
        writeln!(self.out)
    }

    fn needs_helper(format: &Format) -> bool {
        use Format::{Map, Option, Seq, Tuple, TupleArray};
        matches!(
            format,
            Option(_) | Seq(_) | Map { .. } | Tuple(_) | TupleArray { .. }
        )
    }

    #[allow(clippy::unused_self)]
    fn quote_serialize_value(&self, value: &str, format: &Format, use_this: bool) -> String {
        use Format::{
            Bool, Bytes, Char, F32, F64, I8, I16, I32, I64, I128, Str, TypeName, U8, U16, U32, U64,
            U128, Unit,
        };
        let this_str = if use_this { "this." } else { "" };

        match format {
            TypeName(_) => {
                format!("{this_str}{value}.serialize(serializer);")
            }
            Unit => format!("serializer.serializeUnit({this_str}{value});"),
            Bool => format!("serializer.serializeBool({this_str}{value});"),
            I8 => format!("serializer.serializeI8({this_str}{value});"),
            I16 => format!("serializer.serializeI16({this_str}{value});"),
            I32 => format!("serializer.serializeI32({this_str}{value});"),
            I64 => format!("serializer.serializeI64({this_str}{value});"),
            I128 => format!("serializer.serializeI128({this_str}{value});"),
            U8 => format!("serializer.serializeU8({this_str}{value});"),
            U16 => format!("serializer.serializeU16({this_str}{value});"),
            U32 => format!("serializer.serializeU32({this_str}{value});"),
            U64 => format!("serializer.serializeU64({this_str}{value});"),
            U128 => format!("serializer.serializeU128({this_str}{value});"),
            F32 => format!("serializer.serializeF32({this_str}{value});"),
            F64 => format!("serializer.serializeF64({this_str}{value});"),
            Char => format!("serializer.serializeChar({this_str}{value});"),
            Str => format!("serializer.serializeStr({this_str}{value});"),
            Bytes => format!("serializer.serializeBytes({this_str}{value});"),
            _ => format!(
                "Helpers.serialize{}({}{}, serializer);",
                common::mangle_type(format).to_upper_camel_case(),
                this_str,
                value
            ),
        }
    }

    fn quote_deserialize(&self, format: &Format) -> String {
        use Format::{
            Bool, Bytes, Char, F32, F64, I8, I16, I32, I64, I128, Str, TypeName, U8, U16, U32, U64,
            U128, Unit,
        };
        match format {
            TypeName(name) => format!(
                "{}.deserialize(deserializer)",
                self.quote_qualified_name(
                    &name.to_legacy_string(heck::ToUpperCamelCase::to_upper_camel_case)
                )
            ),
            Unit => "deserializer.deserializeUnit()".to_string(),
            Bool => "deserializer.deserializeBool()".to_string(),
            I8 => "deserializer.deserializeI8()".to_string(),
            I16 => "deserializer.deserializeI16()".to_string(),
            I32 => "deserializer.deserializeI32()".to_string(),
            I64 => "deserializer.deserializeI64()".to_string(),
            I128 => "deserializer.deserializeI128()".to_string(),
            U8 => "deserializer.deserializeU8()".to_string(),
            U16 => "deserializer.deserializeU16()".to_string(),
            U32 => "deserializer.deserializeU32()".to_string(),
            U64 => "deserializer.deserializeU64()".to_string(),
            U128 => "deserializer.deserializeU128()".to_string(),
            F32 => "deserializer.deserializeF32()".to_string(),
            F64 => "deserializer.deserializeF64()".to_string(),
            Char => "deserializer.deserializeChar()".to_string(),
            Str => "deserializer.deserializeStr()".to_string(),
            Bytes => "deserializer.deserializeBytes()".to_string(),
            _ => format!(
                "Helpers.deserialize{}(deserializer)",
                common::mangle_type(format).to_upper_camel_case(),
            ),
        }
    }

    fn output_serialization_helper(&mut self, name: &str, format0: &Format) -> std::io::Result<()> {
        use Format::{Map, Option, Seq, Tuple, TupleArray};

        write!(
            self.out,
            "static serialize{}(value: {}, serializer: Serializer): void {{",
            name.to_upper_camel_case(),
            self.quote_type(format0)
        )?;
        self.out.indent();
        match format0 {
            Option(format) => {
                write!(
                    self.out,
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

            Seq(format) => {
                write!(
                    self.out,
                    r"
serializer.serializeLen(value.length);
value.forEach((item: {}) => {{
    {}
}});
",
                    self.quote_type(format),
                    self.quote_serialize_value("item", format, false)
                )?;
            }

            Map { key, value } => {
                write!(
                    self.out,
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

            Tuple(format_list) => {
                writeln!(self.out)?;
                for (index, format) in format_list.iter().enumerate() {
                    let expr = format!("value[{index}]");
                    writeln!(
                        self.out,
                        "{}",
                        self.quote_serialize_value(&expr, format, false)
                    )?;
                }
            }

            TupleArray {
                content,
                size: _size,
            } => {
                write!(
                    self.out,
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
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    #[allow(clippy::too_many_lines)]
    fn output_deserialization_helper(
        &mut self,
        name: &str,
        format0: &Format,
    ) -> std::io::Result<()> {
        use Format::{Map, Option, Seq, Tuple, TupleArray};

        write!(
            self.out,
            "static deserialize{}(deserializer: Deserializer): {} {{",
            name.to_upper_camel_case(),
            self.quote_type(format0),
        )?;
        self.out.indent();
        match format0 {
            Option(format) => {
                write!(
                    self.out,
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

            Seq(format) => {
                write!(
                    self.out,
                    r"
const length = deserializer.deserializeLen();
const list: {} = [];
for (let i = 0; i < length; i++) {{
    list.push({});
}}
return list;
",
                    self.quote_type(format0),
                    self.quote_deserialize(format)
                )?;
            }

            Map { key, value } => {
                write!(
                    self.out,
                    r"
const length = deserializer.deserializeLen();
const obj = new Map<{0}, {1}>();
let previousKeyStart = 0;
let previousKeyEnd = 0;
for (let i = 0; i < length; i++) {{
    const keyStart = deserializer.getBufferOffset();
    const key = {2};
    const keyEnd = deserializer.getBufferOffset();
    if (i > 0) {{
        deserializer.checkThatKeySlicesAreIncreasing(
            [previousKeyStart, previousKeyEnd],
            [keyStart, keyEnd]);
    }}
    previousKeyStart = keyStart;
    previousKeyEnd = keyEnd;
    const value = {3};
    obj.set(key, value);
}}
return obj;
",
                    self.quote_type(key),
                    self.quote_type(value),
                    self.quote_deserialize(key),
                    self.quote_deserialize(value),
                )?;
            }

            Tuple(format_list) => {
                write!(
                    self.out,
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

            TupleArray { content, size } => {
                write!(
                    self.out,
                    r"
const list: {} = [];
for (let i = 0; i < {}; i++) {{
    list.push([{}]);
}}
return list;
",
                    self.quote_type(format0),
                    size,
                    self.quote_deserialize(content)
                )?;
            }

            _ => panic!("unexpected case"),
        }
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_variant(
        &mut self,
        base: &str,
        index: u32,
        name: &str,
        variant: &VariantFormat,
    ) -> std::io::Result<()> {
        use VariantFormat::{NewType, Struct, Tuple, Unit, Variable};
        let fields = match variant {
            Unit => Vec::new(),
            NewType(format) => vec![Named {
                name: "value".to_string(),
                value: format.as_ref().clone(),
            }],
            Tuple(formats) => formats
                .iter()
                .enumerate()
                .map(|(i, f)| Named {
                    name: format!("field{i}"),
                    value: f.clone(),
                })
                .collect(),
            Struct(fields) => fields.clone(),
            Variable(_) => panic!("incorrect value"),
        };
        self.output_struct_or_variant_container(Some(base), Some(index), name, &fields)
    }

    fn output_variants(
        &mut self,
        base: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> std::io::Result<()> {
        for (index, variant) in variants {
            self.output_variant(base, *index, &variant.name, &variant.value)?;
        }
        Ok(())
    }

    fn output_struct_or_variant_container(
        &mut self,
        variant_base: Option<&str>,
        variant_index: Option<u32>,
        name: &str,
        fields: &[Named<Format>],
    ) -> std::io::Result<()> {
        let mut variant_base_name = String::new();

        // Beginning of class
        if let Some(base) = variant_base {
            writeln!(self.out)?;
            self.output_comment(name)?;
            writeln!(
                self.out,
                "export class {base}Variant{name} extends {base} {{"
            )?;
            variant_base_name = format!("{base}Variant");
        } else {
            self.output_comment(name)?;
            writeln!(self.out, "export class {name} {{")?;
        }
        if !fields.is_empty() {
            writeln!(self.out)?;
        }
        // Constructor.
        writeln!(
            self.out,
            "constructor ({}) {{",
            fields
                .iter()
                .map(|f| { format!("public {}: {}", &f.name, self.quote_type(&f.value)) })
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if let Some(_base) = variant_base {
            self.out.indent();
            writeln!(self.out, "super();")?;
            self.out.unindent();
        }
        writeln!(self.out, "}}\n")?;
        // Serialize
        if self.generator.config.serialization {
            writeln!(
                self.out,
                "public serialize(serializer: Serializer): void {{",
            )?;
            self.out.indent();
            if let Some(index) = variant_index {
                writeln!(self.out, "serializer.serializeVariantIndex({index});")?;
            }
            for field in fields {
                writeln!(
                    self.out,
                    "{}",
                    self.quote_serialize_value(&field.name, &field.value, true)
                )?;
            }
            self.out.unindent();
            writeln!(self.out, "}}\n")?;
        }
        // Deserialize (struct) or Load (variant)
        if self.generator.config.serialization {
            if variant_index.is_none() {
                writeln!(
                    self.out,
                    "static deserialize(deserializer: Deserializer): {name} {{",
                )?;
            } else {
                writeln!(
                    self.out,
                    "static load(deserializer: Deserializer): {variant_base_name}{name} {{",
                )?;
            }
            self.out.indent();
            for field in fields {
                writeln!(
                    self.out,
                    "const {} = {};",
                    field.name,
                    self.quote_deserialize(&field.value)
                )?;
            }
            writeln!(
                self.out,
                r"return new {0}{1}({2});",
                variant_base_name,
                name,
                fields
                    .iter()
                    .map(|f| f.name.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
            self.out.unindent();
            writeln!(self.out, "}}\n")?;
        }
        writeln!(self.out, "}}")
    }

    fn output_enum_container(
        &mut self,
        name: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> std::io::Result<()> {
        self.output_comment(name)?;
        writeln!(self.out, "export abstract class {name} {{")?;
        if self.generator.config.serialization {
            writeln!(
                self.out,
                "abstract serialize(serializer: Serializer): void;\n"
            )?;
            write!(
                self.out,
                "static deserialize(deserializer: Deserializer): {name} {{"
            )?;
            self.out.indent();
            writeln!(
                self.out,
                r"
const index = deserializer.deserializeVariantIndex();
switch (index) {{",
            )?;
            self.out.indent();
            for (index, variant) in variants {
                writeln!(
                    self.out,
                    "case {}: return {}Variant{}.load(deserializer);",
                    index, name, variant.name,
                )?;
            }
            writeln!(
                self.out,
                "default: throw new Error(\"Unknown variant index for {name}: \" + index);",
            )?;
            self.out.unindent();
            writeln!(self.out, "}}")?;
            self.out.unindent();
            writeln!(self.out, "}}")?;
        }
        writeln!(self.out, "}}\n")?;
        self.output_variants(name, variants)?;
        Ok(())
    }

    pub fn output_container(
        &mut self,
        name: &str,
        format: &ContainerFormat,
    ) -> std::io::Result<()> {
        use ContainerFormat::{Enum, NewTypeStruct, Struct, TupleStruct, UnitStruct};
        let fields = match format {
            UnitStruct => Vec::new(),
            NewTypeStruct(format) => vec![Named {
                name: "value".to_string(),
                value: format.as_ref().clone(),
            }],
            TupleStruct(formats) => formats
                .iter()
                .enumerate()
                .map(|(i, f)| Named {
                    name: format!("field{i}"),
                    value: f.clone(),
                })
                .collect::<Vec<_>>(),
            Struct(fields) => fields.clone(),
            Enum(variants) => {
                self.output_enum_container(name, variants)?;
                return Ok(());
            }
        };
        self.output_struct_or_variant_container(None, None, name, &fields)
    }
}
