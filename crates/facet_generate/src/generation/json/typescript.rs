//! `EmitterPlugin<TypeScript>` implementation for the [`JsonPlugin`].
//!
//! Every generated type converts to and from the plain JSON value
//! `serde_json` gives the same Rust type, and `JSON.stringify` /
//! `JSON.parse` do the text — so JSON written on either side reads on the
//! other.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `import * as $json from "./serde/json";` |
//! | `module_helpers` | the `Uuid` alias |
//! | `type_body` | static `toJson` / `fromJson` and `jsonSerialize` / `jsonDeserialize` on a class |
//! | `after_type` | `toJson{Enum}` / `fromJson{Enum}` and `jsonSerialize{Enum}` / `jsonDeserialize{Enum}` beside an enum |
//! | `runtime_files` | `serde/json.ts` |
//!
//! A class's methods are static, so that the plain object an internally
//! tagged variant spreads a struct into (`{ type: "V", ...point }`) is
//! written the same way as the struct itself.
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
//! The generated code reaches the runtime only through the `$json` import,
//! which no generated name can shadow, and names no global itself.

use std::io;

use heck::ToUpperCamelCase;

use crate::generation::{
    CodeGeneratorConfig, Feature, PackageLocation, SERDE_NAMESPACE,
    indent::{IndentWrite, IndentedWriter, Newlines, with_block},
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
    typescript::{TypeScript, naming, render_type},
};
use crate::reflection::format::{
    ContainerFormat, EnumTagging, Format, Named, QualifiedTypeName, VariantFormat,
};

use super::JsonPlugin;

/// The binding the generated code imports the runtime under.
const JSON: &str = "$json";

const FEATURE_UUID: &str = "export type Uuid = string & { readonly __uuid: unique symbol };\n";

// ---------------------------------------------------------------------------
// EmitterPlugin implementation
// ---------------------------------------------------------------------------

impl EmitterPlugin<TypeScript> for JsonPlugin {
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let serde = config.external_packages.get(SERDE_NAMESPACE).map_or_else(
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
        );
        vec![format!(r#"import * as {JSON} from "{serde}/json";"#)]
    }

    fn runtime_files(&self) -> Vec<RuntimeFile> {
        vec![RuntimeFile {
            relative_path: "serde/json.ts".to_string(),
            contents: include_bytes!("../../../runtime/typescript-json/serde/json.ts").to_vec(),
        }]
    }

    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        if config.features.contains(&Feature::Uuid) {
            writeln!(w)?;
            write!(w, "{FEATURE_UUID}")?;
        }
        Ok(())
    }

    fn has_type_body(&self, _ctx: &EmitContext) -> bool {
        true
    }

    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        let codec = Codec { config: ctx.config };
        match ctx.container.format {
            ContainerFormat::Enum(..) => Ok(()),
            format => codec.write_class_methods(w, ctx.name(), format),
        }
    }

    fn after_type(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        if let ContainerFormat::Enum(variants, tagging, _) = ctx.container.format {
            let codec = Codec { config: ctx.config };
            let variants: Vec<_> = variants.values().collect();
            codec.write_enum_functions(w, ctx.name(), &variants, tagging)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Literals and property access
// ---------------------------------------------------------------------------

/// `s` as a JavaScript string literal.
fn literal(s: &str) -> String {
    serde_json::to_string(s).expect("a string always serializes")
}

/// The key of an object-literal entry holding the JSON field `name`.
///
/// Quoted, as it is a wire name; `__proto__` is computed, since the plain
/// key would set the object's prototype instead.
fn json_key(name: &str) -> String {
    if name == "__proto__" {
        format!("[{}]", literal(name))
    } else {
        literal(name)
    }
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

/// Writes the expressions converting TypeScript values to and from JSON.
struct Codec<'a> {
    config: &'a CodeGeneratorConfig,
}

impl Codec<'_> {
    /// `Type` or `Namespace.Type`, for a struct.
    fn class(name: &QualifiedTypeName) -> String {
        name.format(ToUpperCamelCase::to_upper_camel_case, ".")
    }

    fn render(&self, format: &Format) -> String {
        render_type(format, self.config)
    }

    /// Whether a value of `format` is its own JSON value.
    fn is_plain(format: &Format) -> bool {
        match format {
            Format::Bool
            | Format::I8
            | Format::I16
            | Format::I32
            | Format::U8
            | Format::U16
            | Format::U32
            | Format::Char
            | Format::Str
            | Format::Uuid => true,
            Format::Option(inner) | Format::Seq(inner) | Format::Set(inner) => {
                Self::is_plain(inner)
            }
            _ => false,
        }
    }

    /// The expression converting `expr`, a value of `format`, to JSON.
    fn write_json(&self, format: &Format, expr: &str, depth: usize) -> String {
        match format {
            f if Self::is_plain(f) => expr.to_string(),
            Format::TypeName(name) if self.config.is_enum(name) => {
                format!("{}({expr})", naming::enum_function("toJson", name))
            }
            Format::TypeName(name) => format!("{}.toJson({expr})", Self::class(name)),
            Format::Unit => "null".to_string(),
            Format::I64 | Format::I128 | Format::U64 | Format::U128 => {
                format!("{JSON}.writeBigInt({expr})")
            }
            Format::F32 | Format::F64 => format!("{JSON}.writeFloat({expr})"),
            Format::Bytes => format!("{JSON}.writeBytes({expr})"),
            Format::Option(inner) => format!(
                "({expr} === null ? null : {})",
                self.write_json(inner, expr, depth)
            ),
            Format::Seq(inner) | Format::Set(inner) => {
                let v = format!("v{depth}");
                format!(
                    "{expr}.map(({v}) => {})",
                    self.write_json(inner, &v, depth + 1)
                )
            }
            Format::Map { key, value } => {
                let (k, v) = (format!("k{depth}"), format!("v{depth}"));
                format!(
                    "{JSON}.writeMap({expr}, ({k}) => {}, ({v}) => {})",
                    self.write_key(key, &k, depth + 1),
                    self.write_json(value, &v, depth + 1)
                )
            }
            Format::Tuple(formats) => self.write_json_array(
                formats
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (f, format!("{expr}[{i}]"))),
                depth,
            ),
            Format::TupleArray { content, .. } => {
                let v = format!("v{depth}");
                format!(
                    "{expr}.map(({v}) => {})",
                    self.write_json(content, &format!("{v}[0]"), depth + 1)
                )
            }
            Format::Variable(_) => panic!("unexpected variable in a JSON format"),
            _ => unreachable!("plain formats are handled above"),
        }
    }

    /// `[…]`, the JSON of each `(format, expr)` element.
    fn write_json_array<'f>(
        &self,
        elements: impl Iterator<Item = (&'f Format, String)>,
        depth: usize,
    ) -> String {
        let items: Vec<String> = elements
            .map(|(f, expr)| self.write_json(f, &expr, depth))
            .collect();
        format!("[{}]", items.join(", "))
    }

    /// The expression for the object key of `expr`, a map key of `format`.
    fn write_key(&self, format: &Format, expr: &str, depth: usize) -> String {
        match format {
            Format::Str | Format::Char | Format::Uuid => expr.to_string(),
            Format::Bool
            | Format::I8
            | Format::I16
            | Format::I32
            | Format::I64
            | Format::I128
            | Format::U8
            | Format::U16
            | Format::U32
            | Format::U64
            | Format::U128 => format!("`${{{expr}}}`"),
            _ => format!("{JSON}.writeKey({})", self.write_json(format, expr, depth)),
        }
    }

    /// The runtime reader of a leaf format, if it has one.
    const fn leaf_reader(format: &Format) -> Option<&'static str> {
        Some(match format {
            Format::Unit => "readUnit",
            Format::Bool => "readBool",
            Format::I8 => "readI8",
            Format::I16 => "readI16",
            Format::I32 => "readI32",
            Format::I64 => "readI64",
            Format::I128 => "readI128",
            Format::U8 => "readU8",
            Format::U16 => "readU16",
            Format::U32 => "readU32",
            Format::U64 => "readU64",
            Format::U128 => "readU128",
            Format::F32 => "readF32",
            Format::F64 => "readF64",
            Format::Char => "readChar",
            Format::Str => "readStr",
            Format::Bytes => "readBytes",
            _ => return None,
        })
    }

    /// The expression reading a value of `format` from `expr`, a JSON value.
    fn read_json(&self, format: &Format, expr: &str, depth: usize) -> String {
        if let Some(reader) = Self::leaf_reader(format) {
            return format!("{JSON}.{reader}({expr})");
        }
        match format {
            Format::TypeName(name) if self.config.is_enum(name) => {
                format!("{}({expr})", naming::enum_function("fromJson", name))
            }
            Format::TypeName(name) => format!("{}.fromJson({expr})", Self::class(name)),
            Format::Uuid => format!("{JSON}.readUuid({expr}) as Uuid"),
            Format::Option(inner) => {
                format!("{JSON}.readOption({expr}, {})", self.reader(inner, depth))
            }
            Format::Seq(inner) | Format::Set(inner) => {
                format!("{JSON}.readSeq({expr}, {})", self.reader(inner, depth))
            }
            Format::Map { key, value } => {
                let k = format!("k{depth}");
                format!(
                    "{JSON}.readMap({expr}, ({k}) => {}, {})",
                    self.read_key(key, &k, depth + 1),
                    self.reader(value, depth)
                )
            }
            Format::Tuple(formats) => self.read_tuple(formats, expr, depth),
            Format::TupleArray { content, size } => {
                let j = format!("j{depth}");
                format!(
                    "{JSON}.readSeq({expr}, ({j}): [{}] => [{}], {size})",
                    self.render(content),
                    self.read_json(content, &j, depth + 1)
                )
            }
            Format::Variable(_) => panic!("unexpected variable in a JSON format"),
            _ => unreachable!("leaf formats are handled above"),
        }
    }

    /// `$json.readTuple<[…]>(expr, […])`: an array of exactly one element of
    /// each of `formats`.
    fn read_tuple(&self, formats: &[Format], expr: &str, depth: usize) -> String {
        let types: Vec<String> = formats.iter().map(|f| self.render(f)).collect();
        let readers: Vec<String> = formats.iter().map(|f| self.reader(f, depth)).collect();
        format!(
            "{JSON}.readTuple<[{}]>({expr}, [{}])",
            types.join(", "),
            readers.join(", ")
        )
    }

    /// A function reading a value of `format` from its JSON value.
    fn reader(&self, format: &Format, depth: usize) -> String {
        if let Some(reader) = Self::leaf_reader(format) {
            return format!("{JSON}.{reader}");
        }
        match format {
            Format::TypeName(name) if self.config.is_enum(name) => {
                naming::enum_function("fromJson", name)
            }
            Format::TypeName(name) => format!("{}.fromJson", Self::class(name)),
            _ => {
                let j = format!("j{depth}");
                format!("({j}) => {}", self.read_json(format, &j, depth + 1))
            }
        }
    }

    /// The expression reading a map key of `format` from `expr`, an object
    /// key.
    fn read_key(&self, format: &Format, expr: &str, depth: usize) -> String {
        match format {
            Format::Str => expr.to_string(),
            Format::Char | Format::Uuid => self.read_json(format, expr, depth),
            Format::Bool
            | Format::I8
            | Format::I16
            | Format::I32
            | Format::I64
            | Format::I128
            | Format::U8
            | Format::U16
            | Format::U32
            | Format::U64
            | Format::U128 => self.read_json(format, &format!("{JSON}.keyLiteral({expr})"), depth),
            _ => format!("{JSON}.readKey({expr}, {})", self.reader(format, depth)),
        }
    }

    // -----------------------------------------------------------------------
    // Structs
    // -----------------------------------------------------------------------

    /// The `toJson` / `fromJson` and `jsonSerialize` / `jsonDeserialize`
    /// static methods of the class `name`.
    fn write_class_methods(
        &self,
        w: &mut dyn IndentWrite,
        name: &str,
        format: &ContainerFormat,
    ) -> io::Result<()> {
        writeln!(w)?;
        write!(w, "static toJson(value: {name}): {JSON}.JsonValue ")?;
        with_block(w, Newlines::BOTH, |w| match format {
            ContainerFormat::UnitStruct(_) => writeln!(w, "return null;"),
            ContainerFormat::NewTypeStruct(inner, _) => {
                writeln!(w, "return {};", self.write_json(inner, "value.value", 0))
            }
            ContainerFormat::TupleStruct(formats, _) => {
                let array = self.write_json_array(
                    formats
                        .iter()
                        .enumerate()
                        .map(|(i, f)| (f, format!("value.field{i}"))),
                    0,
                );
                writeln!(w, "return {array};")
            }
            ContainerFormat::Struct(fields, _) => {
                write!(w, "return ")?;
                self.write_object(w, &[], fields, "value")?;
                writeln!(w, ";")
            }
            ContainerFormat::Enum(..) => unreachable!("an enum is not a class"),
        })?;

        writeln!(w)?;
        write!(w, "static fromJson(json: unknown): {name} ")?;
        with_block(w, Newlines::BOTH, |w| match format {
            ContainerFormat::UnitStruct(_) => {
                writeln!(w, "{JSON}.readUnitStruct(json, {});", literal(name))?;
                writeln!(w, "return new {name}();")
            }
            ContainerFormat::NewTypeStruct(inner, _) => {
                writeln!(
                    w,
                    "return new {name}({});",
                    self.read_json(inner, "json", 0)
                )
            }
            ContainerFormat::TupleStruct(formats, _) => {
                writeln!(
                    w,
                    "return new {name}(...{});",
                    self.read_tuple(formats, "json", 0)
                )
            }
            ContainerFormat::Struct(fields, _) => {
                writeln!(w, "const obj = {JSON}.readObject(json, {});", literal(name))?;
                self.write_new(w, name, fields, "obj")
            }
            ContainerFormat::Enum(..) => unreachable!("an enum is not a class"),
        })?;

        writeln!(w)?;
        write!(w, "static jsonSerialize(value: {name}): string ")?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "return {JSON}.stringify({name}.toJson(value));")
        })?;

        writeln!(w)?;
        write!(w, "static jsonDeserialize(text: string): {name} ")?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "return {name}.fromJson({JSON}.parse(text));")
        })?;
        Ok(())
    }

    /// An object literal: the `(key, expression)` entries of `prefix`, then
    /// each of `fields` of `owner` under its wire name.
    fn write_object(
        &self,
        w: &mut dyn IndentWrite,
        prefix: &[(&str, String)],
        fields: &[Named<Format>],
        owner: &str,
    ) -> io::Result<()> {
        if prefix.is_empty() && fields.is_empty() {
            return write!(w, "{{}}");
        }
        writeln!(w, "{{")?;
        w.indent();
        for (key, expr) in prefix {
            writeln!(w, "{}: {expr},", json_key(key))?;
        }
        for field in fields {
            let expr = naming::member(owner, &field.name);
            writeln!(
                w,
                "{}: {},",
                json_key(&field.name),
                self.write_json(&field.value, &expr, 0)
            )?;
        }
        w.unindent();
        write!(w, "}}")
    }

    /// `return new name(…)`, reading each of `fields` from the object `obj`.
    fn write_new(
        &self,
        w: &mut dyn IndentWrite,
        name: &str,
        fields: &[Named<Format>],
        obj: &str,
    ) -> io::Result<()> {
        if fields.is_empty() {
            return writeln!(w, "return new {name}();");
        }
        writeln!(w, "return new {name}(")?;
        w.indent();
        for field in fields {
            writeln!(w, "{},", self.read_field(obj, field))?;
        }
        w.unindent();
        writeln!(w, ");")
    }

    /// The expression reading `field` from the object `obj`.
    fn read_field(&self, obj: &str, field: &Named<Format>) -> String {
        self.read_json(
            &field.value,
            &format!("{JSON}.field({obj}, {})", literal(&field.name)),
            0,
        )
    }

    // -----------------------------------------------------------------------
    // Enums
    // -----------------------------------------------------------------------

    /// `toJson{Enum}` / `fromJson{Enum}` and `jsonSerialize{Enum}` /
    /// `jsonDeserialize{Enum}`.
    fn write_enum_functions(
        &self,
        w: &mut dyn IndentWrite,
        name: &str,
        variants: &[&Named<VariantFormat>],
        tagging: &EnumTagging,
    ) -> io::Result<()> {
        let tag_field = match tagging {
            EnumTagging::External => "kind",
            EnumTagging::Internal { tag } | EnumTagging::Adjacent { tag, .. } => tag.as_str(),
        };
        let what = literal(name);

        writeln!(w)?;
        write!(
            w,
            "export function toJson{name}(value: {name}): {JSON}.JsonValue "
        )?;
        with_block(w, Newlines::BOTH, |w| {
            if variants.is_empty() {
                return writeln!(w, "throw {JSON}.unknownVariant({what}, value);");
            }
            write!(w, "switch ({}) ", naming::member("value", tag_field))?;
            with_block(w, Newlines::BOTH, |w| {
                for variant in variants {
                    write!(w, "case {}: ", literal(&variant.name))?;
                    self.write_variant_to_json(w, variant, tagging)?;
                }
                writeln!(w, "default: throw {JSON}.unknownVariant({what}, value);")
            })
        })?;

        writeln!(w)?;
        write!(w, "export function fromJson{name}(json: unknown): {name} ")?;
        with_block(w, Newlines::BOTH, |w| {
            let units: Vec<String> = variants
                .iter()
                .filter(|v| matches!(v.value, VariantFormat::Unit))
                .map(|v| literal(&v.name))
                .collect();
            let units = format!("[{}]", units.join(", "));
            let (content, read) = match tagging {
                EnumTagging::External => (
                    "content",
                    format!("{JSON}.readExternal(json, {what}, {units})"),
                ),
                EnumTagging::Internal { tag } => (
                    "obj",
                    format!("{JSON}.readInternal(json, {}, {what})", literal(tag)),
                ),
                EnumTagging::Adjacent { tag, content } => (
                    "content",
                    format!(
                        "{JSON}.readAdjacent(json, {}, {}, {what}, {units})",
                        literal(tag),
                        literal(content)
                    ),
                ),
            };
            if variants
                .iter()
                .all(|v| matches!(v.value, VariantFormat::Unit))
            {
                writeln!(w, "const [variant] = {read};")?;
            } else {
                writeln!(w, "const [variant, {content}] = {read};")?;
            }
            if variants.is_empty() {
                return writeln!(w, "throw {JSON}.unknownVariant({what}, variant);");
            }
            write!(w, "switch (variant) ")?;
            with_block(w, Newlines::BOTH, |w| {
                for variant in variants {
                    write!(w, "case {}: ", literal(&variant.name))?;
                    self.write_variant_from_json(w, name, variant, tagging, tag_field, content)?;
                }
                writeln!(w, "default: throw {JSON}.unknownVariant({what}, variant);")
            })
        })?;

        writeln!(w)?;
        write!(
            w,
            "export function jsonSerialize{name}(value: {name}): string "
        )?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "return {JSON}.stringify(toJson{name}(value));")
        })?;

        writeln!(w)?;
        write!(
            w,
            "export function jsonDeserialize{name}(text: string): {name} "
        )?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(w, "return fromJson{name}({JSON}.parse(text));")
        })
    }

    /// `return …;`, the JSON of the variant `variant` held in `value`.
    fn write_variant_to_json(
        &self,
        w: &mut dyn IndentWrite,
        variant: &Named<VariantFormat>,
        tagging: &EnumTagging,
    ) -> io::Result<()> {
        let vname = variant.name.as_str();
        // The TypeScript value holding the variant's content, and the
        // elements of a tuple variant.
        let (holder, element): (String, fn(&str, usize) -> String) = match tagging {
            EnumTagging::Adjacent { content, .. } => {
                (naming::member("value", content), |h, i| format!("{h}[{i}]"))
            }
            _ => ("value".to_string(), |h, i| format!("{h}.field{i}")),
        };
        let content = match &variant.value {
            VariantFormat::Unit => None,
            VariantFormat::NewType(inner) => Some(match (tagging, inner.as_ref()) {
                (EnumTagging::Adjacent { .. }, f) => self.write_json(f, &holder, 0),
                // The emitter spreads a named type into the variant.
                (EnumTagging::Internal { .. }, f @ Format::TypeName(_)) => {
                    self.write_json(f, "value", 0)
                }
                (_, f) => self.write_json(f, "value.value", 0),
            }),
            VariantFormat::Tuple(formats) => Some(
                self.write_json_array(
                    formats
                        .iter()
                        .enumerate()
                        .map(|(i, f)| (f, element(&holder, i))),
                    0,
                ),
            ),
            VariantFormat::Struct(fields) => {
                if let EnumTagging::Internal { tag } = tagging {
                    write!(w, "return ")?;
                    self.write_object(w, &[(tag, literal(vname))], fields, "value")?;
                    return writeln!(w, ";");
                }
                let mut buf = Vec::new();
                {
                    let mut inner = IndentedWriter::new(&mut buf, self.config.indent);
                    self.write_object(&mut inner, &[], fields, &holder)?;
                }
                Some(String::from_utf8(buf).expect("generated code is UTF-8"))
            }
            VariantFormat::Variable(_) => panic!("unexpected variable in a variant"),
        };

        match (tagging, content) {
            (EnumTagging::External, None) => writeln!(w, "return {};", literal(vname)),
            (EnumTagging::External, Some(content)) => write_return_object(w, &[(vname, content)]),
            (EnumTagging::Internal { tag } | EnumTagging::Adjacent { tag, .. }, None) => {
                write_return_object(w, &[(tag, literal(vname))])
            }
            (EnumTagging::Internal { tag }, Some(content)) => writeln!(
                w,
                "return {JSON}.writeTagged({}, {}, {content});",
                literal(tag),
                literal(vname)
            ),
            (EnumTagging::Adjacent { tag, content: key }, Some(content)) => {
                write_return_object(w, &[(tag, literal(vname)), (key, content)])
            }
        }
    }

    /// `return …;`, the variant `variant` read from the local `content`,
    /// the variant's JSON content (or, internally tagged, the whole object).
    fn write_variant_from_json(
        &self,
        w: &mut dyn IndentWrite,
        enum_name: &str,
        variant: &Named<VariantFormat>,
        tagging: &EnumTagging,
        tag_field: &str,
        content: &str,
    ) -> io::Result<()> {
        let vname = variant.name.as_str();
        let tag_entry = format!("{}: {}", naming::property_key(tag_field), literal(vname));
        let internal = match tagging {
            EnumTagging::Internal { tag } => Some(tag.as_str()),
            _ => None,
        };
        // The JSON content, which an internally tagged variant shares with
        // its tag.
        let untagged = |content: &str| match internal {
            Some(tag) => format!("{JSON}.untag({content}, {})", literal(tag)),
            None => content.to_string(),
        };
        let adjacent = match tagging {
            EnumTagging::Adjacent { content, .. } => Some(content.as_str()),
            _ => None,
        };

        match &variant.value {
            VariantFormat::Unit => writeln!(w, "return {{ {tag_entry} }};"),
            VariantFormat::NewType(inner) => match (internal, adjacent, inner.as_ref()) {
                (Some(_), _, f @ Format::TypeName(_)) => writeln!(
                    w,
                    "return {{ {tag_entry}, ...{} }};",
                    self.read_json(f, content, 0)
                ),
                (_, Some(key), f) => writeln!(
                    w,
                    "return {{ {tag_entry}, {}: {} }};",
                    naming::property_key(key),
                    self.read_json(f, content, 0)
                ),
                (_, None, f) => writeln!(
                    w,
                    "return {{ {tag_entry}, value: {} }};",
                    self.read_json(f, &untagged(content), 0)
                ),
            },
            VariantFormat::Tuple(formats) => {
                let items = self.read_tuple(formats, &untagged(content), 0);
                if let Some(key) = adjacent {
                    return writeln!(
                        w,
                        "return {{ {tag_entry}, {}: {items} }};",
                        naming::property_key(key)
                    );
                }
                with_block(w, Newlines::BOTH, |w| {
                    writeln!(w, "const items = {items};")?;
                    let fields: Vec<String> = (0..formats.len())
                        .map(|i| format!("field{i}: items[{i}]"))
                        .collect();
                    writeln!(w, "return {{ {tag_entry}, {} }};", fields.join(", "))
                })
            }
            VariantFormat::Struct(fields) => {
                let write_return = |w: &mut dyn IndentWrite, obj: &str| -> io::Result<()> {
                    writeln!(w, "return {{")?;
                    w.indent();
                    writeln!(w, "{tag_entry},")?;
                    if let Some(key) = adjacent {
                        writeln!(w, "{}: {{", naming::property_key(key))?;
                        w.indent();
                    }
                    for field in fields {
                        writeln!(
                            w,
                            "{}: {},",
                            naming::property_key(&field.name),
                            self.read_field(obj, field)
                        )?;
                    }
                    if adjacent.is_some() {
                        w.unindent();
                        writeln!(w, "}},")?;
                    }
                    w.unindent();
                    writeln!(w, "}};")
                };
                if internal.is_some() {
                    return write_return(w, content);
                }
                with_block(w, Newlines::BOTH, |w| {
                    writeln!(
                        w,
                        "const obj = {JSON}.readObject({content}, {});",
                        literal(&format!("{enum_name}::{vname}"))
                    )?;
                    write_return(w, "obj")
                })
            }
            VariantFormat::Variable(_) => panic!("unexpected variable in a variant"),
        }
    }
}

/// `return { "key": expr, … };`, on one line when every expression is.
fn write_return_object(w: &mut dyn IndentWrite, entries: &[(&str, String)]) -> io::Result<()> {
    let entries: Vec<String> = entries
        .iter()
        .map(|(key, expr)| format!("{}: {expr}", json_key(key)))
        .collect();
    if entries.iter().all(|entry| !entry.contains('\n')) {
        return writeln!(w, "return {{ {} }};", entries.join(", "));
    }
    writeln!(w, "return {{")?;
    w.indent();
    for entry in entries {
        writeln!(w, "{entry},")?;
    }
    w.unindent();
    writeln!(w, "}};")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::generation::indent::IndentConfig;

    fn make_config(features: &[Feature]) -> CodeGeneratorConfig {
        let mut cfg = CodeGeneratorConfig::new("test".to_string());
        cfg.features = features.iter().copied().collect::<BTreeSet<_>>();
        cfg
    }

    fn render(f: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>) -> String {
        let mut buf = Vec::new();
        let mut w = IndentedWriter::new(&mut buf, IndentConfig::Space(4));
        f(&mut w).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn imports_the_runtime_under_a_name_nothing_generated_can_shadow() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<TypeScript>;
        assert_eq!(
            plugin.imports(&make_config(&[])),
            [r#"import * as $json from "./serde/json";"#]
        );
    }

    #[test]
    fn installs_only_the_json_runtime() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<TypeScript>;
        let paths: Vec<_> = plugin
            .runtime_files()
            .into_iter()
            .map(|f| f.relative_path)
            .collect();
        assert_eq!(paths, ["serde/json.ts"]);
    }

    #[test]
    fn module_helpers_emit_only_the_uuid_alias() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<TypeScript>;
        let all = make_config(&[
            Feature::ListOfT,
            Feature::OptionOfT,
            Feature::SetOfT,
            Feature::MapOfT,
            Feature::TupleArray,
            Feature::BigInt,
            Feature::Bytes,
        ]);
        assert_eq!(render(|w| plugin.module_helpers(w, &all)), "");
        let uuid = make_config(&[Feature::Uuid]);
        assert_eq!(
            render(|w| plugin.module_helpers(w, &uuid)),
            format!("\n{FEATURE_UUID}")
        );
    }

    #[test]
    fn map_keys_are_written_as_strings() {
        let config = make_config(&[]);
        let codec = Codec { config: &config };
        assert_eq!(codec.write_key(&Format::Str, "k", 0), "k");
        assert_eq!(codec.write_key(&Format::U128, "k", 0), "`${k}`");
        assert_eq!(codec.write_key(&Format::Bool, "k", 0), "`${k}`");
        let name = Format::TypeName(QualifiedTypeName::root("Key".to_string()));
        assert_eq!(
            codec.write_key(&name, "k", 0),
            "$json.writeKey(Key.toJson(k))"
        );
    }

    #[test]
    fn map_keys_are_read_from_strings() {
        let config = make_config(&[]);
        let codec = Codec { config: &config };
        assert_eq!(codec.read_key(&Format::Str, "k", 0), "k");
        assert_eq!(
            codec.read_key(&Format::I64, "k", 0),
            "$json.readI64($json.keyLiteral(k))"
        );
        let name = Format::TypeName(QualifiedTypeName::root("Key".to_string()));
        assert_eq!(
            codec.read_key(&name, "k", 0),
            "$json.readKey(k, Key.fromJson)"
        );
    }

    #[test]
    fn a_field_named_proto_is_an_own_property() {
        assert_eq!(json_key("__proto__"), r#"["__proto__"]"#);
        assert_eq!(json_key("with-dash"), r#""with-dash""#);
    }
}
