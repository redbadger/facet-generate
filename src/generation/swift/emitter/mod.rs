#![allow(clippy::too_many_lines)]
use std::{
    collections::BTreeMap,
    io::{Result, Write},
};

use heck::ToLowerCamelCase as _;
use indoc::writedoc;

use heck::ToUpperCamelCase as _;

use crate::{
    generation::{
        CodeGeneratorConfig, Container, Emitter, Encoding, Feature, indent::IndentWrite,
        module::Module,
    },
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName, VariantFormat},
};

const FEATURE_LIST_OF_T: &[u8] = include_bytes!("features/ListOfT.swift");
const FEATURE_MAP_OF_T: &[u8] = include_bytes!("features/MapOfT.swift");
const FEATURE_OPTION_OF_T: &[u8] = include_bytes!("features/OptionOfT.swift");
const FEATURE_SET_OF_T: &[u8] = include_bytes!("features/SetOfT.swift");
const FEATURE_TUPLE_ARRAY: &[u8] = include_bytes!("features/TupleArray.swift");

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

impl Emitter<Swift> for Module {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: Swift) -> Result<()> {
        let CodeGeneratorConfig {
            encoding, features, ..
        } = self.config();

        let mut imports = vec![];
        if !encoding.is_none() {
            imports.push("Serde".to_string());
        }
        for ns in self.config().external_definitions.keys() {
            imports.push(ns.to_upper_camel_case());
        }
        imports.sort();
        imports.dedup();

        for import in &imports {
            writeln!(w, "import {import}")?;
        }

        if encoding.is_none() {
            return Ok(());
        }

        for feature in features {
            match feature {
                Feature::OptionOfT => {
                    writeln!(w)?;
                    w.write_all(FEATURE_OPTION_OF_T)?;
                }
                Feature::ListOfT => {
                    writeln!(w)?;
                    w.write_all(FEATURE_LIST_OF_T)?;
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

        Ok(())
    }
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
            Format::Unit => {
                if lang.encoding.is_none() {
                    write!(w, "Void")
                } else {
                    write!(w, "Unit")
                }
            }
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
                } else if lang.encoding.is_none() {
                    // Use native Swift tuples when no serialization is needed
                    write!(w, "(")?;
                    for (i, format) in formats.iter().enumerate() {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        format.write(w, lang)?;
                    }
                    write!(w, ")")
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
                if lang.encoding.is_none() {
                    write!(w, "public var {name}: ")?;
                } else {
                    write!(w, "@Indirect public var {name}: ")?;
                }
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
                _ => write_format_serialize(w, value, &format!("{receiver}.{name}")),
            },
            Usage::Deserialize { receiver: _ } => match value {
                Format::Variable(_) => {
                    unreachable!("placeholders should not get this far")
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
                _ => write_format_deserialize(w, value, name),
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

    if lang.encoding.is_none() {
        write!(w, "public struct {name} ")?;
    } else {
        write!(w, "public struct {name}: Hashable ")?;
    }
    let mut w = w.blockln()?;

    for field in fields {
        (*field, Usage::Field).write(&mut w, lang)?;
    }

    if !fields.is_empty() || lang.encoding.is_bincode() {
        writeln!(w)?;
    }

    write!(w, "public init(")?;
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(w, ", ")?;
        }
        (*field, Usage::Parameter).write(&mut w, lang)?;
    }
    write!(w, ") ")?;
    {
        let mut w = w.blockln()?;
        for field in fields {
            (*field, Usage::Assignment).write(&mut w, lang)?;
        }
    }

    match lang.encoding {
        Encoding::None => {}
        Encoding::Json => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            {
                let mut w = w.blockln()?;
                push_serializer(&mut w)?;
                for field in fields {
                    (
                        *field,
                        Usage::Serialize {
                            receiver: "self".to_string(),
                        },
                    )
                        .write(&mut w, lang)?;
                }
                pop_serializer(&mut w)?;
            }
            write_json_serialize(&mut w)?;
            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            {
                let mut w = w.blockln()?;
                push_deserializer(&mut w)?;
                for field in fields {
                    (
                        *field,
                        Usage::Deserialize {
                            receiver: "self".to_string(),
                        },
                    )
                        .write(&mut w, lang)?;
                }
                pop_deserializer(&mut w)?;
                write!(w, "return {name}(")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    (*field, Usage::Argument).write(&mut w, lang)?;
                }
                writeln!(w, ")")?;
            }
            write_json_deserialize(&mut w, name)?;
        }
        Encoding::Bincode => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            {
                let mut w = w.blockln()?;
                push_serializer(&mut w)?;
                for field in fields {
                    (
                        *field,
                        Usage::Serialize {
                            receiver: "self".to_string(),
                        },
                    )
                        .write(&mut w, lang)?;
                }
                pop_serializer(&mut w)?;
            }
            write_bincode_serialize(&mut w)?;
            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            {
                let mut w = w.blockln()?;
                push_deserializer(&mut w)?;
                for field in fields {
                    (
                        *field,
                        Usage::Deserialize {
                            receiver: "self".to_string(),
                        },
                    )
                        .write(&mut w, lang)?;
                }
                pop_deserializer(&mut w)?;
                write!(w, "return {name}(")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    (*field, Usage::Argument).write(&mut w, lang)?;
                }
                writeln!(w, ")")?;
            }
            write_bincode_deserialize(&mut w, name)?;
        }
        Encoding::Bcs => todo!(),
    }

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

    if lang.encoding.is_none() {
        write!(w, "indirect public enum {name} ")?;
    } else {
        write!(w, "indirect public enum {name}: Hashable ")?;
    }
    let mut w = w.blockln()?;

    let variants = variants.values().collect::<Vec<_>>();

    for format in &variants {
        (*format, Usage::Field).write(&mut w, lang)?;
    }

    match lang.encoding {
        Encoding::None => {}
        Encoding::Json => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            {
                let mut w = w.blockln()?;
                push_serializer(&mut w)?;
                write!(w, "switch self ")?;
                {
                    let mut w = w.blockln()?;
                    w.unindent();
                    for (i, variant) in variants.iter().enumerate() {
                        (
                            &variant.without_docs(),
                            Usage::Serialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(&mut w, lang)?;
                    }
                    w.indent();
                }
                pop_serializer(&mut w)?;
            }
            write_json_serialize(&mut w)?;

            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            {
                let mut w = w.blockln()?;
                writeln!(
                    w,
                    "let index = try deserializer.deserialize_variant_index()"
                )?;
                push_deserializer(&mut w)?;
                write!(w, "switch index ")?;
                {
                    let mut w = w.blockln()?;
                    w.unindent();
                    for (i, variant) in variants.iter().enumerate() {
                        (
                            &variant.without_docs(),
                            Usage::Deserialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(&mut w, lang)?;
                    }
                    writeln!(
                        w,
                        r#"default: throw DeserializationError.invalidInput(issue: "Unknown variant index for {name}: \(index)")"#
                    )?;
                    w.indent();
                }
            }
            write_json_deserialize(&mut w, name)?;
        }
        Encoding::Bincode => {
            writeln!(w)?;
            write!(
                w,
                "public func serialize<S: Serializer>(serializer: S) throws "
            )?;
            {
                let mut w = w.blockln()?;
                push_serializer(&mut w)?;
                write!(w, "switch self ")?;
                {
                    let mut w = w.blockln()?;
                    w.unindent();
                    for (i, variant) in variants.iter().enumerate() {
                        (
                            &variant.without_docs(),
                            Usage::Serialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(&mut w, lang)?;
                    }
                    w.indent();
                }
                pop_serializer(&mut w)?;
            }
            write_bincode_serialize(&mut w)?;

            writeln!(w)?;
            write!(
                w,
                "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
            )?;
            {
                let mut w = w.blockln()?;
                writeln!(
                    w,
                    "let index = try deserializer.deserialize_variant_index()"
                )?;
                push_deserializer(&mut w)?;
                write!(w, "switch index ")?;
                {
                    let mut w = w.blockln()?;
                    w.unindent();
                    for (i, variant) in variants.iter().enumerate() {
                        (
                            &variant.without_docs(),
                            Usage::Deserialize {
                                receiver: i.to_string(),
                            },
                        )
                            .write(&mut w, lang)?;
                    }
                    writeln!(
                        w,
                        r#"default: throw DeserializationError.invalidInput(issue: "Unknown variant index for {name}: \(index)")"#
                    )?;
                    w.indent();
                }
            }
            write_bincode_deserialize(&mut w, name)?;
        }
        Encoding::Bcs => todo!(),
    }

    Ok(())
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

fn write_format_serialize<W: IndentWrite>(
    w: &mut W,
    format: &Format,
    value_expr: &str,
) -> Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(_) => {
            writeln!(w, "try {value_expr}.serialize(serializer: serializer)")
        }
        Format::Option(inner) => {
            write!(
                w,
                "try serializeOption(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block()?;
                writeln!(w, " value, serializer in")?;
                write_format_serialize(&mut w, inner, "value")
            }
        }
        Format::Seq(inner) => {
            write!(
                w,
                "try serializeArray(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block()?;
                writeln!(w, " item, serializer in")?;
                write_format_serialize(&mut w, inner, "item")
            }
        }
        Format::Set(inner) => {
            write!(
                w,
                "try serializeSet(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block()?;
                writeln!(w, " item, serializer in")?;
                write_format_serialize(&mut w, inner, "item")
            }
        }
        Format::Map { key, value } => {
            write!(
                w,
                "try serializeMap(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block()?;
                writeln!(w, " key, value, serializer in")?;
                write_format_serialize(&mut w, key, "key")?;
                write_format_serialize(&mut w, value, "value")
            }
        }
        Format::Tuple(formats) => {
            for (i, fmt) in formats.iter().enumerate() {
                write_format_serialize(w, fmt, &format!("{value_expr}.field{i}"))?;
            }
            Ok(())
        }
        Format::TupleArray { content, .. } => {
            write!(
                w,
                "try serializeTupleArray(value: {value_expr}, serializer: serializer) "
            )?;
            {
                let mut w = w.block()?;
                writeln!(w, " item, serializer in")?;
                write_format_serialize(&mut w, content, "item")
            }
        }
        primitive => {
            let t = format!("{primitive:?}").to_lowercase();
            writeln!(w, "try serializer.serialize_{t}(value: {value_expr})")
        }
    }
}

fn write_format_deserialize<W: IndentWrite>(w: &mut W, format: &Format, var: &str) -> Result<()> {
    match format {
        Format::Tuple(formats) if formats.len() > 1 => {
            for (i, fmt) in formats.iter().enumerate() {
                write_format_deserialize(w, fmt, &format!("{var}Field{i}"))?;
            }
            write!(w, "let {var} = Tuple{}(", formats.len())?;
            for i in 0..formats.len() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write!(w, "{var}Field{i}")?;
            }
            writeln!(w, ")")
        }
        _ => {
            write!(w, "let {var} = ")?;
            write_deserialize_expr(w, format)?;
            writeln!(w)
        }
    }
}

/// Writes a deserialization expression (no `let` prefix, no trailing newline).
fn write_deserialize_expr<W: IndentWrite>(w: &mut W, format: &Format) -> Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qtn) => {
            let type_name = &qtn.format(|ns| heck::AsUpperCamelCase(ns).to_string(), ".");
            write!(w, "try {type_name}.deserialize(deserializer: deserializer)")
        }
        Format::Option(inner) => {
            writeln!(
                w,
                "try deserializeOption(deserializer: deserializer) {{ deserializer in"
            )?;
            w.indent();
            write_deserialize_expr(w, inner)?;
            writeln!(w)?;
            w.unindent();
            write!(w, "}}")
        }
        Format::Seq(inner) => {
            writeln!(
                w,
                "try deserializeArray(deserializer: deserializer) {{ deserializer in"
            )?;
            w.indent();
            write_deserialize_expr(w, inner)?;
            writeln!(w)?;
            w.unindent();
            write!(w, "}}")
        }
        Format::Set(inner) => {
            writeln!(
                w,
                "try deserializeSet(deserializer: deserializer) {{ deserializer in"
            )?;
            w.indent();
            write_deserialize_expr(w, inner)?;
            writeln!(w)?;
            w.unindent();
            write!(w, "}}")
        }
        Format::Map { key, value } => {
            writeln!(
                w,
                "try deserializeMap(deserializer: deserializer) {{ deserializer in"
            )?;
            w.indent();
            write_format_deserialize(w, key, "key")?;
            write_format_deserialize(w, value, "value")?;
            writeln!(w, "return (key, value)")?;
            w.unindent();
            write!(w, "}}")
        }
        Format::Tuple(formats) if formats.len() == 1 => write_deserialize_expr(w, &formats[0]),
        Format::Tuple(formats) => {
            // Multi-element tuple: use Tuple{N} with temporaries
            // We write a multi-statement block, but since this is expression position,
            // the caller must handle the context (e.g. inside a closure body).
            for (i, fmt) in formats.iter().enumerate() {
                write_format_deserialize(w, fmt, &format!("field{i}"))?;
            }
            write!(w, "Tuple{}(", formats.len())?;
            for i in 0..formats.len() {
                if i > 0 {
                    write!(w, ", ")?;
                }
                write!(w, "field{i}")?;
            }
            write!(w, ")")
        }
        Format::TupleArray { content, size } => {
            writeln!(
                w,
                "try deserializeTupleArray(deserializer: deserializer, size: {size}) {{ deserializer in"
            )?;
            w.indent();
            write_deserialize_expr(w, content)?;
            writeln!(w)?;
            w.unindent();
            write!(w, "}}")
        }
        primitive => {
            let t = format!("{primitive:?}").to_lowercase();
            write!(w, "try deserializer.deserialize_{t}()")
        }
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
