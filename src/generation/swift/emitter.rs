use std::collections::BTreeMap;
use std::io::{Result, Write};

use heck::{AsUpperCamelCase, ToLowerCamelCase as _, ToUpperCamelCase};

use crate::generation::Encoding;
use crate::generation::swift::generator::CodeGenerator;
use crate::reflection::format::{ContainerFormat, Named, VariantFormat};
use crate::{
    Registry,
    generation::{common, indent::IndentedWriter},
    reflection::format::{Format, FormatHolder as _},
};

/// Shared state for the code generation of a Swift source file.
pub struct SwiftEmitter<'a, T> {
    /// Writer.
    pub out: IndentedWriter<T>,
    /// Generator.
    pub generator: &'a CodeGenerator<'a>,
    /// Current namespace (e.g. vec!["Package", "`MyClass`"])
    pub current_namespace: Vec<String>,
}

impl<T> SwiftEmitter<'_, T>
where
    T: Write,
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
        use Format::{
            Bool, Bytes, Char, F32, F64, I8, I16, I32, I64, I128, Map, Option, Seq, Str, Tuple,
            TupleArray, TypeName, U8, U16, U32, U64, U128, Unit, Variable,
        };
        match format {
            TypeName(qualified_name) => self.quote_typename(&qualified_name.name),
            Unit => "Unit".into(),
            Bool => "Bool".into(),
            I8 => "Int8".into(),
            I16 => "Int16".into(),
            I32 => "Int32".into(),
            I64 => "Int64".into(),
            I128 => "Int128".into(),
            U8 => "UInt8".into(),
            U16 => "UInt16".into(),
            U32 => "UInt32".into(),
            U64 => "UInt64".into(),
            U128 => "UInt128".into(),
            F32 => "Float".into(),
            F64 => "Double".into(),
            Char => "Character".into(),
            Str => "String".into(),
            Bytes => "[UInt8]".into(),

            Option(format) => format!("{}?", self.quote_type(format)),
            Seq(format) => format!("[{}]", self.quote_type(format)),
            Map { key, value } => {
                format!("[{}: {}]", self.quote_type(key), self.quote_type(value))
            }
            // Sadly, Swift tuples are not hashable.
            Tuple(formats) => format!("Tuple{}<{}>", formats.len(), self.quote_types(formats)),
            TupleArray { content, size: _ } => {
                // Sadly, there are no fixed-size arrays in Swift.
                format!("[{}]", self.quote_type(content))
            }

            Variable(_) => panic!("unexpected value"),
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
        use Format::{Map, Option, Seq, Tuple, TupleArray};
        matches!(
            format,
            Option(_) | Seq(_) | Map { .. } | Tuple(_) | TupleArray { .. }
        )
    }

    #[allow(clippy::unused_self)]
    fn quote_serialize_value(&self, value: &str, format: &Format) -> String {
        use Format::{
            Bool, Bytes, Char, F32, F64, I8, I16, I32, I64, I128, Str, TypeName, U8, U16, U32, U64,
            U128, Unit,
        };
        match format {
            TypeName(_) => {
                format!("try {value}.serialize(serializer: serializer)")
            }
            Unit => format!("try serializer.serialize_unit(value: {value})"),
            Bool => format!("try serializer.serialize_bool(value: {value})"),
            I8 => format!("try serializer.serialize_i8(value: {value})"),
            I16 => format!("try serializer.serialize_i16(value: {value})"),
            I32 => format!("try serializer.serialize_i32(value: {value})"),
            I64 => format!("try serializer.serialize_i64(value: {value})"),
            I128 => format!("try serializer.serialize_i128(value: {value})"),
            U8 => format!("try serializer.serialize_u8(value: {value})"),
            U16 => format!("try serializer.serialize_u16(value: {value})"),
            U32 => format!("try serializer.serialize_u32(value: {value})"),
            U64 => format!("try serializer.serialize_u64(value: {value})"),
            U128 => format!("try serializer.serialize_u128(value: {value})"),
            F32 => format!("try serializer.serialize_f32(value: {value})"),
            F64 => format!("try serializer.serialize_f64(value: {value})"),
            Char => format!("try serializer.serialize_char(value: {value})"),
            Str => format!("try serializer.serialize_str(value: {value})"),
            Bytes => format!("try serializer.serialize_bytes(value: {value})"),
            _ => format!(
                "try serialize_{}(value: {}, serializer: serializer)",
                common::mangle_type(format),
                value
            ),
        }
    }

    // TODO: Should this be an extension for Serializer?
    fn output_serialization_helper(&mut self, name: &str, format0: &Format) -> Result<()> {
        use Format::{Map, Option, Seq, Tuple, TupleArray};

        write!(
            self.out,
            "func serialize_{}<S: Serializer>(value: {}, serializer: S) throws {{",
            name,
            self.quote_type(format0)
        )?;
        self.out.indent();
        match format0 {
            Option(format) => {
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

            Seq(format) => {
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

            Map { key, value } => {
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

            Tuple(format_list) => {
                writeln!(self.out)?;
                for (index, format) in format_list.iter().enumerate() {
                    let expr = format!("value.field{index}");
                    writeln!(self.out, "{}", self.quote_serialize_value(&expr, format))?;
                }
            }

            TupleArray { content, size: _ } => {
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
        use Format::{Map, Option, Seq, Tuple, TupleArray};

        write!(
            self.out,
            "func deserialize_{}<D: Deserializer>(deserializer: D) throws -> {} {{",
            name,
            self.quote_type(format0),
        )?;
        self.out.indent();
        match format0 {
            Option(format) => {
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

            Seq(format) => {
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

            Map { key, value } => {
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

            Tuple(format_list) => {
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

            TupleArray { content, size } => {
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
        use VariantFormat::{NewType, Struct, Tuple, Unit, Variable};
        self.output_comment(name)?;
        let name = name.to_lower_camel_case();
        match variant {
            Unit => {
                writeln!(self.out, "case {name}")?;
            }
            NewType(format) => {
                writeln!(self.out, "case {}({})", name, self.quote_type(format))?;
            }
            Tuple(formats) => {
                if formats.is_empty() {
                    writeln!(self.out, "case {name}")?;
                } else {
                    writeln!(self.out, "case {}({})", name, self.quote_types(formats))?;
                }
            }
            Struct(fields) => {
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
            Variable(_) => panic!("incorrect value"),
        }
        Ok(())
    }

    fn variant_fields(variant: &VariantFormat) -> Vec<Named<Format>> {
        use VariantFormat::{NewType, Struct, Tuple, Unit, Variable};
        match variant {
            Unit => Vec::new(),
            NewType(format) => vec![Named {
                name: "x".to_string(),
                value: format.as_ref().clone(),
            }],
            Tuple(formats) => formats
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, f)| Named {
                    name: format!("x{i}"),
                    value: f,
                })
                .collect(),
            Struct(fields) => fields.clone(),
            Variable(_) => panic!("incorrect value"),
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
        if self.generator.config.serialization {
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

            for encoding in &self.generator.config.encodings {
                self.output_struct_serialize_for_encoding(*encoding)?;
            }
        }
        // Deserialize
        if self.generator.config.serialization {
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

            for encoding in &self.generator.config.encodings {
                self.output_struct_deserialize_for_encoding(name, *encoding)?;
            }
        }
        // Custom code
        self.output_custom_code(name)?;
        self.leave_class();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    fn output_struct_serialize_for_encoding(&mut self, encoding: Encoding) -> Result<()> {
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

    fn output_struct_deserialize_for_encoding(
        &mut self,
        name: &str,
        encoding: Encoding,
    ) -> Result<()> {
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
        if self.generator.config.serialization {
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

            for encoding in &self.generator.config.encodings {
                self.output_struct_serialize_for_encoding(*encoding)?;
            }
        }
        // Deserialize
        if self.generator.config.serialization {
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

            for encoding in &self.generator.config.encodings {
                self.output_struct_deserialize_for_encoding(name, *encoding)?;
            }
        }

        self.current_namespace.pop();
        // Custom code
        self.output_custom_code(name)?;
        self.out.unindent();
        writeln!(self.out, "}}")?;
        Ok(())
    }

    pub fn output_container(&mut self, name: &str, format: &ContainerFormat) -> Result<()> {
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
                .collect(),
            Struct(fields) => fields
                .iter()
                .map(|f| Named {
                    name: f.name.to_lower_camel_case(),
                    value: f.value.clone(),
                })
                .collect(),
            Enum(variants) => {
                self.output_enum_container(name, variants)?;
                return Ok(());
            }
        };
        self.output_struct_container(name, &fields)
    }
}

fn quote_deserialize(format: &Format) -> String {
    use Format::{
        Bool, Bytes, Char, F32, F64, I8, I16, I32, I64, I128, Str, TypeName, U8, U16, U32, U64,
        U128, Unit,
    };
    match format {
        TypeName(name) => format!(
            "try {}.deserialize(deserializer: deserializer)",
            &name.to_legacy_string(ToUpperCamelCase::to_upper_camel_case)
        ),
        Unit => "try deserializer.deserialize_unit()".to_string(),
        Bool => "try deserializer.deserialize_bool()".to_string(),
        I8 => "try deserializer.deserialize_i8()".to_string(),
        I16 => "try deserializer.deserialize_i16()".to_string(),
        I32 => "try deserializer.deserialize_i32()".to_string(),
        I64 => "try deserializer.deserialize_i64()".to_string(),
        I128 => "try deserializer.deserialize_i128()".to_string(),
        U8 => "try deserializer.deserialize_u8()".to_string(),
        U16 => "try deserializer.deserialize_u16()".to_string(),
        U32 => "try deserializer.deserialize_u32()".to_string(),
        U64 => "try deserializer.deserialize_u64()".to_string(),
        U128 => "try deserializer.deserialize_u128()".to_string(),
        F32 => "try deserializer.deserialize_f32()".to_string(),
        F64 => "try deserializer.deserialize_f64()".to_string(),
        Char => "try deserializer.deserialize_char()".to_string(),
        Str => "try deserializer.deserialize_str()".to_string(),
        Bytes => "try deserializer.deserialize_bytes()".to_string(),
        _ => format!(
            "try deserialize_{}(deserializer: deserializer)",
            common::mangle_type(format)
        ),
    }
}
