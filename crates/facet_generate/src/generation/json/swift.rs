//! `EmitterPlugin<Swift>` implementation for the [`JsonPlugin`].
//!
//! Generated types conform to `Codable` and are encoded with Foundation's
//! `JSONEncoder` / `JSONDecoder`, in the shape `serde_json` gives the same Rust
//! types — so JSON written on either side reads on the other.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `import Serde` |
//! | `type_conformances` | `Codable` |
//! | `has_type_body` | Always `true` |
//! | `type_body` | `CodingKeys`, hand-written `init(from:)` / `encode(to:)` where synthesis would not match, and `jsonSerialize` / `jsonDeserialize` wrappers |
//! | `runtime_files` | `Int128` / `UInt128` / `Indirect` from the Serde runtime, and `JsonCoding.swift` |
//!
//! # Wire format
//!
//! The wire format is `serde_json`'s for the reflected Rust types:
//!
//! | Rust | JSON |
//! |---|---|
//! | struct with named fields | object keyed by the field's (renamed) Rust name |
//! | unit struct, `()` | `null` |
//! | newtype struct | the inner value |
//! | tuple struct, tuple | array |
//! | `Option<T>` | `null` or the value (a missing key reads as `null`) |
//! | `Vec<T>`, `[T; N]`, sets, bytes | array |
//! | map | object, with integer and boolean keys written as strings |
//! | `char`, `Uuid` | string (a `Uuid` hyphenated and lowercase) |
//! | 128-bit integer | number, written and read exactly |
//! | enum | externally tagged — `"Unit"`, `{"NewType": …}`, `{"Tuple": […]}`, `{"Struct": {…}}` — or internally / adjacently tagged per `#[facet(tag, content)]` |
//!
//! # Synthesized and hand-written coding
//!
//! A struct with named fields gets explicit `CodingKeys` mapping each property
//! to its wire name. When every field's Swift type already encodes the way
//! Rust does, Swift synthesizes `init(from:)` and `encode(to:)` from those
//! keys. Otherwise — an optional field (synthesis omits a `nil` key, Rust
//! writes `null`), a tuple, `()`, `char`, `Uuid`, a map with non-string keys,
//! or an `@Indirect` field — both are written out. Unit, newtype and tuple
//! structs and every enum always get hand-written coding, since Swift's own
//! shapes for them differ from Rust's.

#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::io;

use heck::{ToLowerCamelCase as _, ToUpperCamelCase as _};
use indoc::writedoc;

use crate::generation::{
    CodeGeneratorConfig,
    indent::{IndentWrite, Newlines, with_block},
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
    swift::{Swift, case_name, emitter::needs_indirect, field_name, naming, render_type},
};
use crate::reflection::format::{ContainerFormat, EnumTagging, Format, Named, VariantFormat};

use super::JsonPlugin;

// ---------------------------------------------------------------------------
// EmitterPlugin implementation
// ---------------------------------------------------------------------------

impl EmitterPlugin<Swift> for JsonPlugin {
    /// The parts of the Serde runtime the generated types use (`Int128`,
    /// `UInt128`, and the `@Indirect` property wrapper), and the JSON helpers.
    fn runtime_files(&self) -> Vec<RuntimeFile> {
        [
            (
                "Indirect.swift",
                &include_bytes!("../../../runtime/swift/Sources/Serde/Indirect.swift")[..],
            ),
            (
                "Int128.swift",
                &include_bytes!("../../../runtime/swift/Sources/Serde/Int128.swift")[..],
            ),
            (
                "UInt128.swift",
                &include_bytes!("../../../runtime/swift/Sources/Serde/UInt128.swift")[..],
            ),
            (
                "JsonCoding.swift",
                &include_bytes!("../../../runtime/swift-json/Sources/Serde/JsonCoding.swift")[..],
            ),
        ]
        .into_iter()
        .map(|(name, contents)| RuntimeFile {
            relative_path: format!("Sources/Serde/{name}"),
            contents: contents.to_vec(),
        })
        .collect()
    }

    fn imports(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec!["Serde".to_string()]
    }

    fn type_conformances(&self, ctx: &EmitContext) -> Vec<String> {
        vec![naming::builtin("Codable", ctx.config).into_owned()]
    }

    fn has_type_body(&self, _ctx: &EmitContext) -> bool {
        true
    }

    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        let name = ctx.name();
        let names = Names::new(ctx.config);
        match ctx.container.format {
            ContainerFormat::UnitStruct(_) => write_unit_struct(w, &names)?,
            ContainerFormat::NewTypeStruct(format, _) => {
                write_newtype_struct(w, format, &names)?;
            }
            ContainerFormat::TupleStruct(formats, _) => {
                write_tuple_struct(w, formats, &names)?;
            }
            ContainerFormat::Struct(fields, _) => write_struct(w, name, fields, &names)?,
            ContainerFormat::Enum(variants, tagging, _) => {
                write_enum(w, name, variants, tagging, &names)?;
            }
        }
        write_wrappers(w, name, &names)
    }
}

// ---------------------------------------------------------------------------
// Names
// ---------------------------------------------------------------------------

/// The spellings of the standard-library names the generated code uses,
/// qualified where the module shadows them.
struct Names<'a> {
    config: &'a CodeGeneratorConfig,
    coding_key: String,
    decoder: String,
    decoding_error: String,
    encoder: String,
    encoding_error: String,
    string: String,
}

impl<'a> Names<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        let builtin = |name| naming::builtin(name, config).into_owned();
        Self {
            config,
            coding_key: builtin("CodingKey"),
            decoder: builtin("Decoder"),
            decoding_error: builtin("DecodingError"),
            encoder: builtin("Encoder"),
            encoding_error: builtin("EncodingError"),
            string: builtin("String"),
        }
    }

    fn render(&self, format: &Format) -> String {
        render_type(format, self.config)
    }
}

/// The key type of objects whose keys are computed at run time.
const JSON_KEY: &str = "Serde.JsonKey";

/// A Swift string literal holding `value`.
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
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `Serde.JsonKey("…")`, the key spelled `name`.
fn json_key(name: &str) -> String {
    format!("{JSON_KEY}({})", literal(name))
}

// ---------------------------------------------------------------------------
// Value classification
// ---------------------------------------------------------------------------

/// Whether a value of this format encodes, through its Swift type's own
/// `Codable` conformance, exactly the way `serde_json` encodes the Rust value.
fn is_native(format: &Format) -> bool {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::Unit | Format::Char | Format::Uuid => false,
        Format::Tuple(formats) => formats.len() == 1 && is_native(&formats[0]),
        Format::Option(inner)
        | Format::Seq(inner)
        | Format::Set(inner)
        | Format::TupleArray { content: inner, .. } => is_native(inner),
        Format::Map { key, value } => matches!(**key, Format::Str) && is_native(value),
        _ => true,
    }
}

/// The runtime adapter a leaf value Swift cannot encode the Rust way is
/// wrapped in, if the format is one.
fn adapter(format: &Format) -> Option<&'static str> {
    match format {
        Format::Unit => Some("Serde.JsonUnit"),
        Format::Char => Some("Serde.JsonChar"),
        Format::Uuid => Some("Serde.JsonUuid"),
        _ => None,
    }
}

/// `expr` wrapped in its format's adapter.
fn adapt(format: &Format, adapter: &str, expr: &str) -> String {
    if matches!(format, Format::Unit) {
        format!("{adapter}()")
    } else {
        format!("{adapter}({expr})")
    }
}

/// The single element of a one-element tuple, which Swift writes as the
/// element itself.
fn unwrap_single(format: &Format) -> &Format {
    match format {
        Format::Tuple(formats) if formats.len() == 1 => unwrap_single(&formats[0]),
        format => format,
    }
}

/// How a map key of this format becomes an object key and back.
enum MapKey<'a> {
    /// A string, used as it is.
    Str,
    /// A value whose own coding gives the key (an integer, a boolean, or a
    /// type that encodes as one of those or as a string).
    Coded(String),
    /// A leaf wrapped in a runtime adapter.
    Adapted(&'static str),
    /// A key JSON cannot express.
    Unsupported(&'a Format),
}

fn map_key<'a>(key: &'a Format, names: &Names) -> MapKey<'a> {
    let key = unwrap_single(key);
    if matches!(key, Format::Str) {
        MapKey::Str
    } else if let Some(adapter) = adapter(key) {
        MapKey::Adapted(adapter)
    } else if is_native(key) {
        MapKey::Coded(names.render(key))
    } else {
        MapKey::Unsupported(key)
    }
}

// ---------------------------------------------------------------------------
// Slots
// ---------------------------------------------------------------------------

/// Where a value is encoded to or decoded from.
#[derive(Clone, Copy)]
enum Slot<'a> {
    /// Under the key `key` of the keyed container `container`.
    Keyed { container: &'a str, key: &'a str },
    /// The next element of the unkeyed container `container`.
    Unkeyed(&'a str),
    /// The whole of the encoder or decoder `coder`.
    Whole(&'a str),
}

impl Slot<'_> {
    /// The expression for an unkeyed container nested in the slot.
    fn nested_unkeyed(self) -> String {
        match self {
            Slot::Keyed { container, key } => {
                format!("{container}.nestedUnkeyedContainer(forKey: {key})")
            }
            Slot::Unkeyed(container) => format!("{container}.nestedUnkeyedContainer()"),
            Slot::Whole(coder) => format!("{coder}.unkeyedContainer()"),
        }
    }

    /// The expression for a container keyed by `keys` nested in the slot.
    fn nested_keyed(self, keys: &str) -> String {
        match self {
            Slot::Keyed { container, key } => {
                format!("{container}.nestedContainer(keyedBy: {keys}.self, forKey: {key})")
            }
            Slot::Unkeyed(container) => {
                format!("{container}.nestedContainer(keyedBy: {keys}.self)")
            }
            Slot::Whole(coder) => format!("{coder}.container(keyedBy: {keys}.self)"),
        }
    }

    /// The coding path at the slot, for error messages.
    fn coding_path(self) -> String {
        match self {
            Slot::Keyed { container, .. } | Slot::Unkeyed(container) => {
                format!("{container}.codingPath")
            }
            Slot::Whole(coder) => format!("{coder}.codingPath"),
        }
    }
}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

/// Writes statements encoding `expr`, a value of `format`, into `slot`.
///
/// `depth` numbers the locals, so that nested values don't shadow each other.
fn write_encode(
    w: &mut dyn IndentWrite,
    format: &Format,
    expr: &str,
    slot: Slot,
    depth: usize,
    names: &Names,
) -> io::Result<()> {
    let format = unwrap_single(format);
    if is_native(format) {
        return write_encode_value(w, expr, slot, depth);
    }
    if let Some(adapter) = adapter(format) {
        return write_encode_value(w, &adapt(format, adapter, expr), slot, depth);
    }
    match format {
        Format::Option(inner) => {
            let value = format!("value{depth}");
            // `()` is written without its value, so don't bind it.
            if matches!(unwrap_single(inner), Format::Unit) {
                write!(w, "if case .some = {expr} ")?;
            } else {
                write!(w, "if let {value} = {expr} ")?;
            }
            with_block(w, Newlines::OPEN, |w| {
                write_encode(w, inner, &value, slot, depth + 1, names)
            })?;
            write!(w, " else ")?;
            with_block(w, Newlines::BOTH, |w| write_encode_nil(w, slot, depth))
        }
        Format::Seq(inner) | Format::Set(inner) | Format::TupleArray { content: inner, .. } => {
            let nested = format!("nested{depth}");
            let item = if matches!(unwrap_single(inner), Format::Unit) {
                "_".to_string()
            } else {
                format!("item{depth}")
            };
            write!(w, "do ")?;
            with_block(w, Newlines::BOTH, |w| {
                writeln!(w, "var {nested} = {}", slot.nested_unkeyed())?;
                write!(w, "for {item} in {expr} ")?;
                with_block(w, Newlines::BOTH, |w| {
                    write_encode(w, inner, &item, Slot::Unkeyed(&nested), depth + 1, names)
                })
            })
        }
        Format::Tuple(formats) => {
            let nested = format!("nested{depth}");
            write!(w, "do ")?;
            with_block(w, Newlines::BOTH, |w| {
                writeln!(w, "var {nested} = {}", slot.nested_unkeyed())?;
                for (i, format) in formats.iter().enumerate() {
                    write_encode(
                        w,
                        format,
                        &format!("{expr}.{i}"),
                        Slot::Unkeyed(&nested),
                        depth + 1,
                        names,
                    )?;
                }
                Ok(())
            })
        }
        Format::Map { key, value } => {
            let map_key = map_key(key, names);
            if let MapKey::Unsupported(key) = map_key {
                return writeln!(
                    w,
                    "throw {}.invalidValue({expr}, .init(codingPath: {}, debugDescription: {}))",
                    names.encoding_error,
                    slot.coding_path(),
                    literal(&format!(
                        "A JSON object key cannot be a {}",
                        names.render(key)
                    )),
                );
            }
            let nested = format!("nested{depth}");
            let key_var = format!("key{depth}");
            let value_var = if matches!(unwrap_single(value), Format::Unit) {
                "_".to_string()
            } else {
                format!("value{depth}")
            };
            write!(w, "do ")?;
            with_block(w, Newlines::BOTH, |w| {
                writeln!(w, "var {nested} = {}", slot.nested_keyed(JSON_KEY))?;
                write!(w, "for ({key_var}, {value_var}) in {expr} ")?;
                with_block(w, Newlines::BOTH, |w| {
                    let key_expr = match map_key {
                        MapKey::Str => format!("{JSON_KEY}({key_var})"),
                        MapKey::Coded(_) => {
                            format!("{JSON_KEY}(try Serde.jsonMapKey({key_var}))")
                        }
                        MapKey::Adapted(adapter) => format!(
                            "{JSON_KEY}(try Serde.jsonMapKey({}))",
                            adapt(unwrap_single(key), adapter, &key_var)
                        ),
                        MapKey::Unsupported(_) => unreachable!("handled above"),
                    };
                    let object_key = format!("objectKey{depth}");
                    writeln!(w, "let {object_key} = {key_expr}")?;
                    write_encode(
                        w,
                        value,
                        &value_var,
                        Slot::Keyed {
                            container: &nested,
                            key: &object_key,
                        },
                        depth + 1,
                        names,
                    )
                })
            })
        }
        _ => unreachable!("native and adapted formats are handled above"),
    }
}

/// Writes a statement encoding the `Encodable` value `expr` into `slot`.
fn write_encode_value(
    w: &mut dyn IndentWrite,
    expr: &str,
    slot: Slot,
    depth: usize,
) -> io::Result<()> {
    match slot {
        Slot::Keyed { container, key } => {
            writeln!(w, "try {container}.encode({expr}, forKey: {key})")
        }
        Slot::Unkeyed(container) => writeln!(w, "try {container}.encode({expr})"),
        Slot::Whole(coder) => {
            writeln!(w, "var single{depth} = {coder}.singleValueContainer()")?;
            writeln!(w, "try single{depth}.encode({expr})")
        }
    }
}

/// Writes a statement encoding `null` into `slot`.
fn write_encode_nil(w: &mut dyn IndentWrite, slot: Slot, depth: usize) -> io::Result<()> {
    match slot {
        Slot::Keyed { container, key } => writeln!(w, "try {container}.encodeNil(forKey: {key})"),
        Slot::Unkeyed(container) => writeln!(w, "try {container}.encodeNil()"),
        Slot::Whole(coder) => {
            writeln!(w, "var single{depth} = {coder}.singleValueContainer()")?;
            writeln!(w, "try single{depth}.encodeNil()")
        }
    }
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

/// Writes an expression decoding a value of `format` from `slot`.
///
/// The expression may span several lines (a closure called in place), and has
/// no trailing newline.
fn write_decode(
    w: &mut dyn IndentWrite,
    format: &Format,
    slot: Slot,
    depth: usize,
    names: &Names,
) -> io::Result<()> {
    let format = unwrap_single(format);
    if is_native(format) {
        return match (slot, format) {
            (Slot::Keyed { container, key }, Format::Option(inner)) => write!(
                w,
                "try {container}.decodeIfPresent({}.self, forKey: {key})",
                names.render(inner)
            ),
            _ => write_decode_value(w, &names.render(format), slot),
        };
    }
    if let Some(adapter) = adapter(format) {
        write_decode_value(w, adapter, slot)?;
        return write!(w, ".value");
    }

    let ty = names.render(format);
    write_closure(w, &ty, |w| match format {
        Format::Option(inner) => {
            match slot {
                Slot::Keyed { container, key } => writeln!(
                    w,
                    "guard {container}.contains({key}), try !{container}.decodeNil(forKey: {key}) else {{ return nil }}"
                )?,
                Slot::Unkeyed(container) => {
                    writeln!(
                        w,
                        "guard try !{container}.decodeNil() else {{ return nil }}"
                    )?;
                }
                Slot::Whole(coder) => writeln!(
                    w,
                    "guard try !{coder}.singleValueContainer().decodeNil() else {{ return nil }}"
                )?,
            }
            write!(w, "return ")?;
            write_decode(w, inner, slot, depth + 1, names)?;
            writeln!(w)
        }
        Format::Seq(inner) | Format::Set(inner) | Format::TupleArray { content: inner, .. } => {
            let nested = format!("nested{depth}");
            let result = format!("result{depth}");
            let add = if matches!(format, Format::Set(_)) {
                "insert"
            } else {
                "append"
            };
            writeln!(w, "var {nested} = try {}", slot.nested_unkeyed())?;
            writeln!(w, "var {result}: {ty} = []")?;
            write!(w, "while !{nested}.isAtEnd ")?;
            with_block(w, Newlines::BOTH, |w| {
                write!(w, "{result}.{add}(")?;
                write_decode(w, inner, Slot::Unkeyed(&nested), depth + 1, names)?;
                writeln!(w, ")")
            })?;
            writeln!(w, "return {result}")
        }
        Format::Tuple(formats) => {
            let nested = format!("nested{depth}");
            writeln!(w, "var {nested} = try {}", slot.nested_unkeyed())?;
            writeln!(w, "return (")?;
            w.indent();
            for (i, format) in formats.iter().enumerate() {
                write_decode(w, format, Slot::Unkeyed(&nested), depth + 1, names)?;
                writeln!(w, "{}", if i + 1 < formats.len() { "," } else { "" })?;
            }
            w.unindent();
            writeln!(w, ")")
        }
        Format::Map { key, value } => {
            let map_key = map_key(key, names);
            if let MapKey::Unsupported(key) = map_key {
                return writeln!(
                    w,
                    "throw {}.dataCorrupted(.init(codingPath: {}, debugDescription: {}))",
                    names.decoding_error,
                    slot.coding_path(),
                    literal(&format!(
                        "A JSON object key cannot be a {}",
                        names.render(key)
                    )),
                );
            }
            let nested = format!("nested{depth}");
            let result = format!("result{depth}");
            let key_var = format!("key{depth}");
            writeln!(w, "let {nested} = try {}", slot.nested_keyed(JSON_KEY))?;
            writeln!(w, "var {result}: {ty} = [:]")?;
            write!(w, "for {key_var} in {nested}.allKeys ")?;
            with_block(w, Newlines::BOTH, |w| {
                let map_key_expr = match map_key {
                    MapKey::Str => format!("{key_var}.stringValue"),
                    MapKey::Coded(ty) => {
                        format!("try Serde.jsonMapKey({key_var}.stringValue, as: {ty}.self)")
                    }
                    MapKey::Adapted(adapter) => format!(
                        "try Serde.jsonMapKey({key_var}.stringValue, as: {adapter}.self).value"
                    ),
                    MapKey::Unsupported(_) => unreachable!("handled above"),
                };
                let map_key_var = format!("mapKey{depth}");
                writeln!(w, "let {map_key_var} = {map_key_expr}")?;
                write!(w, "{result}.updateValue(")?;
                write_decode(
                    w,
                    value,
                    Slot::Keyed {
                        container: &nested,
                        key: &key_var,
                    },
                    depth + 1,
                    names,
                )?;
                writeln!(w, ", forKey: {map_key_var})")
            })?;
            writeln!(w, "return {result}")
        }
        _ => unreachable!("native and adapted formats are handled above"),
    })
}

/// Writes an expression decoding a `Decodable` value of type `ty` from `slot`.
fn write_decode_value(w: &mut dyn IndentWrite, ty: &str, slot: Slot) -> io::Result<()> {
    match slot {
        Slot::Keyed { container, key } => {
            write!(w, "try {container}.decode({ty}.self, forKey: {key})")
        }
        Slot::Unkeyed(container) => write!(w, "try {container}.decode({ty}.self)"),
        Slot::Whole(coder) => write!(w, "try {coder}.singleValueContainer().decode({ty}.self)"),
    }
}

/// Writes a closure returning `ty`, called in place, whose body `body` writes.
fn write_closure(
    w: &mut dyn IndentWrite,
    ty: &str,
    body: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>,
) -> io::Result<()> {
    writeln!(w, "try {{ () throws -> {ty} in")?;
    w.indent();
    body(w)?;
    w.unindent();
    write!(w, "}}()")
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Writes `public init(from decoder: Decoder) throws { … }`.
fn write_init_from(
    w: &mut dyn IndentWrite,
    names: &Names,
    body: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>,
) -> io::Result<()> {
    writeln!(w)?;
    write!(w, "public init(from decoder: {}) throws ", names.decoder)?;
    with_block(w, Newlines::BOTH, body)
}

/// Writes `public func encode(to encoder: Encoder) throws { … }`.
fn write_encode_to(
    w: &mut dyn IndentWrite,
    names: &Names,
    body: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>,
) -> io::Result<()> {
    writeln!(w)?;
    write!(
        w,
        "public func encode(to encoder: {}) throws ",
        names.encoder
    )?;
    with_block(w, Newlines::BOTH, body)
}

/// Rust writes a unit struct as `null`.
///
/// The registry also records a braced struct with no fields (or none left
/// once skipped ones are dropped) as a unit struct, which Rust writes as `{}`,
/// so that reads too.
fn write_unit_struct(w: &mut dyn IndentWrite, names: &Names) -> io::Result<()> {
    write_init_from(w, names, |w| {
        write!(w, "if try decoder.singleValueContainer().decodeNil() ")?;
        with_block(w, Newlines::BOTH, |w| writeln!(w, "return"))?;
        writeln!(w, "_ = try decoder.container(keyedBy: {JSON_KEY}.self)")
    })?;
    write_encode_to(w, names, |w| {
        writeln!(w, "var container = encoder.singleValueContainer()")?;
        writeln!(w, "try container.encodeNil()")
    })
}

/// Rust writes a newtype struct as the value it wraps.
fn write_newtype_struct(w: &mut dyn IndentWrite, format: &Format, names: &Names) -> io::Result<()> {
    write_init_from(w, names, |w| {
        write!(w, "self.value = ")?;
        write_decode(w, format, Slot::Whole("decoder"), 0, names)?;
        writeln!(w)
    })?;
    write_encode_to(w, names, |w| {
        write_encode(w, format, "self.value", Slot::Whole("encoder"), 0, names)
    })
}

/// Rust writes a tuple struct as an array.
fn write_tuple_struct(
    w: &mut dyn IndentWrite,
    formats: &[Format],
    names: &Names,
) -> io::Result<()> {
    write_init_from(w, names, |w| {
        let keyword = if formats.is_empty() {
            "_ ="
        } else {
            "var container ="
        };
        writeln!(w, "{keyword} try decoder.unkeyedContainer()")?;
        for (i, format) in formats.iter().enumerate() {
            write!(w, "self.field{i} = ")?;
            write_decode(w, format, Slot::Unkeyed("container"), 0, names)?;
            writeln!(w)?;
        }
        Ok(())
    })?;
    write_encode_to(w, names, |w| {
        let keyword = if formats.is_empty() {
            "_ ="
        } else {
            "var container ="
        };
        writeln!(w, "{keyword} encoder.unkeyedContainer()")?;
        for (i, format) in formats.iter().enumerate() {
            write_encode(
                w,
                format,
                &format!("self.field{i}"),
                Slot::Unkeyed("container"),
                0,
                names,
            )?;
        }
        Ok(())
    })
}

/// Writes `enum <keys>: String, CodingKey` with a case per field, each
/// raw value the field's wire name.
fn write_coding_keys(
    w: &mut dyn IndentWrite,
    keys: &str,
    fields: &[Named<Format>],
    names: &Names,
) -> io::Result<()> {
    writeln!(w)?;
    write!(w, "enum {keys}: {}, {} ", names.string, names.coding_key)?;
    with_block(w, Newlines::BOTH, |w| {
        for field in fields {
            write_coding_key_case(w, &field_name(&field.name), &field.name)?;
        }
        Ok(())
    })
}

fn write_coding_key_case(w: &mut dyn IndentWrite, case: &str, wire: &str) -> io::Result<()> {
    if case.trim_matches('`') == wire {
        writeln!(w, "case {case}")
    } else {
        writeln!(w, "case {case} = {}", literal(wire))
    }
}

/// A struct with named fields is a JSON object keyed by the fields' wire
/// names.
fn write_struct(
    w: &mut dyn IndentWrite,
    name: &str,
    fields: &[Named<Format>],
    names: &Names,
) -> io::Result<()> {
    if fields.is_empty() {
        // Synthesized coding writes and reads `{}`, as Rust does.
        return Ok(());
    }
    write_coding_keys(w, "CodingKeys", fields, names)?;

    let synthesized = fields.iter().all(|field| {
        is_native(&field.value)
            && !matches!(field.value, Format::Option(_))
            && !needs_indirect(&field.value, name)
    });
    if synthesized {
        return Ok(());
    }

    write_init_from(w, names, |w| {
        writeln!(
            w,
            "let container = try decoder.container(keyedBy: CodingKeys.self)"
        )?;
        for field in fields {
            let property = field_name(&field.name);
            let key = format!(".{property}");
            write!(w, "self.{property} = ")?;
            write_decode(
                w,
                &field.value,
                Slot::Keyed {
                    container: "container",
                    key: &key,
                },
                0,
                names,
            )?;
            writeln!(w)?;
        }
        Ok(())
    })?;
    write_encode_to(w, names, |w| {
        writeln!(
            w,
            "var container = encoder.container(keyedBy: CodingKeys.self)"
        )?;
        for field in fields {
            let property = field_name(&field.name);
            let key = format!(".{property}");
            write_encode(
                w,
                &field.value,
                &format!("self.{property}"),
                Slot::Keyed {
                    container: "container",
                    key: &key,
                },
                0,
                names,
            )?;
        }
        Ok(())
    })
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The name of the `CodingKey` enum holding a struct variant's field keys.
fn variant_keys(variant: &Named<VariantFormat>) -> String {
    format!("{}CodingKeys", variant.name.to_upper_camel_case())
}

/// The keys type a struct variant's fields are coded with: its own
/// `CodingKey` enum, or `Serde.JsonKey` when it has no fields (an enum with
/// no cases cannot have a raw type).
fn variant_keys_type(variant: &Named<VariantFormat>, fields: &[Named<Format>]) -> String {
    if fields.is_empty() {
        JSON_KEY.to_string()
    } else {
        variant_keys(variant)
    }
}

/// The payload bindings of a variant's `case` pattern: `payload0`, ….
fn payloads(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("payload{i}")).collect()
}

/// `case .name(let payload0, …):`, the pattern binding every payload (but
/// `()`, which is written without its value).
fn case_pattern(variant: &Named<VariantFormat>) -> String {
    let name = case_name(&variant.name);
    let formats: Vec<&Format> = match &variant.value {
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
        VariantFormat::Unit => vec![],
        VariantFormat::NewType(format) => vec![format.as_ref()],
        VariantFormat::Tuple(formats) => formats.iter().collect(),
        VariantFormat::Struct(fields) => fields.iter().map(|f| &f.value).collect(),
    };
    if formats.is_empty() {
        format!("case .{name}:")
    } else {
        let bindings = formats
            .iter()
            .zip(payloads(formats.len()))
            .map(|(format, p)| {
                if matches!(unwrap_single(format), Format::Unit) {
                    "_".to_string()
                } else {
                    format!("let {p}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("case .{name}({bindings}):")
    }
}

/// Writes `self = .name(…)` with each argument written by `arg`.
fn write_construct(
    w: &mut dyn IndentWrite,
    variant: &Named<VariantFormat>,
    mut arg: impl FnMut(&mut dyn IndentWrite, usize, &Format) -> io::Result<()>,
) -> io::Result<()> {
    let name = case_name(&variant.name);
    let (formats, labels): (Vec<&Format>, Vec<Option<String>>) = match &variant.value {
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
        VariantFormat::Unit => return writeln!(w, "self = .{name}"),
        VariantFormat::NewType(format) => (vec![format.as_ref()], vec![None]),
        VariantFormat::Tuple(formats) => (formats.iter().collect(), vec![None; formats.len()]),
        VariantFormat::Struct(fields) => (
            fields.iter().map(|f| &f.value).collect(),
            // The argument label is the bare name: Swift warns that a keyword
            // does not need escaping in an argument list.
            fields
                .iter()
                .map(|f| Some(f.name.to_lower_camel_case()))
                .collect(),
        ),
    };
    if formats.is_empty() {
        return writeln!(w, "self = .{name}()");
    }
    writeln!(w, "self = .{name}(")?;
    w.indent();
    for (i, (format, label)) in formats.iter().zip(&labels).enumerate() {
        if let Some(label) = label {
            write!(w, "{label}: ")?;
        }
        arg(w, i, format)?;
        writeln!(w, "{}", if i + 1 < formats.len() { "," } else { "" })?;
    }
    w.unindent();
    writeln!(w, ")")
}

fn write_enum(
    w: &mut dyn IndentWrite,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    tagging: &EnumTagging,
    names: &Names,
) -> io::Result<()> {
    let variants: Vec<&Named<VariantFormat>> = variants.values().collect();

    if variants.is_empty() {
        write_init_from(w, names, |w| {
            writeln!(
                w,
                "throw {}.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: {}))",
                names.decoding_error,
                literal(&format!("{name} has no variants")),
            )
        })?;
        return write_encode_to(w, names, |w| writeln!(w, "switch self {{}}"));
    }

    // The keys of the variants' names, for externally tagged enums.
    if matches!(tagging, EnumTagging::External) {
        writeln!(w)?;
        write!(
            w,
            "enum CodingKeys: {}, {} ",
            names.string, names.coding_key
        )?;
        with_block(w, Newlines::BOTH, |w| {
            for variant in &variants {
                write_coding_key_case(w, &case_name(&variant.name), &variant.name)?;
            }
            Ok(())
        })?;
    }
    for variant in &variants {
        if let VariantFormat::Struct(fields) = &variant.value
            && !fields.is_empty()
        {
            write_coding_keys(w, &variant_keys(variant), fields, names)?;
        }
    }

    match tagging {
        EnumTagging::External => write_external_enum(w, name, &variants, names),
        EnumTagging::Internal { tag } => write_internal_enum(w, name, &variants, tag, names),
        EnumTagging::Adjacent { tag, content } => {
            write_adjacent_enum(w, name, &variants, tag, content, names)
        }
    }
}

/// `"Unit"`, `{"NewType": …}`, `{"Tuple": […]}`, `{"Struct": {…}}`.
fn write_external_enum(
    w: &mut dyn IndentWrite,
    name: &str,
    variants: &[&Named<VariantFormat>],
    names: &Names,
) -> io::Result<()> {
    write_init_from(w, names, |w| {
        let units: Vec<_> = variants
            .iter()
            .filter(|v| matches!(v.value, VariantFormat::Unit))
            .collect();
        if !units.is_empty() {
            // A unit variant is written as its name alone.
            write!(
                w,
                "if let container = try? decoder.singleValueContainer(), let name = try? container.decode({}.self) ",
                names.string
            )?;
            with_block(w, Newlines::BOTH, |w| {
                write!(w, "switch name ")?;
                with_block(w, Newlines::BOTH, |w| {
                    w.unindent();
                    for variant in &units {
                        writeln!(w, "case {}:", literal(&variant.name))?;
                        w.indent();
                        writeln!(w, "self = .{}", case_name(&variant.name))?;
                        w.unindent();
                    }
                    writeln!(w, "default:")?;
                    w.indent();
                    writeln!(
                        w,
                        r#"throw {}.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for {name}")"#,
                        names.decoding_error,
                    )?;
                    w.unindent();
                    w.indent();
                    Ok(())
                })?;
                writeln!(w, "return")
            })?;
        }
        writeln!(
            w,
            "let container = try decoder.container(keyedBy: CodingKeys.self)"
        )?;
        write!(
            w,
            "guard container.allKeys.count == 1, let key = container.allKeys.first else "
        )?;
        with_block(w, Newlines::BOTH, |w| {
            writeln!(
                w,
                "throw {}.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: {}))",
                names.decoding_error,
                literal(&format!("Expected exactly one variant of {name}")),
            )
        })?;
        write!(w, "switch key ")?;
        with_block(w, Newlines::BOTH, |w| {
            w.unindent();
            for variant in variants {
                let case = case_name(&variant.name);
                let key = format!(".{case}");
                writeln!(w, "case .{case}:")?;
                w.indent();
                write_payload_decode(w, variant, "container", &key, names)?;
                w.unindent();
            }
            w.indent();
            Ok(())
        })
    })?;
    write_encode_to(w, names, |w| {
        write!(w, "switch self ")?;
        with_block(w, Newlines::BOTH, |w| {
            w.unindent();
            for variant in variants {
                writeln!(w, "{}", case_pattern(variant))?;
                w.indent();
                if let VariantFormat::Unit = variant.value {
                    writeln!(w, "var container = encoder.singleValueContainer()")?;
                    writeln!(w, "try container.encode({})", literal(&variant.name))?;
                } else {
                    writeln!(
                        w,
                        "var container = encoder.container(keyedBy: CodingKeys.self)"
                    )?;
                    let key = format!(".{}", case_name(&variant.name));
                    write_payload_encode(w, variant, "container", &key, names)?;
                }
                w.unindent();
            }
            w.indent();
            Ok(())
        })
    })
}

/// Writes statements decoding a variant's payload from under `key` in the
/// keyed container `container`, and assigning the variant to `self`.
fn write_payload_decode(
    w: &mut dyn IndentWrite,
    variant: &Named<VariantFormat>,
    container: &str,
    key: &str,
    names: &Names,
) -> io::Result<()> {
    match &variant.value {
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
        VariantFormat::Unit => writeln!(w, "self = .{}", case_name(&variant.name)),
        VariantFormat::NewType(_) => write_construct(w, variant, |w, _, format| {
            write_decode(w, format, Slot::Keyed { container, key }, 0, names)
        }),
        VariantFormat::Tuple(_) => {
            writeln!(
                w,
                "var nested = try {container}.nestedUnkeyedContainer(forKey: {key})"
            )?;
            write_construct(w, variant, |w, _, format| {
                write_decode(w, format, Slot::Unkeyed("nested"), 0, names)
            })
        }
        VariantFormat::Struct(fields) => {
            let keys = variant_keys_type(variant, fields);
            if fields.is_empty() {
                writeln!(
                    w,
                    "_ = try {container}.nestedContainer(keyedBy: {keys}.self, forKey: {key})"
                )?;
            } else {
                writeln!(
                    w,
                    "let nested = try {container}.nestedContainer(keyedBy: {keys}.self, forKey: {key})"
                )?;
            }
            write_struct_variant_construct(w, variant, fields, "nested", names)
        }
    }
}

/// Writes `self = .name(field: …, …)`, each field decoded from the keyed
/// container `container`.
fn write_struct_variant_construct(
    w: &mut dyn IndentWrite,
    variant: &Named<VariantFormat>,
    fields: &[Named<Format>],
    container: &str,
    names: &Names,
) -> io::Result<()> {
    write_construct(w, variant, |w, i, format| {
        let key = format!(".{}", field_name(&fields[i].name));
        write_decode(
            w,
            format,
            Slot::Keyed {
                container,
                key: &key,
            },
            0,
            names,
        )
    })
}

/// Writes statements encoding a (non-unit) variant's bound payloads under
/// `key` in the keyed container `container`.
fn write_payload_encode(
    w: &mut dyn IndentWrite,
    variant: &Named<VariantFormat>,
    container: &str,
    key: &str,
    names: &Names,
) -> io::Result<()> {
    match &variant.value {
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
        VariantFormat::Unit => Ok(()),
        VariantFormat::NewType(format) => write_encode(
            w,
            format,
            "payload0",
            Slot::Keyed { container, key },
            0,
            names,
        ),
        VariantFormat::Tuple(formats) => {
            let keyword = if formats.is_empty() {
                "_ ="
            } else {
                "var nested ="
            };
            writeln!(
                w,
                "{keyword} {container}.nestedUnkeyedContainer(forKey: {key})"
            )?;
            for (format, payload) in formats.iter().zip(payloads(formats.len())) {
                write_encode(w, format, &payload, Slot::Unkeyed("nested"), 0, names)?;
            }
            Ok(())
        }
        VariantFormat::Struct(fields) => {
            let keys = variant_keys_type(variant, fields);
            let keyword = if fields.is_empty() {
                "_ ="
            } else {
                "var nested ="
            };
            writeln!(
                w,
                "{keyword} {container}.nestedContainer(keyedBy: {keys}.self, forKey: {key})"
            )?;
            write_struct_variant_fields_encode(w, fields, "nested", names)
        }
    }
}

/// Writes statements encoding a struct variant's bound fields into the keyed
/// container `container`.
fn write_struct_variant_fields_encode(
    w: &mut dyn IndentWrite,
    fields: &[Named<Format>],
    container: &str,
    names: &Names,
) -> io::Result<()> {
    for (field, payload) in fields.iter().zip(payloads(fields.len())) {
        let key = format!(".{}", field_name(&field.name));
        write_encode(
            w,
            &field.value,
            &payload,
            Slot::Keyed {
                container,
                key: &key,
            },
            0,
            names,
        )?;
    }
    Ok(())
}

/// Writes the `default:` case of a switch over a decoded tag.
fn write_unknown_tag(
    w: &mut dyn IndentWrite,
    name: &str,
    tag_key: &str,
    container: &str,
    names: &Names,
) -> io::Result<()> {
    writeln!(w, "default:")?;
    w.indent();
    writeln!(
        w,
        r#"throw {}.dataCorruptedError(forKey: {tag_key}, in: {container}, debugDescription: "Unknown variant \(tag) for {name}")"#,
        names.decoding_error,
    )?;
    w.unindent();
    Ok(())
}

/// Whether a newtype variant's payload can share an internally tagged object
/// with the tag: Rust accepts only a payload written as an object.
fn is_object_like(format: &Format) -> bool {
    matches!(
        unwrap_single(format),
        Format::TypeName(_) | Format::Map { .. }
    )
}

/// `{"tag": "Name", …fields}` — `#[facet(tag = "…")]`.
fn write_internal_enum(
    w: &mut dyn IndentWrite,
    name: &str,
    variants: &[&Named<VariantFormat>],
    tag: &str,
    names: &Names,
) -> io::Result<()> {
    let tag_key = json_key(tag);
    write_init_from(w, names, |w| {
        writeln!(
            w,
            "let tagContainer = try decoder.container(keyedBy: {JSON_KEY}.self)"
        )?;
        writeln!(
            w,
            "let tag = try tagContainer.decode({}.self, forKey: {tag_key})",
            names.string
        )?;
        write!(w, "switch tag ")?;
        with_block(w, Newlines::BOTH, |w| {
            w.unindent();
            for variant in variants {
                writeln!(w, "case {}:", literal(&variant.name))?;
                w.indent();
                match &variant.value {
                    VariantFormat::Variable(_) => {
                        unreachable!("placeholders should not get this far")
                    }
                    VariantFormat::Unit => writeln!(w, "self = .{}", case_name(&variant.name))?,
                    VariantFormat::NewType(format) if is_object_like(format) => {
                        write_construct(w, variant, |w, _, format| {
                            write_decode(w, format, Slot::Whole("decoder"), 0, names)
                        })?;
                    }
                    VariantFormat::NewType(_) | VariantFormat::Tuple(_) => writeln!(
                        w,
                        "throw {}.dataCorruptedError(forKey: {tag_key}, in: tagContainer, debugDescription: {})",
                        names.decoding_error,
                        literal(&unsupported_internal(name, &variant.name)),
                    )?,
                    VariantFormat::Struct(fields) => {
                        let keys = variant_keys_type(variant, fields);
                        if !fields.is_empty() {
                            writeln!(
                                w,
                                "let nested = try decoder.container(keyedBy: {keys}.self)"
                            )?;
                        }
                        write_struct_variant_construct(w, variant, fields, "nested", names)?;
                    }
                }
                w.unindent();
            }
            write_unknown_tag(w, name, &tag_key, "tagContainer", names)?;
            w.indent();
            Ok(())
        })
    })?;
    write_encode_to(w, names, |w| {
        write!(w, "switch self ")?;
        with_block(w, Newlines::BOTH, |w| {
            w.unindent();
            for variant in variants {
                let unsupported = match &variant.value {
                    VariantFormat::NewType(format) => !is_object_like(format),
                    VariantFormat::Tuple(_) => true,
                    _ => false,
                };
                if unsupported {
                    writeln!(w, "case .{}:", case_name(&variant.name))?;
                } else {
                    writeln!(w, "{}", case_pattern(variant))?;
                }
                w.indent();
                match &variant.value {
                    VariantFormat::Variable(_) => {
                        unreachable!("placeholders should not get this far")
                    }
                    VariantFormat::NewType(format) if !is_object_like(format) => {
                        write_unsupported_internal_encode(w, name, variant, names)?;
                    }
                    VariantFormat::Tuple(_) => {
                        write_unsupported_internal_encode(w, name, variant, names)?;
                    }
                    format => {
                        writeln!(
                            w,
                            "var tagContainer = encoder.container(keyedBy: {JSON_KEY}.self)"
                        )?;
                        writeln!(
                            w,
                            "try tagContainer.encode({}, forKey: {tag_key})",
                            literal(&variant.name)
                        )?;
                        match format {
                            // The payload's fields join the tag in its object.
                            VariantFormat::NewType(format) if is_native(format) => {
                                writeln!(w, "try payload0.encode(to: encoder)")?;
                            }
                            VariantFormat::NewType(format) => {
                                write_encode(
                                    w,
                                    format,
                                    "payload0",
                                    Slot::Whole("encoder"),
                                    0,
                                    names,
                                )?;
                            }
                            VariantFormat::Struct(fields) if !fields.is_empty() => {
                                writeln!(
                                    w,
                                    "var nested = encoder.container(keyedBy: {}.self)",
                                    variant_keys(variant)
                                )?;
                                write_struct_variant_fields_encode(w, fields, "nested", names)?;
                            }
                            _ => {}
                        }
                    }
                }
                w.unindent();
            }
            w.indent();
            Ok(())
        })
    })
}

fn unsupported_internal(name: &str, variant: &str) -> String {
    format!("{name}.{variant} cannot be internally tagged: its payload is not written as an object")
}

fn write_unsupported_internal_encode(
    w: &mut dyn IndentWrite,
    name: &str,
    variant: &Named<VariantFormat>,
    names: &Names,
) -> io::Result<()> {
    writeln!(
        w,
        "throw {}.invalidValue(self, .init(codingPath: encoder.codingPath, debugDescription: {}))",
        names.encoding_error,
        literal(&unsupported_internal(name, &variant.name)),
    )
}

/// `{"tag": "Name", "content": …}` — `#[facet(tag = "…", content = "…")]`.
fn write_adjacent_enum(
    w: &mut dyn IndentWrite,
    name: &str,
    variants: &[&Named<VariantFormat>],
    tag: &str,
    content: &str,
    names: &Names,
) -> io::Result<()> {
    let tag_key = json_key(tag);
    let content_key = json_key(content);
    write_init_from(w, names, |w| {
        writeln!(
            w,
            "let container = try decoder.container(keyedBy: {JSON_KEY}.self)"
        )?;
        writeln!(
            w,
            "let tag = try container.decode({}.self, forKey: {tag_key})",
            names.string
        )?;
        write!(w, "switch tag ")?;
        with_block(w, Newlines::BOTH, |w| {
            w.unindent();
            for variant in variants {
                writeln!(w, "case {}:", literal(&variant.name))?;
                w.indent();
                write_payload_decode(w, variant, "container", &content_key, names)?;
                w.unindent();
            }
            write_unknown_tag(w, name, &tag_key, "container", names)?;
            w.indent();
            Ok(())
        })
    })?;
    write_encode_to(w, names, |w| {
        write!(w, "switch self ")?;
        with_block(w, Newlines::BOTH, |w| {
            w.unindent();
            for variant in variants {
                writeln!(w, "{}", case_pattern(variant))?;
                w.indent();
                writeln!(
                    w,
                    "var container = encoder.container(keyedBy: {JSON_KEY}.self)"
                )?;
                writeln!(
                    w,
                    "try container.encode({}, forKey: {tag_key})",
                    literal(&variant.name)
                )?;
                write_payload_encode(w, variant, "container", &content_key, names)?;
                w.unindent();
            }
            w.indent();
            Ok(())
        })
    })
}

// ---------------------------------------------------------------------------
// Serialization wrappers
// ---------------------------------------------------------------------------

fn write_wrappers(w: &mut dyn IndentWrite, name: &str, names: &Names) -> io::Result<()> {
    let byte = naming::builtin("UInt8", names.config);
    writeln!(w)?;
    writedoc!(
        w,
        r"
        public func jsonSerialize() throws -> [{byte}] {{
            return try Serde.jsonSerialize(self)
        }}

        public static func jsonDeserialize(input: [{byte}]) throws -> {name} {{
            return try Serde.jsonDeserialize({name}.self, from: input)
        }}
        "
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::indent::IndentedWriter;
    use crate::generation::{CodeGeneratorConfig, Feature};
    use std::collections::BTreeSet;

    fn make_config(features: &[Feature]) -> CodeGeneratorConfig {
        let mut cfg = CodeGeneratorConfig::new("com.example".to_string());
        cfg.features = features.iter().copied().collect::<BTreeSet<_>>();
        cfg
    }

    fn render_body(format: &ContainerFormat, name: &str) -> String {
        use crate::generation::Container;
        use crate::reflection::format::QualifiedTypeName;

        let cfg = make_config(&[]);
        let plugin = &JsonPlugin as &dyn EmitterPlugin<Swift>;
        let name = QualifiedTypeName::root(name.to_string());
        let container = Container {
            name: &name,
            format,
        };
        let ctx = EmitContext::top_level(&container, &cfg);
        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin
                .type_body(&mut w as &mut dyn IndentWrite, &ctx)
                .unwrap();
        }
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn imports_serde() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<Swift>;
        assert_eq!(plugin.imports(&make_config(&[])), vec!["Serde"]);
        // The emitter imports `Foundation` for `UUID` (#191).
        assert_eq!(
            plugin.imports(&make_config(&[Feature::Uuid])),
            vec!["Serde"]
        );
    }

    #[test]
    fn runtime_files_are_the_json_subset_of_serde() {
        let plugin = &JsonPlugin as &dyn EmitterPlugin<Swift>;
        let paths: Vec<_> = plugin
            .runtime_files()
            .into_iter()
            .map(|f| f.relative_path)
            .collect();
        assert_eq!(
            paths,
            [
                "Sources/Serde/Indirect.swift",
                "Sources/Serde/Int128.swift",
                "Sources/Serde/UInt128.swift",
                "Sources/Serde/JsonCoding.swift",
            ]
        );
    }

    #[test]
    fn conforms_to_codable() {
        use crate::generation::Container;
        use crate::reflection::format::{Doc, QualifiedTypeName};

        let plugin = &JsonPlugin as &dyn EmitterPlugin<Swift>;
        let config = CodeGeneratorConfig::new("test".to_string());
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);
        assert!(plugin.has_type_body(&ctx));
        assert_eq!(plugin.type_conformances(&ctx), vec!["Codable"]);
    }

    #[test]
    fn struct_of_native_fields_synthesizes_coding() {
        use crate::reflection::format::Doc;

        let fields = vec![
            Named::new(&Format::Str, "name".to_string()),
            Named::new(&Format::I32, "max_age".to_string()),
        ];
        let output = render_body(&ContainerFormat::Struct(fields, Doc::default()), "MyStruct");
        insta::assert_snapshot!(output, @r#"
        enum CodingKeys: String, CodingKey {
            case name
            case maxAge = "max_age"
        }

        public func jsonSerialize() throws -> [UInt8] {
            return try Serde.jsonSerialize(self)
        }

        public static func jsonDeserialize(input: [UInt8]) throws -> MyStruct {
            return try Serde.jsonDeserialize(MyStruct.self, from: input)
        }
        "#);
    }

    #[test]
    fn literals_are_escaped() {
        assert_eq!(literal(r#"a"b\c"#), r#""a\"b\\c""#);
    }
}
