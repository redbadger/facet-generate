#![allow(clippy::too_many_lines)]
use std::{
    collections::BTreeMap,
    io::{Result, Write},
};

use heck::ToLowerCamelCase as _;
use indoc::writedoc;

use crate::{
    generation::{Container, Emitter, Encoding, indent::IndentWrite},
    reflection::format::{
        ContainerFormat, Doc, Format, Named, Namespace, QualifiedTypeName, VariantFormat,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct Swift {
    encoding: Encoding,
}

impl Swift {
    pub fn new(encoding: Encoding) -> Self {
        Self { encoding }
    }
}

enum Usage {
    Field,
    Parameter,
    Argument,
    Assignment,
    Serialize { receiver: String },
    Deserialize { receiver: String },
}

impl Emitter<Swift> for Container<'_> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: Swift) -> Result<()> {
        let Container {
            name: QualifiedTypeName { namespace: _, name },
            format,
        } = self;
        match format {
            ContainerFormat::UnitStruct(doc) => struct_(w, name, &[], doc, lang),
            ContainerFormat::NewTypeStruct(format, doc) => struct_(
                w,
                name,
                &[&Named::new(format, "value".to_string())],
                doc,
                lang,
            ),
            ContainerFormat::TupleStruct(formats, doc) => {
                let formats = named(formats, "field");
                struct_(w, name, &formats.iter().collect::<Vec<_>>(), doc, lang)
            }
            ContainerFormat::Struct(nameds, doc) => {
                struct_(w, name, &nameds.iter().collect::<Vec<_>>(), doc, lang)
            }
            ContainerFormat::Enum(variants, doc) => enum_(w, name, variants, doc, lang),
        }
    }
}

impl Emitter<Swift> for Format {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: Swift) -> Result<()> {
        match &self {
            Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Format::TypeName(qualified_type_name) => {
                write!(
                    w,
                    "{ty}",
                    ty = qualified_type_name
                        .format(|ns| heck::AsUpperCamelCase(ns).to_string(), ".")
                )
            }
            Format::Unit => write!(w, "Unit"),
            Format::Bool => write!(w, "Bool"),
            Format::I8 => write!(w, "Int8"),
            Format::I16 => write!(w, "Int16"),
            Format::I32 => write!(w, "Int32"),
            Format::I64 => write!(w, "Int64"),
            Format::I128 => write!(w, "Int128"),
            Format::U8 => write!(w, "UInt8"),
            Format::U16 => write!(w, "UInt16"),
            Format::U32 => write!(w, "UInt32"),
            Format::U64 => write!(w, "UInt64"),
            Format::U128 => write!(w, "UInt128"),
            Format::F32 => write!(w, "Float"),
            Format::F64 => write!(w, "Double"),
            Format::Char => write!(w, "Character"),
            Format::Str => write!(w, "String"),
            Format::Bytes => write!(w, "[UInt8]"),

            Format::Option(format) => {
                format.write(w, lang)?;
                write!(w, "?")
            }
            Format::Seq(format)
            | Format::TupleArray {
                content: format,
                size: _,
            } => {
                write!(w, "[")?;
                format.write(w, lang)?;
                write!(w, "]")
            }
            Format::Set(format) => {
                write!(w, "Set<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Format::Map { key, value } => {
                write!(w, "[")?;
                key.write(w, lang)?;
                write!(w, ": ")?;
                value.write(w, lang)?;
                write!(w, "]")
            }
            Format::Tuple(formats) => {
                let len = formats.len();
                if len == 1 {
                    // A single-element tuple is just the element itself
                    formats[0].write(w, lang)
                } else {
                    // Use TupleN<...> types from the Serde runtime because
                    // native Swift tuples don't conform to Hashable
                    write!(w, "Tuple{len}<")?;
                    for (i, format) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        format.write(w, lang)?;
                    }
                    write!(w, ">")
                }
            }
        }
    }
}

impl Emitter<Swift> for (&Named<Format>, Usage) {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: Swift) -> Result<()> {
        let (Named { name, doc, value }, usage) = self;
        let name = &name.to_lower_camel_case();

        match usage {
            Usage::Field => {
                doc.write(w, lang)?;
                write!(w, "@Indirect public var {name}: ")?;
                value.write(w, lang)?;
                writeln!(w)
            }
            Usage::Parameter => {
                write!(w, "{name}: ")?;
                value.write(w, lang)
            }
            Usage::Argument => {
                write!(w, "{name}: {name}")
            }
            Usage::Assignment => writeln!(w, "self.{name} = {name}"),
            Usage::Serialize { receiver } => match value {
                Format::Variable(_) => {
                    unreachable!("placeholders should not get this far")
                }
                Format::TypeName(_) => {
                    writeln!(w, "try {receiver}.{name}.serialize(serializer: serializer)")
                }
                Format::Tuple(formats) if !lang.encoding.is_bincode() => {
                    push_serializer(w)?;
                    let formats = named(formats, "");
                    for format in formats {
                        (
                            &format,
                            Usage::Serialize {
                                receiver: name.clone(),
                            },
                        )
                            .write(w, lang)?;
                    }
                    pop_serializer(w)
                }
                Format::Option(_)
                | Format::Seq(_)
                | Format::Set(_)
                | Format::Map { .. }
                | Format::Tuple(_)
                | Format::TupleArray { .. } => {
                    let suffix = format_suffix(value);
                    writeln!(
                        w,
                        "try serialize_{suffix}(value: {receiver}.{name}, serializer: serializer)"
                    )
                }
                primitive => {
                    let t = format!("{primitive:?}").to_lowercase();
                    writeln!(w, "try serializer.serialize_{t}(value: {receiver}.{name})")
                }
            },
            Usage::Deserialize { receiver: _ } => match value {
                Format::Variable(_) => {
                    unreachable!("placeholders should not get this far")
                }
                Format::TypeName(qualified_type_name) => {
                    let type_name = &qualified_type_name.name;
                    writeln!(
                        w,
                        "let {name} = try {type_name}.deserialize(deserializer: deserializer)"
                    )
                }
                Format::Tuple(formats) if !lang.encoding.is_bincode() => {
                    push_deserializer(w)?;
                    let formats = named(formats, name);
                    for (i, format) in formats.iter().enumerate() {
                        (
                            format,
                            Usage::Deserialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(w, lang)?;
                    }
                    write!(w, "let {name} = (")?;
                    for (i, format) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "{}", format.name)?;
                    }
                    writeln!(w, ")")?;
                    pop_deserializer(w)
                }
                Format::Option(_)
                | Format::Seq(_)
                | Format::Set(_)
                | Format::Map { .. }
                | Format::Tuple(_)
                | Format::TupleArray { .. } => {
                    let suffix = format_suffix(value);
                    writeln!(
                        w,
                        "let {name} = try deserialize_{suffix}(deserializer: deserializer)"
                    )
                }
                primitive => {
                    let t = format!("{primitive:?}").to_lowercase();
                    writeln!(w, "let {name} = try deserializer.deserialize_{t}()")
                }
            },
        }
    }
}

impl Emitter<Swift> for (&Named<VariantFormat>, Usage) {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: Swift) -> Result<()> {
        let (
            Named {
                name,
                doc,
                value: format,
            },
            usage,
        ) = self;
        let name = name.to_lower_camel_case();

        doc.write(w, lang)?;

        match usage {
            Usage::Field => match format {
                VariantFormat::Variable(_variable) => {
                    unreachable!("placeholders should not get this far")
                }
                VariantFormat::Unit => {
                    writeln!(w, "case {name}")
                }
                VariantFormat::NewType(format) => {
                    write!(w, "case {name}(")?;
                    format.write(w, lang)?;
                    writeln!(w, ")")
                }
                VariantFormat::Tuple(formats) => {
                    write!(w, "case {name}(")?;
                    for (i, format) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        format.write(w, lang)?;
                    }
                    writeln!(w, ")")
                }
                VariantFormat::Struct(nameds) => {
                    write!(w, "case {name}(")?;
                    for (i, format) in nameds.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        (format, Usage::Parameter).write(w, lang)?;
                    }
                    writeln!(w, ")")
                }
            },
            Usage::Parameter | Usage::Argument | Usage::Assignment => Ok(()),
            Usage::Serialize { receiver: index } => {
                match format {
                    VariantFormat::Variable(_) => {
                        unreachable!("placeholders should not get this far")
                    }
                    VariantFormat::Unit => {
                        writeln!(w, "case .{name}:")?;
                        w.indent();
                        writeln!(w, "try serializer.serialize_variant_index(value: {index})")?;
                        w.unindent();
                    }
                    VariantFormat::NewType(fmt) => {
                        writeln!(w, "case .{name}(let x):")?;
                        w.indent();
                        writeln!(w, "try serializer.serialize_variant_index(value: {index})")?;
                        write_format_serialize(w, fmt, "x")?;
                        w.unindent();
                    }
                    VariantFormat::Tuple(formats) => {
                        write!(w, "case .{name}(")?;
                        for (i, _) in formats.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            write!(w, "let x{i}")?;
                        }
                        writeln!(w, "):")?;
                        w.indent();
                        writeln!(w, "try serializer.serialize_variant_index(value: {index})")?;
                        for (i, fmt) in formats.iter().enumerate() {
                            write_format_serialize(w, fmt, &format!("x{i}"))?;
                        }
                        w.unindent();
                    }
                    VariantFormat::Struct(nameds) => {
                        write!(w, "case .{name}(")?;
                        for (i, named) in nameds.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            let field_name = named.name.to_lower_camel_case();
                            write!(w, "let {field_name}")?;
                        }
                        writeln!(w, "):")?;
                        w.indent();
                        writeln!(w, "try serializer.serialize_variant_index(value: {index})")?;
                        for named in nameds {
                            let field_name = named.name.to_lower_camel_case();
                            write_format_serialize(w, &named.value, &field_name)?;
                        }
                        w.unindent();
                    }
                }
                Ok(())
            }
            Usage::Deserialize { receiver: index } => {
                writeln!(w, "case {index}:")?;
                w.indent();
                match format {
                    VariantFormat::Variable(_) => {
                        unreachable!("placeholders should not get this far")
                    }
                    VariantFormat::Unit => {
                        pop_deserializer(w)?;
                        writeln!(w, "return .{name}")?;
                    }
                    VariantFormat::NewType(fmt) => {
                        write_format_deserialize(w, fmt, "x")?;
                        pop_deserializer(w)?;
                        writeln!(w, "return .{name}(x)")?;
                    }
                    VariantFormat::Tuple(formats) => {
                        for (i, fmt) in formats.iter().enumerate() {
                            write_format_deserialize(w, fmt, &format!("x{i}"))?;
                        }
                        pop_deserializer(w)?;
                        write!(w, "return .{name}(")?;
                        for (i, _) in formats.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            write!(w, "x{i}")?;
                        }
                        writeln!(w, ")")?;
                    }
                    VariantFormat::Struct(nameds) => {
                        for named in nameds {
                            let field_name = named.name.to_lower_camel_case();
                            write_format_deserialize(w, &named.value, &field_name)?;
                        }
                        pop_deserializer(w)?;
                        write!(w, "return .{name}(")?;
                        for (i, named) in nameds.iter().enumerate() {
                            if i > 0 {
                                write!(w, ", ")?;
                            }
                            let field_name = named.name.to_lower_camel_case();
                            write!(w, "{field_name}: {field_name}")?;
                        }
                        writeln!(w, ")")?;
                    }
                }
                w.unindent();
                Ok(())
            }
        }
    }
}

impl Emitter<Swift> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: Swift) -> Result<()> {
        if lang.encoding.is_bincode() {
            return Ok(());
        }
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }

        Ok(())
    }
}

fn struct_<W: IndentWrite>(
    w: &mut W,
    name: &str,
    fields: &[&Named<Format>],
    doc: &Doc,
    lang: Swift,
) -> Result<()> {
    doc.write(w, lang)?;

    write!(w, "public struct {name}: Hashable ")?;

    w.start_block()?;
    for field in fields {
        (*field, Usage::Field).write(w, lang)?;
    }

    if !fields.is_empty() || lang.encoding.is_bincode() {
        writeln!(w)?;
    }

    write!(w, "public init(")?;
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(w, ", ")?;
        }
        (*field, Usage::Parameter).write(w, lang)?;
    }
    write!(w, ") ")?;
    w.start_block()?;
    for field in fields {
        (*field, Usage::Assignment).write(w, lang)?;
    }
    w.end_block()?;

    match lang.encoding {
        Encoding::None => {}
        Encoding::Json => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            w.start_block()?;
            push_serializer(w)?;
            for field in fields {
                (
                    *field,
                    Usage::Serialize {
                        receiver: "self".to_string(),
                    },
                )
                    .write(w, lang)?;
            }
            pop_serializer(w)?;
            w.end_block()?;
            write_json_serialize(w)?;
            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            w.start_block()?;
            push_deserializer(w)?;
            for field in fields {
                (
                    *field,
                    Usage::Deserialize {
                        receiver: "self".to_string(),
                    },
                )
                    .write(w, lang)?;
            }
            pop_deserializer(w)?;
            write!(w, "return {name}(")?;
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                (*field, Usage::Argument).write(w, lang)?;
            }
            writeln!(w, ")")?;
            w.end_block()?;
            write_json_deserialize(w, name)?;
        }
        Encoding::Bincode => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            w.start_block()?;
            push_serializer(w)?;
            for field in fields {
                (
                    *field,
                    Usage::Serialize {
                        receiver: "self".to_string(),
                    },
                )
                    .write(w, lang)?;
            }
            pop_serializer(w)?;
            w.end_block()?;
            write_bincode_serialize(w)?;
            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            w.start_block()?;
            push_deserializer(w)?;
            for field in fields {
                (
                    *field,
                    Usage::Deserialize {
                        receiver: "self".to_string(),
                    },
                )
                    .write(w, lang)?;
            }
            pop_deserializer(w)?;
            write!(w, "return {name}.init(")?;
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                (*field, Usage::Argument).write(w, lang)?;
            }
            writeln!(w, ")")?;
            w.end_block()?;
            write_bincode_deserialize(w, name)?;
        }
        Encoding::Bcs => todo!(),
    }

    w.end_block()?;

    Ok(())
}

fn enum_<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    lang: Swift,
) -> Result<()> {
    doc.write(w, lang)?;

    write!(w, "indirect public enum {name}: Hashable ")?;

    w.start_block()?;

    let variants = variants.values().collect::<Vec<_>>();

    for format in &variants {
        (*format, Usage::Field).write(w, lang)?;
    }

    match lang.encoding {
        Encoding::None => {}
        Encoding::Json => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            w.start_block()?;
            push_serializer(w)?;
            write!(w, "switch self ")?;
            w.start_block()?;
            w.unindent(); // in Swift, `case` is not indented
            for (i, variant) in variants.iter().enumerate() {
                (
                    &variant.without_docs(),
                    Usage::Serialize {
                        receiver: i.to_string(),
                    },
                )
                    .write(w, lang)?;
            }
            w.indent();
            w.end_block()?;
            pop_serializer(w)?;
            w.end_block()?;
            write_json_serialize(w)?;

            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            w.start_block()?;
            writeln!(
                w,
                "let index = try deserializer.deserialize_variant_index()"
            )?;
            push_deserializer(w)?;
            write!(w, "switch index ")?;
            w.start_block()?;
            w.unindent(); // in Swift, `case` is not indented
            for (i, variant) in variants.iter().enumerate() {
                (
                    &variant.without_docs(),
                    Usage::Deserialize {
                        receiver: i.to_string(),
                    },
                )
                    .write(w, lang)?;
            }
            writeln!(
                w,
                r#"default: throw DeserializationError.invalidInput(issue: "Unknown variant index for {name}: \(index)")"#
            )?;
            w.indent();
            w.end_block()?;
            w.end_block()?;
            write_json_deserialize(w, name)?;
        }
        Encoding::Bincode => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            w.start_block()?;
            push_serializer(w)?;
            write!(w, "switch self ")?;
            w.start_block()?;
            w.unindent(); // in Swift, `case` is not indented
            for (i, variant) in variants.iter().enumerate() {
                (
                    &variant.without_docs(),
                    Usage::Serialize {
                        receiver: i.to_string(),
                    },
                )
                    .write(w, lang)?;
            }
            w.indent();
            w.end_block()?;
            pop_serializer(w)?;
            w.end_block()?;
            write_bincode_serialize(w)?;

            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            w.start_block()?;
            writeln!(
                w,
                "let index = try deserializer.deserialize_variant_index()"
            )?;
            push_deserializer(w)?;
            write!(w, "switch index ")?;
            w.start_block()?;
            w.unindent(); // in Swift, `case` is not indented
            for (i, variant) in variants.iter().enumerate() {
                (
                    &variant.without_docs(),
                    Usage::Deserialize {
                        receiver: i.to_string(),
                    },
                )
                    .write(w, lang)?;
            }
            writeln!(
                w,
                r#"default: throw DeserializationError.invalidInput(issue: "Unknown variant index for {name}: \(index)")"#
            )?;
            w.indent();
            w.end_block()?;
            w.end_block()?;
            write_bincode_deserialize(w, name)?;
        }
        Encoding::Bcs => todo!(),
    }

    w.end_block()
}

fn write_bincode_serialize<W: IndentWrite>(w: &mut W) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r"
        public func bincodeSerialize() throws -> [UInt8] {{
            let serializer = BincodeSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }}
        "
    )
}

fn write_bincode_deserialize<W: IndentWrite>(w: &mut W, name: &str) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r#"
        public static func bincodeDeserialize(input: [UInt8]) throws -> {name} {{
            let deserializer = BincodeDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {{
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }}
            return obj
        }}
        "#
    )
}

fn write_json_serialize<W: IndentWrite>(w: &mut W) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r"
        public func jsonSerialize() throws -> [UInt8] {{
            let serializer = JsonSerializer.init();
            try self.serialize(serializer: serializer)
            return serializer.get_bytes()
        }}
        "
    )
}

fn write_json_deserialize<W: IndentWrite>(w: &mut W, name: &str) -> Result<()> {
    writeln!(w)?;
    writedoc!(
        w,
        r#"
        public static func jsonDeserialize(input: [UInt8]) throws -> {name} {{
            let deserializer = JsonDeserializer.init(input: input);
            let obj = try deserialize(deserializer: deserializer)
            if deserializer.get_buffer_offset() < input.count {{
                throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
            }}
            return obj
        }}
        "#
    )
}

fn write_format_serialize<W: Write>(w: &mut W, format: &Format, value_expr: &str) -> Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(_) => {
            writeln!(w, "try {value_expr}.serialize(serializer: serializer)")
        }
        Format::Option(_)
        | Format::Seq(_)
        | Format::Set(_)
        | Format::Map { .. }
        | Format::Tuple(_)
        | Format::TupleArray { .. } => {
            let suffix = format_suffix(format);
            writeln!(
                w,
                "try serialize_{suffix}(value: {value_expr}, serializer: serializer)"
            )
        }
        primitive => {
            let t = format!("{primitive:?}").to_lowercase();
            writeln!(w, "try serializer.serialize_{t}(value: {value_expr})")
        }
    }
}

fn write_format_deserialize<W: Write>(w: &mut W, format: &Format, var: &str) -> Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qtn) => {
            let type_name = &qtn.name;
            writeln!(
                w,
                "let {var} = try {type_name}.deserialize(deserializer: deserializer)"
            )
        }
        Format::Option(_)
        | Format::Seq(_)
        | Format::Set(_)
        | Format::Map { .. }
        | Format::Tuple(_)
        | Format::TupleArray { .. } => {
            let suffix = format_suffix(format);
            writeln!(
                w,
                "let {var} = try deserialize_{suffix}(deserializer: deserializer)"
            )
        }
        primitive => {
            let t = format!("{primitive:?}").to_lowercase();
            writeln!(w, "let {var} = try deserializer.deserialize_{t}()")
        }
    }
}

fn format_suffix(format: &Format) -> String {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qtn) => match &qtn.namespace {
            Namespace::Named(ns) => format!("{ns}_{}", qtn.name),
            Namespace::Root => qtn.name.clone(),
        },
        Format::Option(inner) => format!("option_{}", format_suffix(inner)),
        Format::Seq(inner) => format!("vector_{}", format_suffix(inner)),
        Format::Set(inner) => format!("set_{}", format_suffix(inner)),
        Format::Map { key, value } => {
            format!("map_{}_to_{}", format_suffix(key), format_suffix(value))
        }
        Format::Tuple(formats) => {
            let parts: Vec<String> = formats.iter().map(format_suffix).collect();
            format!("tuple{}_{}", formats.len(), parts.join("_"))
        }
        Format::TupleArray { content, size } => {
            format!("array{}_{}_array", size, format_suffix(content))
        }
        // Primitive types: the Debug name lowercased matches the suffix
        // e.g. Bool -> "bool", I32 -> "i32", Str -> "str", etc.
        primitive => format!("{primitive:?}").to_lowercase(),
    }
}

fn push_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try serializer.increase_container_depth()")
}

fn pop_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try serializer.decrease_container_depth()")
}

fn push_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try deserializer.increase_container_depth()")
}

fn pop_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "try deserializer.decrease_container_depth()")
}

fn named<Format: Clone>(formats: &[Format], prefix: &str) -> Vec<Named<Format>> {
    formats
        .iter()
        .enumerate()
        .map(|(i, f)| Named::new(f, format!("{prefix}{i}")))
        .collect()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bincode;
#[cfg(test)]
mod tests_json;
