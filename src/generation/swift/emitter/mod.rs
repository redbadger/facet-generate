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

        let mut imports = vec!["Serde".to_string()];
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
                    write_feature_option(w)?;
                }
                Feature::ListOfT => {
                    writeln!(w)?;
                    write_feature_array(w)?;
                }
                Feature::SetOfT => {
                    writeln!(w)?;
                    write_feature_set(w)?;
                }
                Feature::MapOfT => {
                    writeln!(w)?;
                    write_feature_map(w)?;
                }
                Feature::BuildList => {
                    writeln!(w)?;
                    write_feature_tuple_array(w)?;
                }
                Feature::BigInt | Feature::Bytes => {}
            }
        }

        Ok(())
    }
}

fn write_feature_option<W: Write>(w: &mut W) -> Result<()> {
    writedoc!(
        w,
        r"
        func serializeOption<T, S: Serializer>(
            value: T?,
            serializer: S,
            serializeElement: (T, S) throws -> Void
        ) throws {{
            if let value = value {{
                try serializer.serialize_option_tag(value: true)
                try serializeElement(value, serializer)
            }} else {{
                try serializer.serialize_option_tag(value: false)
            }}
        }}

        func deserializeOption<T, D: Deserializer>(
            deserializer: D,
            deserializeElement: (D) throws -> T
        ) throws -> T? {{
            let tag = try deserializer.deserialize_option_tag()
            if tag {{
                return try deserializeElement(deserializer)
            }} else {{
                return nil
            }}
        }}
        "
    )
}

fn write_feature_array<W: Write>(w: &mut W) -> Result<()> {
    writedoc!(
        w,
        r"
        func serializeArray<T, S: Serializer>(
            value: [T],
            serializer: S,
            serializeElement: (T, S) throws -> Void
        ) throws {{
            try serializer.serialize_len(value: value.count)
            for item in value {{
                try serializeElement(item, serializer)
            }}
        }}

        func deserializeArray<T, D: Deserializer>(
            deserializer: D,
            deserializeElement: (D) throws -> T
        ) throws -> [T] {{
            let length = try deserializer.deserialize_len()
            var obj: [T] = []
            for _ in 0..<length {{
                obj.append(try deserializeElement(deserializer))
            }}
            return obj
        }}
        "
    )
}

fn write_feature_set<W: Write>(w: &mut W) -> Result<()> {
    writedoc!(
        w,
        r"
        func serializeSet<T: Hashable, S: Serializer>(
            value: Set<T>,
            serializer: S,
            serializeElement: (T, S) throws -> Void
        ) throws {{
            try serializer.serialize_len(value: value.count)
            for item in value {{
                try serializeElement(item, serializer)
            }}
        }}

        func deserializeSet<T: Hashable, D: Deserializer>(
            deserializer: D,
            deserializeElement: (D) throws -> T
        ) throws -> Set<T> {{
            let length = try deserializer.deserialize_len()
            var obj: Set<T> = []
            for _ in 0..<length {{
                obj.insert(try deserializeElement(deserializer))
            }}
            return obj
        }}
        "
    )
}

fn write_feature_map<W: Write>(w: &mut W) -> Result<()> {
    writedoc!(
        w,
        r"
        func serializeMap<K, V, S: Serializer>(
            value: [K: V],
            serializer: S,
            serializeEntry: (K, V, S) throws -> Void
        ) throws {{
            try serializer.serialize_len(value: value.count)
            for (key, value) in value {{
                try serializeEntry(key, value, serializer)
            }}
        }}

        func deserializeMap<K: Hashable, V, D: Deserializer>(
            deserializer: D,
            deserializeEntry: (D) throws -> (K, V)
        ) throws -> [K: V] {{
            let length = try deserializer.deserialize_len()
            var obj: [K: V] = [:]
            for _ in 0..<length {{
                let (key, value) = try deserializeEntry(deserializer)
                obj[key] = value
            }}
            return obj
        }}
        "
    )
}

fn write_feature_tuple_array<W: Write>(w: &mut W) -> Result<()> {
    writedoc!(
        w,
        r"
        func serializeTupleArray<T, S: Serializer>(
            value: [T],
            serializer: S,
            serializeElement: (T, S) throws -> Void
        ) throws {{
            for item in value {{
                try serializeElement(item, serializer)
            }}
        }}

        func deserializeTupleArray<T, D: Deserializer>(
            deserializer: D,
            size: Int,
            deserializeElement: (D) throws -> T
        ) throws -> [T] {{
            var obj: [T] = []
            for _ in 0..<size {{
                obj.append(try deserializeElement(deserializer))
            }}
            return obj
        }}
        "
    )
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
            writeln!(
                w,
                "try serializeOption(value: {value_expr}, serializer: serializer) {{ value, serializer in"
            )?;
            w.indent();
            write_format_serialize(w, inner, "value")?;
            w.unindent();
            writeln!(w, "}}")
        }
        Format::Seq(inner) => {
            writeln!(
                w,
                "try serializeArray(value: {value_expr}, serializer: serializer) {{ item, serializer in"
            )?;
            w.indent();
            write_format_serialize(w, inner, "item")?;
            w.unindent();
            writeln!(w, "}}")
        }
        Format::Set(inner) => {
            writeln!(
                w,
                "try serializeSet(value: {value_expr}, serializer: serializer) {{ item, serializer in"
            )?;
            w.indent();
            write_format_serialize(w, inner, "item")?;
            w.unindent();
            writeln!(w, "}}")
        }
        Format::Map { key, value } => {
            writeln!(
                w,
                "try serializeMap(value: {value_expr}, serializer: serializer) {{ key, value, serializer in"
            )?;
            w.indent();
            write_format_serialize(w, key, "key")?;
            write_format_serialize(w, value, "value")?;
            w.unindent();
            writeln!(w, "}}")
        }
        Format::Tuple(formats) => {
            for (i, fmt) in formats.iter().enumerate() {
                write_format_serialize(w, fmt, &format!("{value_expr}.field{i}"))?;
            }
            Ok(())
        }
        Format::TupleArray { content, .. } => {
            writeln!(
                w,
                "try serializeTupleArray(value: {value_expr}, serializer: serializer) {{ item, serializer in"
            )?;
            w.indent();
            write_format_serialize(w, content, "item")?;
            w.unindent();
            writeln!(w, "}}")
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
            write!(w, "let {var} = Tuple{}.init(", formats.len())?;
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
            let type_name = &qtn.name;
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
            // Multi-element tuple: use Tuple{N}.init with temporaries
            // We write a multi-statement block, but since this is expression position,
            // the caller must handle the context (e.g. inside a closure body).
            for (i, fmt) in formats.iter().enumerate() {
                write_format_deserialize(w, fmt, &format!("field{i}"))?;
            }
            write!(w, "Tuple{}.init(", formats.len())?;
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
