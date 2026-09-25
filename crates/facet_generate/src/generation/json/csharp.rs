//! `EmitterPlugin<CSharp>` implementation for the [`JsonPlugin`].
//!
//! Every generated type names a generated `System.Text.Json` converter, which
//! writes it in the shape `serde_json` gives the same Rust type — so JSON
//! written on either side reads on the other, with the default
//! `JsonSerializerOptions`.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `using Facet.Runtime.Json;`, `using System.Text.Json;` and `using System.Text.Json.Serialization;` |
//! | `type_annotations` | `[JsonConverter(typeof({Type}JsonConverter))]` |
//! | `field_annotations` | `[property: JsonPropertyName("…")]` with a struct field's wire name |
//! | `has_type_body` | `true` for non-unit-enum types |
//! | `type_body` | `JsonSerialize` / `JsonDeserialize` static helper methods |
//! | `after_type` | the type's `{Type}JsonConverter` |
//! | `runtime_files` | the serde runtime, `JsonSerde.cs` and `FacetJson.cs` |
//!
//! # Wire format
//!
//! | Rust | JSON |
//! |---|---|
//! | struct with named fields | object keyed by the field's (renamed) Rust name |
//! | unit struct, `()` | `null` |
//! | newtype struct | the inner value |
//! | tuple struct, tuple | array |
//! | `Option<T>` | `null` or the value (a missing key reads as `null`) |
//! | `Vec<T>`, `[T; N]`, sets, bytes | array |
//! | map | object, with integer, boolean and newtype keys written as strings |
//! | `char`, `Uuid` | string |
//! | 64- and 128-bit integer | number, written and read exactly |
//! | float | number, always with a fraction or an exponent; `null` when not finite |
//! | enum | externally tagged — `"Unit"`, `{"NewType": …}`, `{"Tuple": […]}`, `{"Struct": {…}}` — or internally / adjacently tagged per `#[facet(tag, content)]` |
//!
//! Each converter is built from the runtime's `FacetJson` converters, composed
//! as the field's format is, and reaches another generated type through the
//! converter its own attribute names.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io;

use heck::ToUpperCamelCase;

use crate::generation::{
    CodeGeneratorConfig, Feature,
    csharp::{CSharp, is_value_type, naming, render_type_hiding},
    indent::{IndentWrite, Newlines, with_block},
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
};
use crate::reflection::format::{ContainerFormat, EnumTagging, Format, Named, VariantFormat};

use super::JsonPlugin;

// ---------------------------------------------------------------------------
// EmitterPlugin implementation
// ---------------------------------------------------------------------------

impl EmitterPlugin<CSharp> for JsonPlugin {
    /// Returns the core, serde, and JSON C# runtime sources to be written
    /// into the output directory alongside the generated code.
    fn runtime_files(&self) -> Vec<RuntimeFile> {
        vec![
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/Unit.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/core/Unit.cs").to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/ISerializer.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/serde/ISerializer.cs")
                    .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/IDeserializer.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/serde/IDeserializer.cs")
                    .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/DeserializationError.cs".to_string(),
                contents: include_bytes!(
                    "../csharp/installer/runtime/serde/DeserializationError.cs"
                )
                .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Serde/SerializationError.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/serde/SerializationError.cs")
                    .to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Json/JsonSerde.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/json/JsonSerde.cs").to_vec(),
            },
            RuntimeFile {
                relative_path: "Facet/Runtime/Json/FacetJson.cs".to_string(),
                contents: include_bytes!("../csharp/installer/runtime/json/FacetJson.cs").to_vec(),
            },
        ]
    }

    /// Returns `using` directives for JSON support.
    ///
    /// Always includes `Facet.Runtime.Json`, `System.Text.Json` and
    /// `System.Text.Json.Serialization`. When `Feature::Uuid` is active, also
    /// adds `System` so that `Guid` resolves.
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let mut imports = vec![
            "using Facet.Runtime.Json;".to_string(),
            "using System.Text.Json;".to_string(),
            "using System.Text.Json.Serialization;".to_string(),
        ];
        if config.features.contains(&Feature::Uuid) {
            imports.push("using System;".to_string());
        }
        imports
    }

    /// `[JsonConverter(typeof({Type}JsonConverter))]`, naming the converter
    /// [`after_type`](Self::after_type) writes.
    fn type_annotations(&self, ctx: &EmitContext) -> Vec<String> {
        if ctx.is_variant() {
            return vec![];
        }
        vec![format!(
            "[JsonConverter(typeof({}))]",
            converter_name(ctx.name())
        )]
    }

    /// `[property: JsonPropertyName("…")]` with the wire name of a struct's
    /// field, which the MVVM toolkit puts on the property it generates. A
    /// newtype or tuple struct's JSON has no field names.
    fn field_annotations(&self, field: &Named<Format>, ctx: &EmitContext) -> Vec<String> {
        if !matches!(ctx.container.format, ContainerFormat::Struct(..)) {
            return vec![];
        }
        vec![format!(
            "[property: JsonPropertyName({})]",
            literal(&field.name)
        )]
    }

    /// Returns `true` for types that need `JsonSerialize` / `JsonDeserialize`
    /// methods — i.e. everything except all-unit enums (plain C# `enum` types
    /// don't need instance helpers).
    fn has_type_body(&self, ctx: &EmitContext) -> bool {
        if let ContainerFormat::Enum(variants, _, _) = ctx.container.format
            && is_unit_enum(variants)
        {
            return false;
        }
        true
    }

    /// Emits `JsonSerialize` and `JsonDeserialize` convenience methods.
    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        let type_name = ctx.name().to_upper_camel_case();
        // A property (or, in a variant hierarchy, a nested variant record)
        // named `JsonSerde` would hide the runtime class, so reach it through
        // its qualified name then.
        let hides_json_serde = match ctx.container.format {
            ContainerFormat::Enum(variants, _, _) => variants
                .values()
                .any(|v| v.name.to_upper_camel_case() == JSON_SERDE),
            _ => ctx
                .fields()
                .iter()
                .any(|f| f.name.to_upper_camel_case() == JSON_SERDE),
        };
        let json_serde = if hides_json_serde {
            format!("global::Facet.Runtime.Json.{JSON_SERDE}")
        } else {
            JSON_SERDE.to_string()
        };
        write_json_helpers(w, &type_name, &json_serde)
    }

    /// Emits the type's `{Type}JsonConverter`, after the type.
    fn after_type(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        writeln!(w)?;
        Converter::new(ctx).write(w)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The runtime class the JSON helpers call, in `Facet.Runtime.Json`.
const JSON_SERDE: &str = "JsonSerde";

/// Writes `JsonSerialize` / `JsonDeserialize` methods backed by `JsonSerde`,
/// called through `json_serde`.
fn write_json_helpers(
    w: &mut dyn IndentWrite,
    type_name: &str,
    json_serde: &str,
) -> io::Result<()> {
    writeln!(w, "public string JsonSerialize()")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "return {json_serde}.Serialize(this);")
    })?;
    writeln!(w)?;
    writeln!(w, "public static {type_name} JsonDeserialize(string input)")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "return {json_serde}.Deserialize<{type_name}>(input);")
    })?;
    Ok(())
}

/// The name of the converter generated for the type `name`.
fn converter_name(name: &str) -> String {
    format!("{}JsonConverter", name.to_upper_camel_case())
}

fn is_unit_enum(variants: &BTreeMap<u32, Named<VariantFormat>>) -> bool {
    variants
        .values()
        .all(|v| matches!(v.value, VariantFormat::Unit))
}

/// A C# string literal holding `value`.
fn literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The members every converter inherits from `JsonConverter<T>`, which hide a
/// type of the same name inside it.
const INHERITED: &[&str] = &[
    "CanConvert",
    "Equals",
    "Finalize",
    "GetHashCode",
    "GetType",
    "HandleNull",
    "MemberwiseClone",
    "Read",
    "ReadAsPropertyName",
    "ReferenceEquals",
    "ToString",
    "Type",
    "Write",
    "WriteAsPropertyName",
];

/// The type the reader and writer methods take, which the inherited `Type`
/// property would hide.
const SYSTEM_TYPE: &str = "global::System.Type";

// ---------------------------------------------------------------------------
// Converter
// ---------------------------------------------------------------------------

/// The `{Type}JsonConverter` of one top-level type.
struct Converter<'a> {
    config: &'a CodeGeneratorConfig,
    format: &'a ContainerFormat,
    /// The Rust name, for error messages.
    name: &'a str,
    /// The C# spelling of the type inside its converter.
    type_name: String,
    hidden: Vec<String>,
    /// The type and expression of each field `_0`, `_1`, … holding a
    /// converter, in the order the payloads are written.
    converters: Vec<(String, String)>,
}

impl<'a> Converter<'a> {
    fn new(ctx: &'a EmitContext<'a>) -> Self {
        let config = ctx.config;
        let name = ctx.container.name.name.as_str();
        let hidden: Vec<String> = INHERITED.iter().map(ToString::to_string).collect();
        let bare = name.to_upper_camel_case();
        let type_name = if hidden.contains(&bare) {
            let module = config
                .module_name()
                .split('.')
                .map(ToUpperCamelCase::to_upper_camel_case)
                .collect::<Vec<_>>()
                .join(".");
            format!("global::{module}.{bare}")
        } else {
            bare
        };
        let mut converter = Self {
            config,
            format: ctx.container.format,
            name,
            type_name,
            hidden,
            converters: vec![],
        };
        for format in payload_formats(ctx.container.format) {
            let ty = converter.render(format);
            let expr = converter.converter(format);
            converter.converters.push((ty, expr));
        }
        converter
    }

    /// `name`, one of the runtime's or `System.Text.Json`'s types, qualified
    /// when the module declares a type of the same name.
    fn qualified(&self, name: &str, namespace: &str) -> String {
        if naming::shadows(name, self.config) {
            format!("global::{namespace}.{name}")
        } else {
            name.to_string()
        }
    }

    fn facet_json(&self) -> String {
        self.qualified("FacetJson", "Facet.Runtime.Json")
    }

    fn reader(&self) -> String {
        self.qualified("Utf8JsonReader", "System.Text.Json")
    }

    fn writer(&self) -> String {
        self.qualified("Utf8JsonWriter", "System.Text.Json")
    }

    fn options(&self) -> String {
        self.qualified("JsonSerializerOptions", "System.Text.Json")
    }

    fn render(&self, format: &Format) -> String {
        render_type_hiding(format, self.config, &self.hidden)
    }

    /// A C# expression for the converter that writes a value of `format` the
    /// way `serde_json` writes the Rust value.
    fn converter(&self, format: &Format) -> String {
        let json = self.facet_json();
        let builtin = |name: &str| format!("{json}.{name}");
        match format {
            Format::Variable(_) => unreachable!("placeholders should not get this far"),
            Format::TypeName(_) => format!("{json}.Of<{}>()", self.render(format)),
            Format::Unit => builtin("Unit"),
            Format::Bool => builtin("Bool"),
            Format::I8 => builtin("I8"),
            Format::I16 => builtin("I16"),
            Format::I32 => builtin("I32"),
            Format::I64 => builtin("I64"),
            Format::I128 => builtin("I128"),
            Format::U8 => builtin("U8"),
            Format::U16 => builtin("U16"),
            Format::U32 => builtin("U32"),
            Format::U64 => builtin("U64"),
            Format::U128 => builtin("U128"),
            Format::F32 => builtin("F32"),
            Format::F64 => builtin("F64"),
            Format::Char => builtin("Char"),
            Format::Str => builtin("Str"),
            Format::Bytes => builtin("Bytes"),
            Format::Uuid => builtin("Uuid"),
            Format::Option(inner) => {
                let option = if is_value_type(inner, self.config) {
                    "Option"
                } else {
                    "OptionRef"
                };
                format!("{json}.{option}({})", self.converter(inner))
            }
            Format::Seq(inner) => format!("{json}.List({})", self.converter(inner)),
            Format::Set(inner) => format!("{json}.Set({})", self.converter(inner)),
            Format::TupleArray { content, size } => {
                format!("{json}.Array({}, {size})", self.converter(content))
            }
            Format::Map { key, value } => format!(
                "{json}.Map({}, {})",
                self.converter(key),
                self.converter(value)
            ),
            Format::Tuple(formats) => match formats.as_slice() {
                [] => builtin("Unit"),
                [format] => self.converter(format),
                _ => format!(
                    "{json}.Tuple({})",
                    formats
                        .iter()
                        .map(|f| self.converter(f))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            },
        }
    }

    /// `public override T Read(…)`, the signature of a reader method.
    fn read_signature(&self, method: &str) -> String {
        format!(
            "public override {} {method}(ref {} reader, {SYSTEM_TYPE} typeToConvert, {} options)",
            self.type_name,
            self.reader(),
            self.options()
        )
    }

    /// `public override void Write(…)`, the signature of a writer method.
    fn write_signature(&self, method: &str) -> String {
        format!(
            "public override void {method}({} writer, {} value, {} options)",
            self.writer(),
            self.type_name,
            self.options()
        )
    }

    fn write(&self, w: &mut dyn IndentWrite) -> io::Result<()> {
        let json_converter = self.qualified("JsonConverter", "System.Text.Json.Serialization");
        write!(
            w,
            "public sealed class {} : {json_converter}<{}> ",
            converter_name(self.name),
            self.type_name
        )?;
        with_block(w, Newlines::BOTH, |w| {
            for (index, (ty, expr)) in self.converters.iter().enumerate() {
                writeln!(
                    w,
                    "private static readonly {json_converter}<{ty}> _{index} = {expr};"
                )?;
            }
            if let ContainerFormat::Enum(_, tagging, _) = self.format {
                writeln!(
                    w,
                    "private static readonly {} _enum = new({});",
                    self.qualified("JsonEnum", "Facet.Runtime.Json"),
                    enum_arguments(self.name, tagging)
                )?;
            }
            if !self.converters.is_empty() || matches!(self.format, ContainerFormat::Enum(..)) {
                writeln!(w)?;
            }
            writeln!(w, "public override bool HandleNull => true;")?;
            writeln!(w)?;
            match self.format {
                ContainerFormat::UnitStruct(_) => self.write_unit_struct(w),
                ContainerFormat::NewTypeStruct(..) => self.write_newtype_struct(w),
                ContainerFormat::TupleStruct(formats, _) => self.write_tuple_struct(w, formats),
                ContainerFormat::Struct(fields, _) => self.write_struct(w, fields),
                ContainerFormat::Enum(variants, tagging, _) => {
                    self.write_enum(w, variants, tagging)
                }
            }
        })
    }

    /// Writes a method whose signature is `signature` and body `body`.
    fn method(
        w: &mut dyn IndentWrite,
        signature: &str,
        body: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>,
    ) -> io::Result<()> {
        writeln!(w, "{signature}")?;
        with_block(w, Newlines::BOTH, body)
    }

    /// Rust writes a unit struct as `null`. The registry also records a
    /// braced struct with no fields as a unit struct, which Rust writes as
    /// `{}`, so that reads too.
    fn write_unit_struct(&self, w: &mut dyn IndentWrite) -> io::Result<()> {
        let json = self.facet_json();
        Self::method(w, &self.read_signature("Read"), |w| {
            writeln!(w, "{json}.ReadUnit(ref reader, {});", literal(self.name))?;
            writeln!(w, "return new {}();", self.type_name)
        })?;
        writeln!(w)?;
        Self::method(w, &self.write_signature("Write"), |w| {
            writeln!(w, "writer.WriteNullValue();")
        })
    }

    /// Rust writes a newtype struct as the value it wraps, and a newtype of a
    /// map key as that key.
    fn write_newtype_struct(&self, w: &mut dyn IndentWrite) -> io::Result<()> {
        let json = self.facet_json();
        let ty = &self.type_name;
        Self::method(w, &self.read_signature("Read"), |w| {
            writeln!(
                w,
                "return new {ty} {{ Value = {json}.Read(_0, ref reader, options) }};"
            )
        })?;
        writeln!(w)?;
        Self::method(w, &self.write_signature("Write"), |w| {
            writeln!(w, "_0.Write(writer, value.Value, options);")
        })?;
        writeln!(w)?;
        Self::method(w, &self.read_signature("ReadAsPropertyName"), |w| {
            writeln!(
                w,
                "return new {ty} {{ Value = {json}.ReadKey(_0, ref reader, options) }};"
            )
        })?;
        writeln!(w)?;
        Self::method(w, &self.write_signature("WriteAsPropertyName"), |w| {
            writeln!(w, "_0.WriteAsPropertyName(writer, value.Value, options);")
        })
    }

    /// Rust writes a tuple struct as an array.
    fn write_tuple_struct(&self, w: &mut dyn IndentWrite, formats: &[Format]) -> io::Result<()> {
        Self::method(w, &self.read_signature("Read"), |w| {
            self.read_elements(w, 0, formats.len(), self.name)?;
            let properties = (0..formats.len())
                .map(|i| format!("Field{i} = e{i}"))
                .collect::<Vec<_>>();
            writeln!(
                w,
                "return new {} {{ {} }};",
                self.type_name,
                properties.join(", ")
            )
        })?;
        writeln!(w)?;
        Self::method(w, &self.write_signature("Write"), |w| {
            write_elements(w, "writer", 0, formats.len(), "value")
        })
    }

    /// A struct with named fields is a JSON object keyed by the fields' wire
    /// names.
    fn write_struct(&self, w: &mut dyn IndentWrite, fields: &[Named<Format>]) -> io::Result<()> {
        Self::method(w, &self.read_signature("Read"), |w| {
            let values = self.read_fields(w, 0, fields, self.name)?;
            let properties = fields
                .iter()
                .zip(values)
                .map(|(field, value)| format!("{} = {value},", field.name.to_upper_camel_case()))
                .collect::<Vec<_>>();
            if properties.is_empty() {
                return writeln!(w, "return new {}();", self.type_name);
            }
            writeln!(w, "return new {}", self.type_name)?;
            with_block(w, Newlines::OPEN, |w| {
                for property in &properties {
                    writeln!(w, "{property}")?;
                }
                Ok(())
            })?;
            writeln!(w, ";")
        })?;
        writeln!(w)?;
        Self::method(w, &self.write_signature("Write"), |w| {
            writeln!(w, "writer.WriteStartObject();")?;
            self.write_fields(w, "writer", 0, fields, "value")?;
            writeln!(w, "writer.WriteEndObject();")
        })
    }

    /// Reads `count` array elements into `e0`, `e1`, … with the converters
    /// from `_{first}`.
    fn read_elements(
        &self,
        w: &mut dyn IndentWrite,
        first: usize,
        count: usize,
        label: &str,
    ) -> io::Result<()> {
        let json = self.facet_json();
        let label = literal(label);
        writeln!(w, "{json}.StartArray(ref reader, {label});")?;
        for i in 0..count {
            writeln!(
                w,
                "var e{i} = {json}.Element(_{}, ref reader, options, {label});",
                first + i
            )?;
        }
        writeln!(w, "{json}.EndArray(ref reader, {label});")
    }

    /// Reads the object `fields` with the converters from `_{first}`, and
    /// returns the expressions holding their values.
    fn read_fields(
        &self,
        w: &mut dyn IndentWrite,
        first: usize,
        fields: &[Named<Format>],
        label: &str,
    ) -> io::Result<Vec<String>> {
        let json = self.facet_json();
        let label = literal(label);
        writeln!(w, "{json}.StartObject(ref reader, {label});")?;
        for (i, field) in fields.iter().enumerate() {
            let ty = self.render(&field.value);
            if is_optional(&field.value) {
                writeln!(w, "{ty} f{i} = default;")?;
            } else {
                writeln!(w, "{ty} f{i} = default!;")?;
                writeln!(w, "var has{i} = false;")?;
            }
        }
        writeln!(w, "while ({json}.NextField(ref reader, out var key))")?;
        with_block(w, Newlines::BOTH, |w| {
            if fields.is_empty() {
                return writeln!(w, "reader.Skip();");
            }
            writeln!(w, "switch (key)")?;
            with_block(w, Newlines::BOTH, |w| {
                for (i, field) in fields.iter().enumerate() {
                    writeln!(w, "case {}:", literal(&field.name))?;
                    w.indent();
                    writeln!(
                        w,
                        "f{i} = {json}.Read(_{}, ref reader, options);",
                        first + i
                    )?;
                    if !is_optional(&field.value) {
                        writeln!(w, "has{i} = true;")?;
                    }
                    writeln!(w, "break;")?;
                    w.unindent();
                }
                writeln!(w, "default:")?;
                w.indent();
                writeln!(w, "reader.Skip();")?;
                writeln!(w, "break;")?;
                w.unindent();
                Ok(())
            })
        })?;
        Ok(fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                if is_optional(&field.value) {
                    format!("f{i}")
                } else {
                    format!(
                        "{json}.Required(has{i}, f{i}, {}, {label})",
                        literal(&field.name)
                    )
                }
            })
            .collect())
    }

    /// Writes `fields` of `value` as the fields of an object, with the
    /// converters from `_{first}`.
    fn write_fields(
        &self,
        w: &mut dyn IndentWrite,
        writer: &str,
        first: usize,
        fields: &[Named<Format>],
        value: &str,
    ) -> io::Result<()> {
        let json = self.facet_json();
        for (i, field) in fields.iter().enumerate() {
            writeln!(
                w,
                "{json}.WriteField({writer}, {}, _{}, {value}.{}, options);",
                literal(&field.name),
                first + i,
                field.name.to_upper_camel_case()
            )?;
        }
        Ok(())
    }

    /// An enum is written as its variant, tagged as the registry records:
    /// externally — `"Unit"`, `{"NewType": …}`, `{"Tuple": […]}`,
    /// `{"Struct": {…}}` — or internally or adjacently, per
    /// `#[facet(tag, content)]`.
    fn write_enum(
        &self,
        w: &mut dyn IndentWrite,
        variants: &BTreeMap<u32, Named<VariantFormat>>,
        tagging: &EnumTagging,
    ) -> io::Result<()> {
        let unit_enum = is_unit_enum(variants);
        let ty = &self.type_name;
        // The first converter field of each variant's payload.
        let mut firsts = Vec::with_capacity(variants.len());
        let mut next = 0;
        for variant in variants.values() {
            firsts.push(next);
            next += variant_formats(&variant.value).len();
        }

        Self::method(w, &self.read_signature("Read"), |w| {
            writeln!(w, "return _enum.Read(ref reader, options, _readVariant);")
        })?;
        writeln!(w)?;
        Self::method(w, &self.write_signature("Write"), |w| {
            writeln!(w, "switch (value)")?;
            with_block(w, Newlines::BOTH, |w| {
                for (variant, first) in variants.values().zip(&firsts) {
                    self.write_variant(w, variant, *first, unit_enum)?;
                }
                writeln!(w, "default:")?;
                w.indent();
                writeln!(
                    w,
                    "throw new global::System.ArgumentOutOfRangeException(nameof(value));"
                )?;
                w.unindent();
                Ok(())
            })
        })?;

        if unit_enum && matches!(tagging, EnumTagging::External) {
            // A unit variant can be a map key, which Rust writes as its name.
            writeln!(w)?;
            Self::method(w, &self.read_signature("ReadAsPropertyName"), |w| {
                writeln!(w, "var variant = reader.GetString()!;")?;
                writeln!(w, "switch (variant)")?;
                with_block(w, Newlines::BOTH, |w| {
                    for variant in variants.values() {
                        writeln!(w, "case {}:", literal(&variant.name))?;
                        w.indent();
                        writeln!(w, "return {ty}.{};", variant.name.to_upper_camel_case())?;
                        w.unindent();
                    }
                    writeln!(w, "default:")?;
                    w.indent();
                    writeln!(w, "throw _enum.Unknown(variant);")?;
                    w.unindent();
                    Ok(())
                })
            })?;
            writeln!(w)?;
            Self::method(w, &self.write_signature("WriteAsPropertyName"), |w| {
                writeln!(w, "switch (value)")?;
                with_block(w, Newlines::BOTH, |w| {
                    for variant in variants.values() {
                        writeln!(w, "case {ty}.{}:", variant.name.to_upper_camel_case())?;
                        w.indent();
                        writeln!(w, "writer.WritePropertyName({});", literal(&variant.name))?;
                        writeln!(w, "break;")?;
                        w.unindent();
                    }
                    writeln!(w, "default:")?;
                    w.indent();
                    writeln!(
                        w,
                        "throw new global::System.ArgumentOutOfRangeException(nameof(value));"
                    )?;
                    w.unindent();
                    Ok(())
                })
            })?;
        }

        writeln!(w)?;
        let signature = format!(
            "private static {ty} _readVariant(string variant, bool hasPayload, ref {} reader, {} options)",
            self.reader(),
            self.options()
        );
        Self::method(w, &signature, |w| {
            writeln!(w, "switch (variant)")?;
            with_block(w, Newlines::BOTH, |w| {
                for (variant, first) in variants.values().zip(&firsts) {
                    self.read_variant(w, variant, *first, unit_enum)?;
                }
                writeln!(w, "default:")?;
                w.indent();
                writeln!(w, "throw _enum.Unknown(variant);")?;
                w.unindent();
                Ok(())
            })
        })
    }

    /// The `case` of `Write` that writes `variant`.
    fn write_variant(
        &self,
        w: &mut dyn IndentWrite,
        variant: &Named<VariantFormat>,
        first: usize,
        unit_enum: bool,
    ) -> io::Result<()> {
        let ty = &self.type_name;
        let csharp = variant.name.to_upper_camel_case();
        let wire = literal(&variant.name);
        match &variant.value {
            VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
            VariantFormat::Unit => {
                writeln!(w, "case {ty}.{csharp}:")?;
                w.indent();
                writeln!(w, "_enum.WriteVariant(writer, {wire});")?;
            }
            VariantFormat::NewType(_) => {
                writeln!(w, "case {ty}.{csharp} v:")?;
                w.indent();
                writeln!(
                    w,
                    "_enum.WriteVariant(writer, {wire}, w => _{first}.Write(w, v.Value, options));"
                )?;
            }
            VariantFormat::Tuple(formats) => {
                writeln!(w, "case {ty}.{csharp} v:")?;
                w.indent();
                write!(w, "_enum.WriteVariant(writer, {wire}, w =>")?;
                writeln!(w)?;
                with_block(w, Newlines::OPEN, |w| {
                    write_elements(w, "w", first, formats.len(), "v")
                })?;
                writeln!(w, ");")?;
            }
            VariantFormat::Struct(fields) => {
                writeln!(w, "case {ty}.{csharp} v:")?;
                w.indent();
                writeln!(w, "_enum.WriteStructVariant(writer, {wire}, w =>")?;
                with_block(w, Newlines::OPEN, |w| {
                    self.write_fields(w, "w", first, fields, "v")
                })?;
                writeln!(w, ");")?;
            }
        }
        debug_assert!(!unit_enum || matches!(variant.value, VariantFormat::Unit));
        writeln!(w, "break;")?;
        w.unindent();
        Ok(())
    }

    /// The `case` of `_readVariant` that reads `variant`.
    fn read_variant(
        &self,
        w: &mut dyn IndentWrite,
        variant: &Named<VariantFormat>,
        first: usize,
        unit_enum: bool,
    ) -> io::Result<()> {
        let json = self.facet_json();
        let ty = &self.type_name;
        let csharp = variant.name.to_upper_camel_case();
        let wire = literal(&variant.name);
        let label = format!("{}.{}", self.name, variant.name);
        writeln!(w, "case {wire}:")?;
        match &variant.value {
            VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
            VariantFormat::Unit => {
                w.indent();
                writeln!(w, "_enum.Unit(hasPayload, ref reader, {wire});")?;
                if unit_enum {
                    writeln!(w, "return {ty}.{csharp};")?;
                } else {
                    writeln!(w, "return new {ty}.{csharp}();")?;
                }
                w.unindent();
            }
            VariantFormat::NewType(_) => {
                w.indent();
                writeln!(w, "_enum.Payload(hasPayload, {wire});")?;
                writeln!(
                    w,
                    "return new {ty}.{csharp}({json}.Read(_{first}, ref reader, options));"
                )?;
                w.unindent();
            }
            VariantFormat::Tuple(formats) => {
                with_block(w, Newlines::BOTH, |w| {
                    writeln!(w, "_enum.Payload(hasPayload, {wire});")?;
                    self.read_elements(w, first, formats.len(), &label)?;
                    let elements = (0..formats.len())
                        .map(|i| format!("e{i}"))
                        .collect::<Vec<_>>();
                    writeln!(w, "return new {ty}.{csharp}({});", elements.join(", "))
                })?;
            }
            VariantFormat::Struct(fields) => {
                with_block(w, Newlines::BOTH, |w| {
                    writeln!(w, "_enum.Payload(hasPayload, {wire});")?;
                    let values = self.read_fields(w, first, fields, &label)?;
                    if values.is_empty() {
                        return writeln!(w, "return new {ty}.{csharp}();");
                    }
                    writeln!(w, "return new {ty}.{csharp}(")?;
                    w.indent();
                    let last = values.len() - 1;
                    for (i, value) in values.iter().enumerate() {
                        let separator = if i == last { ");" } else { "," };
                        writeln!(w, "{value}{separator}")?;
                    }
                    w.unindent();
                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}

/// Writes `count` elements of `value`, its `Field0`, `Field1`, … (`Value`
/// when it holds one), as an array, with the converters from `_{first}`.
fn write_elements(
    w: &mut dyn IndentWrite,
    writer: &str,
    first: usize,
    count: usize,
    value: &str,
) -> io::Result<()> {
    writeln!(w, "{writer}.WriteStartArray();")?;
    for i in 0..count {
        writeln!(
            w,
            "_{}.Write({writer}, {value}.Field{i}, options);",
            first + i
        )?;
    }
    writeln!(w, "{writer}.WriteEndArray();")
}

/// The arguments constructing an enum's `JsonEnum`, with its tagging.
fn enum_arguments(name: &str, tagging: &EnumTagging) -> String {
    match tagging {
        EnumTagging::External => literal(name),
        EnumTagging::Internal { tag } => format!("{}, tag: {}", literal(name), literal(tag)),
        EnumTagging::Adjacent { tag, content } => format!(
            "{}, tag: {}, content: {}",
            literal(name),
            literal(tag),
            literal(content)
        ),
    }
}

/// Whether a field of this format is optional: Rust reads a missing key as
/// `None`.
const fn is_optional(format: &Format) -> bool {
    matches!(format, Format::Option(_))
}

/// The formats a variant's payload holds, in order.
fn variant_formats(variant: &VariantFormat) -> Vec<&Format> {
    match variant {
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
        VariantFormat::Unit => vec![],
        VariantFormat::NewType(format) => vec![format],
        VariantFormat::Tuple(formats) => formats.iter().collect(),
        VariantFormat::Struct(fields) => fields.iter().map(|f| &f.value).collect(),
    }
}

/// The formats a container holds, in the order its converter's fields `_0`,
/// `_1`, … are numbered.
fn payload_formats(format: &ContainerFormat) -> Vec<&Format> {
    match format {
        ContainerFormat::UnitStruct(_) => vec![],
        ContainerFormat::NewTypeStruct(format, _) => vec![format],
        ContainerFormat::TupleStruct(formats, _) => formats.iter().collect(),
        ContainerFormat::Struct(fields, _) => fields.iter().map(|f| &f.value).collect(),
        ContainerFormat::Enum(variants, _, _) => variants
            .values()
            .flat_map(|v| variant_formats(&v.value))
            .collect(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{
        CodeGeneratorConfig, Container,
        indent::{IndentConfig, IndentedWriter},
        plugin::EmitContext,
    };
    use crate::reflection::format::{
        ContainerFormat, Doc, EnumTagging, Format, Named, QualifiedTypeName,
    };

    fn render(f: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>) -> String {
        let mut buf = Vec::new();
        let mut w = IndentedWriter::new(&mut buf, IndentConfig::Space(4));
        f(&mut w).unwrap();
        String::from_utf8(buf).unwrap()
    }

    // -------------------------------------------------------------------------
    // imports
    // -------------------------------------------------------------------------

    #[test]
    fn imports_returns_json_usings() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;
        let cfg = CodeGeneratorConfig::new("test".to_string());
        let imports = plugin.imports(&cfg);
        assert!(imports.iter().any(|i| i.contains("Facet.Runtime.Json")));
        assert!(
            imports
                .iter()
                .any(|i| i.contains("Text.Json.Serialization"))
        );
    }

    // -------------------------------------------------------------------------
    // type_annotations
    // -------------------------------------------------------------------------

    #[test]
    fn type_annotations_unit_enum() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = std::collections::BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "Alpha".to_string()));
        variants.insert(1u32, Named::new(&VariantFormat::Unit, "Beta".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, EnumTagging::External, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert_eq!(
            plugin.type_annotations(&ctx),
            ["[JsonConverter(typeof(MyEnumJsonConverter))]"]
        );
    }

    #[test]
    fn type_annotations_variant_hierarchy() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = std::collections::BTreeMap::new();
        variants.insert(
            0u32,
            Named::new(
                &VariantFormat::NewType(Box::new(Format::Str)),
                "Ok".to_string(),
            ),
        );
        variants.insert(
            1u32,
            Named::new(
                &VariantFormat::NewType(Box::new(Format::I32)),
                "Err".to_string(),
            ),
        );

        let name = QualifiedTypeName::root("Result".to_string());
        let format = ContainerFormat::Enum(variants, EnumTagging::External, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert_eq!(
            plugin.type_annotations(&ctx),
            ["[JsonConverter(typeof(ResultJsonConverter))]"]
        );
    }

    #[test]
    fn type_annotations_struct_names_its_converter() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert_eq!(
            plugin.type_annotations(&ctx),
            ["[JsonConverter(typeof(FooJsonConverter))]"]
        );
    }

    // -------------------------------------------------------------------------
    // field_annotations
    // -------------------------------------------------------------------------

    #[test]
    fn field_annotations_returns_json_property_name() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let field = Named::new(&Format::Str, "firstName".to_string());

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        let annotations = plugin.field_annotations(&field, &ctx);
        assert_eq!(annotations, ["[property: JsonPropertyName(\"firstName\")]"]);
    }

    #[test]
    fn field_annotations_keep_the_wire_name() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let field = Named::new(&Format::I32, "with-\"dash\"".to_string());

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        let annotations = plugin.field_annotations(&field, &ctx);
        assert_eq!(
            annotations,
            [r#"[property: JsonPropertyName("with-\"dash\"")]"#]
        );
    }

    // -------------------------------------------------------------------------
    // has_type_body
    // -------------------------------------------------------------------------

    #[test]
    fn has_type_body_true_for_struct() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert!(plugin.has_type_body(&ctx));
    }

    #[test]
    fn has_type_body_false_for_unit_enum() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let mut variants = std::collections::BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "A".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, EnumTagging::External, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        assert!(!plugin.has_type_body(&ctx));
    }

    // -------------------------------------------------------------------------
    // type_body
    // -------------------------------------------------------------------------

    #[test]
    fn type_body_emits_json_serialize_and_deserialize() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<CSharp>;

        let name = QualifiedTypeName::root("MyRecord".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let config = CodeGeneratorConfig::new("test".to_string());
        let ctx = EmitContext::top_level(&container, &config);

        let out = render(|w| plugin.type_body(w, &ctx));
        assert!(out.contains("public string JsonSerialize()"), "{out}");
        assert!(out.contains("return JsonSerde.Serialize(this);"), "{out}");
        assert!(
            out.contains("public static MyRecord JsonDeserialize(string input)"),
            "{out}"
        );
        assert!(
            out.contains("return JsonSerde.Deserialize<MyRecord>(input);"),
            "{out}"
        );
    }
}
