use std::collections::BTreeMap;
use std::io::Result;

use heck::{AsUpperCamelCase, ToLowerCamelCase as _, ToUpperCamelCase};

use crate::generation::swift::generator::CodeGenerator;
use crate::reflection::format::{ContainerFormat, Doc, Named, VariantFormat};
use crate::{
    Registry,
    generation::{common, indent::IndentWrite},
    reflection::format::{Format, FormatHolder as _},
};

/// Shared state for the code generation of a Swift source file.
pub struct SwiftEmitter<'a, T> {
    /// Writer.
    pub out: T,
    /// Generator.
    pub generator: &'a CodeGenerator<'a>,
    /// Current namespace (e.g. vec!["Package", "`MyClass`"])
    pub current_namespace: Vec<String>,
}

impl<T> SwiftEmitter<'_, T>
where
    T: IndentWrite,
{
    pub fn output_preamble(&mut self) -> Result<()> {
        let mut imports = ["Serde".to_string()]
            .iter()
            .chain(self.generator.config.external_definitions.keys())
            .chain(self.generator.config.external_packages.keys())
            .map(AsUpperCamelCase)
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        imports.sort();
        imports.dedup();

        for import in imports {
            writeln!(self.out, "import {import}")?;
        }

        Ok(())
    }

    fn output_comment(&mut self, name: &str) -> std::io::Result<()> {
        let mut path = self.current_namespace.clone();
        path.push(name.to_string());
        if let Some(doc) = self.generator.config.comments.get(&path) {
            let text = textwrap::indent(doc, "// ").replace("\n\n", "\n//\n");
            write!(self.out, "{text}")?;
        }
        Ok(())
    }

    fn output_custom_code(&mut self, name: &str) -> std::io::Result<()> {
        let mut path = self.current_namespace.clone();
        path.push(name.to_string());
        if let Some(code) = self.generator.config.custom_code.get(&path) {
            writeln!(self.out, "\n{code}")?;
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

    fn quote_type(&self, format: &Format) -> String {
        match format {
            Format::TypeName(type_) => self.quote_typename(&type_.name),
            Format::Unit => "Unit".into(),
            Format::Bool => "Bool".into(),
            Format::I8 => "Int8".into(),
            Format::I16 => "Int16".into(),
            Format::I32 => "Int32".into(),
            Format::I64 => "Int64".into(),
            Format::I128 => "Int128".into(),
            Format::U8 => "UInt8".into(),
            Format::U16 => "UInt16".into(),
            Format::U32 => "UInt32".into(),
            Format::U64 => "UInt64".into(),
            Format::U128 => "UInt128".into(),
            Format::F32 => "Float".into(),
            Format::F64 => "Double".into(),
            Format::Char => "Character".into(),
            Format::Str => "String".into(),
            Format::Bytes => "[UInt8]".into(),

            Format::Option(format) => format!("{}?", self.quote_type(format)),
            Format::Seq(format) | Format::Set(format) => format!("[{}]", self.quote_type(format)),
            Format::Map { key, value } => {
                format!("[{}: {}]", self.quote_type(key), self.quote_type(value))
            }
            // Sadly, Swift tuples are not hashable.
            Format::Tuple(formats) => {
                format!("Tuple{}<{}>", formats.len(), self.quote_types(formats))
            }
            Format::TupleArray { content, size: _ } => {
                // Sadly, there are no fixed-size arrays in Swift.
                format!("[{}]", self.quote_type(content))
            }

            Format::Variable(_) => panic!("unexpected value"),
        }
    }

    fn quote_types<'b, I>(&'b self, formats: I) -> String
    where
        I: IntoIterator<Item = &'b Format>,
    {
        formats
            .into_iter()
            .map(|format| self.quote_type(format))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn enter_class(&mut self, name: &str) {
        self.out.indent();
        self.current_namespace.push(name.to_string());
    }

    fn leave_class(&mut self) {
        self.out.unindent();
        self.current_namespace.pop();
    }

    pub fn output_trait_helpers(&mut self, registry: &Registry) -> Result<()> {
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
        for (mangled_name, subtype) in &subtypes {
            self.output_serialization_helper(mangled_name, subtype)?;
            self.output_deserialization_helper(mangled_name, subtype)?;
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

    #[allow(clippy::unused_self)]
    fn quote_serialize_value(&self, value: &str, format: &Format) -> String {
        match format {
            Format::TypeName(_) => {
                format!("try {value}.serialize(serializer: serializer)")
            }
            Format::Unit => format!("try serializer.serialize_unit(value: {value})"),
            Format::Bool => format!("try serializer.serialize_bool(value: {value})"),
            Format::I8 => format!("try serializer.serialize_i8(value: {value})"),
            Format::I16 => format!("try serializer.serialize_i16(value: {value})"),
            Format::I32 => format!("try serializer.serialize_i32(value: {value})"),
            Format::I64 => format!("try serializer.serialize_i64(value: {value})"),
            Format::I128 => format!("try serializer.serialize_i128(value: {value})"),
            Format::U8 => format!("try serializer.serialize_u8(value: {value})"),
            Format::U16 => format!("try serializer.serialize_u16(value: {value})"),
            Format::U32 => format!("try serializer.serialize_u32(value: {value})"),
            Format::U64 => format!("try serializer.serialize_u64(value: {value})"),
            Format::U128 => format!("try serializer.serialize_u128(value: {value})"),
            Format::F32 => format!("try serializer.serialize_f32(value: {value})"),
            Format::F64 => format!("try serializer.serialize_f64(value: {value})"),
            Format::Char => format!("try serializer.serialize_char(value: {value})"),
            Format::Str => format!("try serializer.serialize_str(value: {value})"),
            Format::Bytes => format!("try serializer.serialize_bytes(value: {value})"),
            _ => format!(
                "try serialize_{}(value: {}, serializer: serializer)",
                common::mangle_type(format),
                value
            ),
        }
    }

    // TODO: Should this be an extension for Serializer?
    fn output_serialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
        write!(
            self.out,
            "func serialize_{}<S: Serializer>(value: {}, serializer: S) throws {{",
            name,
            self.quote_type(format0)
        )?;
        self.out.indent();
        match format0 {
            Format::Option(format) => {
                write!(
                    self.out,
                    r"
if let value = value {{
    try serializer.serialize_option_tag(value: true)
    {}
}} else {{
    try serializer.serialize_option_tag(value: false)
}}
",
                    self.quote_serialize_value("value", format)
                )?;
            }

            Format::Seq(format) | Format::Set(format) => {
                write!(
                    self.out,
                    r"
try serializer.serialize_len(value: value.count)
for item in value {{
    {}
}}
",
                    self.quote_serialize_value("item", format)
                )?;
            }

            Format::Map { key, value } => {
                write!(
                    self.out,
                    r"
try serializer.serialize_len(value: value.count)
var offsets : [Int]  = []
for (key, value) in value {{
    offsets.append(serializer.get_buffer_offset())
    {}
    {}
}}
serializer.sort_map_entries(offsets: offsets)
",
                    self.quote_serialize_value("key", key),
                    self.quote_serialize_value("value", value)
                )?;
            }

            Format::Tuple(format_list) => {
                writeln!(self.out)?;
                for (index, format) in format_list.iter().enumerate() {
                    let expr = format!("value.field{index}");
                    writeln!(self.out, "{}", self.quote_serialize_value(&expr, format))?;
                }
            }

            Format::TupleArray { content, size: _ } => {
                write!(
                    self.out,
                    r"
for item in value {{
    {}
}}
",
                    self.quote_serialize_value("item", content),
                )?;
            }

            _ => panic!("unexpected case"),
        }
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_deserialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
        write!(
            self.out,
            "func deserialize_{}<D: Deserializer>(deserializer: D) throws -> {} {{",
            name,
            self.quote_type(format0),
        )?;
        self.out.indent();
        match format0 {
            Format::Option(format) => {
                write!(
                    self.out,
                    r"
let tag = try deserializer.deserialize_option_tag()
if tag {{
    return {}
}} else {{
    return nil
}}
",
                    quote_deserialize(format),
                )?;
            }

            Format::Seq(format) | Format::Set(format) => {
                write!(
                    self.out,
                    r"
let length = try deserializer.deserialize_len()
var obj : [{}] = []
for _ in 0..<length {{
    obj.append({})
}}
return obj
",
                    self.quote_type(format),
                    quote_deserialize(format)
                )?;
            }

            Format::Map { key, value } => {
                write!(
                    self.out,
                    r"
let length = try deserializer.deserialize_len()
var obj : [{0}: {1}] = [:]
var previous_slice = Slice(start: 0, end: 0)
for i in 0..<length {{
    var slice = Slice(start: 0, end: 0)
    slice.start = deserializer.get_buffer_offset()
    let key = {2}
    slice.end = deserializer.get_buffer_offset()
    if i > 0 {{
        try deserializer.check_that_key_slices_are_increasing(key1: previous_slice, key2: slice)
    }}
    previous_slice = slice
    obj[key] = {3}
}}
return obj
",
                    self.quote_type(key),
                    self.quote_type(value),
                    quote_deserialize(key),
                    quote_deserialize(value),
                )?;
            }

            Format::Tuple(format_list) => {
                write!(
                    self.out,
                    r"
return Tuple{}.init({})
",
                    format_list.len(),
                    format_list
                        .iter()
                        .map(quote_deserialize)
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }

            Format::TupleArray { content, size } => {
                write!(
                    self.out,
                    r"
var obj : [{}] = []
for _ in 0..<{} {{
    obj.append({})
}}
return obj
",
                    self.quote_type(content),
                    size,
                    quote_deserialize(content)
                )?;
            }

            _ => panic!("unexpected case"),
        }
        self.out.unindent();
        writeln!(self.out, "}}\n")
    }

    fn output_variant(&mut self, name: &str, variant: &VariantFormat) -> Result<()> {
        self.output_comment(name)?;
        let name = name.to_lower_camel_case();
        match variant {
            VariantFormat::Unit => {
                writeln!(self.out, "case {name}")?;
            }
            VariantFormat::NewType(format) => {
                writeln!(self.out, "case {}({})", name, self.quote_type(format))?;
            }
            VariantFormat::Tuple(formats) => {
                if formats.is_empty() {
                    writeln!(self.out, "case {name}")?;
                } else {
                    writeln!(self.out, "case {}({})", name, self.quote_types(formats))?;
                }
            }
            VariantFormat::Struct(fields) => {
                if fields.is_empty() {
                    writeln!(self.out, "case {name}")?;
                } else {
                    writeln!(
                        self.out,
                        "case {}({})",
                        name,
                        fields
                            .iter()
                            .map(|f| format!("{}: {}", f.name, self.quote_type(&f.value)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )?;
                }
            }
            VariantFormat::Variable(_) => panic!("incorrect value"),
        }
        Ok(())
    }

    fn variant_fields(variant: &VariantFormat) -> Vec<Named<Format>> {
        match variant {
            VariantFormat::Unit => Vec::new(),
            VariantFormat::NewType(format) => vec![Named {
                name: "x".to_string(),
                doc: Doc::new(),
                value: format.as_ref().clone(),
            }],
            VariantFormat::Tuple(formats) => formats
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, f)| Named {
                    name: format!("x{i}"),
                    doc: Doc::new(),
                    value: f,
                })
                .collect(),
            VariantFormat::Struct(fields) => fields.clone(),
            VariantFormat::Variable(_) => panic!("incorrect value"),
        }
    }

    fn output_struct_container(&mut self, name: &str, fields: &[Named<Format>]) -> Result<()> {
        // Struct
        writeln!(self.out)?;
        self.output_comment(name)?;
        writeln!(self.out, "public struct {name}: Hashable {{")?;
        self.enter_class(name);
        for field in fields {
            self.output_comment(&field.name)?;
            writeln!(
                self.out,
                "@Indirect public var {}: {}",
                field.name,
                self.quote_type(&field.value)
            )?;
        }
        // Public constructor
        writeln!(
            self.out,
            "\npublic init({}) {{",
            fields
                .iter()
                .map(|f| format!("{}: {}", &f.name, self.quote_type(&f.value)))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        self.out.indent();
        for field in fields {
            writeln!(self.out, "self.{0} = {0}", &field.name)?;
        }
        self.out.unindent();
        writeln!(self.out, "}}")?;
        // Serialize
        if self.generator.config.has_encoding() {
            writeln!(
                self.out,
                "\npublic func serialize<S: Serializer>(serializer: S) throws {{",
            )?;
            self.out.indent();
            writeln!(self.out, "try serializer.increase_container_depth()")?;
            for field in fields {
                writeln!(
                    self.out,
                    "{}",
                    self.quote_serialize_value(&format!("self.{}", &field.name), &field.value)
                )?;
            }
            writeln!(self.out, "try serializer.decrease_container_depth()")?;
            self.out.unindent();
            writeln!(self.out, "}}")?;

            self.output_struct_serialize_for_encoding()?;
            // Deserialize
            writeln!(
                self.out,
                "\npublic static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} {{",
            )?;
            self.out.indent();
            writeln!(self.out, "try deserializer.increase_container_depth()")?;
            for field in fields {
                writeln!(
                    self.out,
                    "let {} = {}",
                    field.name,
                    quote_deserialize(&field.value)
                )?;
            }
            writeln!(self.out, "try deserializer.decrease_container_depth()")?;
            writeln!(
                self.out,
                "return {}.init({})",
                name,
                fields
                    .iter()
                    .map(|f| format!("{0}: {0}", &f.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
            self.out.unindent();
            writeln!(self.out, "}}")?;

            self.output_struct_deserialize_for_encoding(name)?;
        }
        // Custom code
        self.output_custom_code(name)?;
        self.leave_class();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_struct_serialize_for_encoding(&mut self) -> Result<()> {
        let encoding = self.generator.config.encoding;
        writeln!(
            self.out,
            r"
public func {0}Serialize() throws -> [UInt8] {{
    let serializer = {1}Serializer.init();
    try self.serialize(serializer: serializer)
    return serializer.get_bytes()
}}",
            encoding.name(),
            encoding.name().to_upper_camel_case()
        )
    }

    fn output_struct_deserialize_for_encoding(&mut self, name: &str) -> Result<()> {
        let encoding = self.generator.config.encoding;
        writeln!(
            self.out,
            r#"
public static func {1}Deserialize(input: [UInt8]) throws -> {0} {{
    let deserializer = {2}Deserializer.init(input: input);
    let obj = try deserialize(deserializer: deserializer)
    if deserializer.get_buffer_offset() < input.count {{
        throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
    }}
    return obj
}}"#,
            name,
            encoding.name(),
            encoding.name().to_upper_camel_case(),
        )
    }

    #[allow(clippy::too_many_lines)]
    fn output_enum_container(
        &mut self,
        name: &str,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
    ) -> Result<()> {
        writeln!(self.out)?;
        self.output_comment(name)?;
        writeln!(self.out, "indirect public enum {name}: Hashable {{")?;
        self.current_namespace.push(name.to_string());
        self.out.indent();
        for variant in variants.values() {
            self.output_variant(&variant.name, &variant.value)?;
        }

        // Serialize
        if self.generator.config.has_encoding() {
            writeln!(
                self.out,
                "\npublic func serialize<S: Serializer>(serializer: S) throws {{",
            )?;
            self.out.indent();
            writeln!(self.out, "try serializer.increase_container_depth()")?;
            writeln!(self.out, "switch self {{")?;
            for (index, variant) in variants {
                let fields = Self::variant_fields(&variant.value);
                let formatted_variant_name = &variant.name.to_lower_camel_case();
                if fields.is_empty() {
                    writeln!(self.out, "case .{formatted_variant_name}:")?;
                } else {
                    writeln!(
                        self.out,
                        "case .{}({}):",
                        formatted_variant_name,
                        fields
                            .iter()
                            .map(|f| format!("let {}", f.name))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )?;
                }
                self.out.indent();
                writeln!(
                    self.out,
                    "try serializer.serialize_variant_index(value: {index})"
                )?;
                for field in fields {
                    writeln!(
                        self.out,
                        "{}",
                        self.quote_serialize_value(&field.name, &field.value)
                    )?;
                }
                self.out.unindent();
            }
            writeln!(self.out, "}}")?;
            writeln!(self.out, "try serializer.decrease_container_depth()")?;
            self.out.unindent();
            writeln!(self.out, "}}")?;

            self.output_struct_serialize_for_encoding()?;

            // Deserialize
            write!(
                self.out,
                "\npublic static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} {{"
            )?;
            self.out.indent();
            writeln!(
                self.out,
                r"
let index = try deserializer.deserialize_variant_index()
try deserializer.increase_container_depth()
switch index {{",
            )?;
            for (index, variant) in variants {
                writeln!(self.out, "case {index}:")?;
                self.out.indent();
                let formatted_variant_name = &variant.name.to_lower_camel_case();
                let fields = Self::variant_fields(&variant.value);
                if fields.is_empty() {
                    writeln!(self.out, "try deserializer.decrease_container_depth()")?;
                    writeln!(self.out, "return .{formatted_variant_name}")?;
                    self.out.unindent();
                    continue;
                }
                for field in &fields {
                    writeln!(
                        self.out,
                        "let {} = {}",
                        field.name,
                        quote_deserialize(&field.value)
                    )?;
                }
                writeln!(self.out, "try deserializer.decrease_container_depth()")?;
                let init_values = match &variant.value {
                    VariantFormat::Struct(_) => fields
                        .iter()
                        .map(|f| format!("{0}: {0}", f.name))
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => fields
                        .iter()
                        .map(|f| f.name.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                };
                writeln!(self.out, "return .{formatted_variant_name}({init_values})")?;
                self.out.unindent();
            }
            writeln!(
                self.out,
                "default: throw DeserializationError.invalidInput(issue: \"Unknown variant index for {name}: \\(index)\")",
            )?;
            writeln!(self.out, "}}")?;
            self.out.unindent();
            writeln!(self.out, "}}")?;

            self.output_struct_deserialize_for_encoding(name)?;
        }

        self.current_namespace.pop();
        // Custom code
        self.output_custom_code(name)?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    pub fn output_container(&mut self, name: &str, format: &ContainerFormat) -> Result<()> {
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
                .collect(),
            ContainerFormat::Struct(fields, _doc) => fields
                .iter()
                .map(|f| Named {
                    name: f.name.to_lower_camel_case(),
                    doc: Doc::new(),
                    value: f.value.clone(),
                })
                .collect(),
            ContainerFormat::Enum(variants, _doc) => {
                self.output_enum_container(name, variants)?;
                return Ok(());
            }
        };
        self.output_struct_container(name, &fields)
    }
}

fn quote_deserialize(format: &Format) -> String {
    match format {
        Format::TypeName(name) => format!(
            "try {}.deserialize(deserializer: deserializer)",
            &name.to_legacy_string(ToUpperCamelCase::to_upper_camel_case)
        ),
        Format::Unit => "try deserializer.deserialize_unit()".to_string(),
        Format::Bool => "try deserializer.deserialize_bool()".to_string(),
        Format::I8 => "try deserializer.deserialize_i8()".to_string(),
        Format::I16 => "try deserializer.deserialize_i16()".to_string(),
        Format::I32 => "try deserializer.deserialize_i32()".to_string(),
        Format::I64 => "try deserializer.deserialize_i64()".to_string(),
        Format::I128 => "try deserializer.deserialize_i128()".to_string(),
        Format::U8 => "try deserializer.deserialize_u8()".to_string(),
        Format::U16 => "try deserializer.deserialize_u16()".to_string(),
        Format::U32 => "try deserializer.deserialize_u32()".to_string(),
        Format::U64 => "try deserializer.deserialize_u64()".to_string(),
        Format::U128 => "try deserializer.deserialize_u128()".to_string(),
        Format::F32 => "try deserializer.deserialize_f32()".to_string(),
        Format::F64 => "try deserializer.deserialize_f64()".to_string(),
        Format::Char => "try deserializer.deserialize_char()".to_string(),
        Format::Str => "try deserializer.deserialize_str()".to_string(),
        Format::Bytes => "try deserializer.deserialize_bytes()".to_string(),
        _ => format!(
            "try deserialize_{}(deserializer: deserializer)",
            common::mangle_type(format)
        ),
    }
}
