//! `EmitterPlugin<Swift>` implementation for the [`BincodePlugin`].
//!
//! Provides bincode-specific imports, feature helper snippets, and full
//! type-body generation (serialize / deserialize methods + wrappers) for
//! Swift code generation.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `import Serde` |
//! | `module_helpers` | Feature helper snippets (`ListOfT`, `SetOfT`, …) |
//! | `has_type_body` | Always `true` |
//! | `type_body` | `serialize` / `deserialize` methods + `bincodeSerialize` / `bincodeDeserialize` wrappers |
//!
//! # Bincode struct-field tuple serialisation
//!
//! For a struct field `foo: (A, B)`, Bincode calls
//! `write_format_serialize` directly, producing:
//! ```swift
//! try serializer.serialize_a(value: self.foo.0)
//! try serializer.serialize_b(value: self.foo.1)
//! ```
//! (no extra container-depth push/pop for the tuple itself; the struct-level
//! push/pop wraps the whole body).

use std::collections::BTreeMap;
use std::io;

use heck::ToLowerCamelCase as _;
use indoc::writedoc;

use crate::generation::{
    CodeGeneratorConfig, Feature,
    indent::{IndentWrite, Newlines, with_block},
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
    swift::Swift,
};
use crate::reflection::format::{ContainerFormat, Format, Named, VariantFormat};

use super::BincodePlugin;

// ---------------------------------------------------------------------------
// Inlined feature helper snippets
// ---------------------------------------------------------------------------

const FEATURE_LIST_OF_T: &str = r"func serializeArray<T, S: Serializer>(
    value: [T],
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    try serializer.serialize_len(value: value.count)
    for item in value {
        try serializeElement(item, serializer)
    }
}

func deserializeArray<T, D: Deserializer>(
    deserializer: D,
    deserializeElement: (D) throws -> T
) throws -> [T] {
    let length = try deserializer.deserialize_len()
    var obj: [T] = []
    for _ in 0..<length {
        obj.append(try deserializeElement(deserializer))
    }
    return obj
}
";

const FEATURE_MAP_OF_T: &str = r"func serializeMap<K, V, S: Serializer>(
    value: [K: V],
    serializer: S,
    serializeEntry: (K, V, S) throws -> Void
) throws {
    try serializer.serialize_len(value: value.count)
    for (key, value) in value {
        try serializeEntry(key, value, serializer)
    }
}

func deserializeMap<K: Hashable, V, D: Deserializer>(
    deserializer: D,
    deserializeEntry: (D) throws -> (K, V)
) throws -> [K: V] {
    let length = try deserializer.deserialize_len()
    var obj: [K: V] = [:]
    for _ in 0..<length {
        let (key, value) = try deserializeEntry(deserializer)
        obj[key] = value
    }
    return obj
}
";

const FEATURE_OPTION_OF_T: &str = r"func serializeOption<T, S: Serializer>(
    value: T?,
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    if let value = value {
        try serializer.serialize_option_tag(value: true)
        try serializeElement(value, serializer)
    } else {
        try serializer.serialize_option_tag(value: false)
    }
}

func deserializeOption<T, D: Deserializer>(
    deserializer: D,
    deserializeElement: (D) throws -> T
) throws -> T? {
    let tag = try deserializer.deserialize_option_tag()
    if tag {
        return try deserializeElement(deserializer)
    } else {
        return nil
    }
}
";

const FEATURE_SET_OF_T: &str = r"func serializeSet<T: Hashable, S: Serializer>(
    value: Set<T>,
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    try serializer.serialize_len(value: value.count)
    for item in value {
        try serializeElement(item, serializer)
    }
}

func deserializeSet<T: Hashable, D: Deserializer>(
    deserializer: D,
    deserializeElement: (D) throws -> T
) throws -> Set<T> {
    let length = try deserializer.deserialize_len()
    var obj: Set<T> = []
    for _ in 0..<length {
        obj.insert(try deserializeElement(deserializer))
    }
    return obj
}
";

const FEATURE_TUPLE_ARRAY: &str = r"func serializeTupleArray<T, S: Serializer>(
    value: [T],
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    for item in value {
        try serializeElement(item, serializer)
    }
}

func deserializeTupleArray<T, D: Deserializer>(
    deserializer: D,
    size: Int,
    deserializeElement: (D) throws -> T
) throws -> [T] {
    var obj: [T] = []
    for _ in 0..<size {
        obj.append(try deserializeElement(deserializer))
    }
    return obj
}
";

// ---------------------------------------------------------------------------
// EmitterPlugin implementation
// ---------------------------------------------------------------------------

impl EmitterPlugin<Swift> for BincodePlugin {
    /// Returns the Serde Swift runtime sources to be written into the output
    /// directory. Swift's bincode runtime is bundled inside the Serde target
    /// (see `runtime/swift/Sources/Serde/`), so there are no separate bincode
    /// files to install.
    fn runtime_files(&self) -> Vec<RuntimeFile> {
        static SERDE: include_dir::Dir<'static> =
            include_dir::include_dir!("$CARGO_MANIFEST_DIR/runtime/swift/Sources/Serde");
        SERDE
            .files()
            .map(|f| RuntimeFile {
                relative_path: format!("Sources/Serde/{}", f.path().display()),
                contents: f.contents().to_vec(),
            })
            .collect()
    }

    fn imports(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec!["Serde".to_string()]
    }

    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        for feature in &config.features {
            match feature {
                Feature::OptionOfT => {
                    writeln!(w)?;
                    write!(w, "{FEATURE_OPTION_OF_T}")?;
                }
                Feature::ListOfT => {
                    writeln!(w)?;
                    write!(w, "{FEATURE_LIST_OF_T}")?;
                }
                Feature::SetOfT => {
                    writeln!(w)?;
                    write!(w, "{FEATURE_SET_OF_T}")?;
                }
                Feature::MapOfT => {
                    writeln!(w)?;
                    write!(w, "{FEATURE_MAP_OF_T}")?;
                }
                Feature::TupleArray => {
                    writeln!(w)?;
                    write!(w, "{FEATURE_TUPLE_ARRAY}")?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn has_type_body(&self, _ctx: &EmitContext) -> bool {
        true
    }

    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        let name = ctx.name();
        if let ContainerFormat::Enum(variants, _) = ctx.container.format {
            write_enum_type_body(w, name, variants)
        } else {
            write_struct_type_body(w, name, &ctx.fields())
        }
    }
}

// ---------------------------------------------------------------------------
// Struct type body
// ---------------------------------------------------------------------------

fn write_struct_type_body(
    w: &mut dyn IndentWrite,
    name: &str,
    fields: &[Named<Format>],
) -> io::Result<()> {
    writeln!(w)?;
    write!(
        w,
        "public func serialize<S: Serializer>(serializer: S) throws "
    )?;
    with_block(w, Newlines::BOTH, |w| {
        push_serializer(w)?;
        for field in fields {
            let fname = field.name.to_lower_camel_case();
            write_format_serialize(w, &field.value, &format!("self.{fname}"))?;
        }
        pop_serializer(w)
    })?;
    write_bincode_serialize(w)?;

    writeln!(w)?;
    write!(
        w,
        "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
    )?;
    with_block(w, Newlines::BOTH, |w| {
        push_deserializer(w)?;
        for field in fields {
            let fname = field.name.to_lower_camel_case();
            write_format_deserialize(w, &field.value, &fname)?;
        }
        pop_deserializer(w)?;
        write!(w, "return {name}(")?;
        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                write!(w, ", ")?;
            }
            let fname = field.name.to_lower_camel_case();
            write!(w, "{fname}: {fname}")?;
        }
        writeln!(w, ")")
    })?;
    write_bincode_deserialize(w, name)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Enum type body
// ---------------------------------------------------------------------------

fn write_enum_type_body(
    w: &mut dyn IndentWrite,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> io::Result<()> {
    writeln!(w)?;
    write!(
        w,
        "public func serialize<S: Serializer>(serializer: S) throws "
    )?;
    with_block(w, Newlines::BOTH, |w| {
        push_serializer(w)?;
        write!(w, "switch self ")?;
        with_block(w, Newlines::BOTH, |w| {
            w.unindent();
            for (i, variant) in variants.values().enumerate() {
                write_variant_serialize_case(w, variant, i)?;
            }
            w.indent();
            Ok(())
        })?;
        pop_serializer(w)
    })?;
    write_bincode_serialize(w)?;

    writeln!(w)?;
    write!(
        w,
        "public static func deserialize<D: Deserializer>(deserializer: D) throws -> {name} "
    )?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(
            w,
            "let index = try deserializer.deserialize_variant_index()"
        )?;
        push_deserializer(w)?;
        write!(w, "switch index ")?;
        with_block(w, Newlines::BOTH, |w| {
            w.unindent();
            for (i, variant) in variants.values().enumerate() {
                write_variant_deserialize_case(w, variant, i)?;
            }
            writeln!(
                w,
                r#"default: throw DeserializationError.invalidInput(issue: "Unknown variant index for {name}: \(index)")"#
            )?;
            w.indent();
            Ok(())
        })
    })?;
    write_bincode_deserialize(w, name)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Variant helpers
// ---------------------------------------------------------------------------

fn write_variant_serialize_case(
    w: &mut dyn IndentWrite,
    variant: &Named<VariantFormat>,
    index: usize,
) -> io::Result<()> {
    let name = variant.name.to_lower_camel_case();
    match &variant.value {
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
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

fn write_variant_deserialize_case(
    w: &mut dyn IndentWrite,
    variant: &Named<VariantFormat>,
    index: usize,
) -> io::Result<()> {
    let name = variant.name.to_lower_camel_case();
    writeln!(w, "case {index}:")?;
    w.indent();
    match &variant.value {
        VariantFormat::Variable(_) => unreachable!("placeholders should not get this far"),
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

// ---------------------------------------------------------------------------
// Encoding wrappers
// ---------------------------------------------------------------------------

fn write_bincode_serialize(w: &mut dyn IndentWrite) -> io::Result<()> {
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

fn write_bincode_deserialize(w: &mut dyn IndentWrite, name: &str) -> io::Result<()> {
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

// ---------------------------------------------------------------------------
// Format serialisation helpers
// ---------------------------------------------------------------------------

fn write_format_serialize(
    w: &mut dyn IndentWrite,
    format: &Format,
    value_expr: &str,
) -> io::Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(_) => writeln!(w, "try {value_expr}.serialize(serializer: serializer)"),
        Format::Option(inner) => {
            write!(
                w,
                "try serializeOption(value: {value_expr}, serializer: serializer) "
            )?;
            with_block(w, Newlines::CLOSE, |w| {
                writeln!(w, " value, serializer in")?;
                write_format_serialize(w, inner, "value")
            })
        }
        Format::Seq(inner) => {
            write!(
                w,
                "try serializeArray(value: {value_expr}, serializer: serializer) "
            )?;
            with_block(w, Newlines::CLOSE, |w| {
                writeln!(w, " item, serializer in")?;
                write_format_serialize(w, inner, "item")
            })
        }
        Format::Set(inner) => {
            write!(
                w,
                "try serializeSet(value: {value_expr}, serializer: serializer) "
            )?;
            with_block(w, Newlines::CLOSE, |w| {
                writeln!(w, " item, serializer in")?;
                write_format_serialize(w, inner, "item")
            })
        }
        Format::Map { key, value } => {
            write!(
                w,
                "try serializeMap(value: {value_expr}, serializer: serializer) "
            )?;
            with_block(w, Newlines::CLOSE, |w| {
                writeln!(w, " key, value, serializer in")?;
                write_format_serialize(w, key, "key")?;
                write_format_serialize(w, value, "value")
            })
        }
        Format::Tuple(formats) => {
            for (i, fmt) in formats.iter().enumerate() {
                write_format_serialize(w, fmt, &format!("{value_expr}.{i}"))?;
            }
            Ok(())
        }
        Format::TupleArray { content, .. } => {
            write!(
                w,
                "try serializeTupleArray(value: {value_expr}, serializer: serializer) "
            )?;
            with_block(w, Newlines::CLOSE, |w| {
                writeln!(w, " item, serializer in")?;
                write_format_serialize(w, content, "item")
            })
        }
        primitive => {
            let t = format!("{primitive:?}").to_lowercase();
            writeln!(w, "try serializer.serialize_{t}(value: {value_expr})")
        }
    }
}

fn write_format_deserialize(w: &mut dyn IndentWrite, format: &Format, var: &str) -> io::Result<()> {
    match format {
        Format::Tuple(formats) if formats.len() > 1 => {
            for (i, fmt) in formats.iter().enumerate() {
                write_format_deserialize(w, fmt, &format!("{var}Field{i}"))?;
            }
            write!(w, "let {var} = (")?;
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

fn write_deserialize_expr(w: &mut dyn IndentWrite, format: &Format) -> io::Result<()> {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::TypeName(qtn) => {
            let type_name = qtn.format(|ns| heck::AsUpperCamelCase(ns).to_string(), ".");
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
            for (i, fmt) in formats.iter().enumerate() {
                write_format_deserialize(w, fmt, &format!("field{i}"))?;
            }
            write!(w, "return (")?;
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

// ---------------------------------------------------------------------------
// Depth tracking
// ---------------------------------------------------------------------------

fn push_serializer(w: &mut dyn IndentWrite) -> io::Result<()> {
    writeln!(w, "try serializer.increase_container_depth()")
}

fn pop_serializer(w: &mut dyn IndentWrite) -> io::Result<()> {
    writeln!(w, "try serializer.decrease_container_depth()")
}

fn push_deserializer(w: &mut dyn IndentWrite) -> io::Result<()> {
    writeln!(w, "try deserializer.increase_container_depth()")
}

fn pop_deserializer(w: &mut dyn IndentWrite) -> io::Result<()> {
    writeln!(w, "try deserializer.decrease_container_depth()")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::CodeGeneratorConfig;
    use crate::generation::indent::IndentedWriter;
    use std::collections::BTreeSet;

    fn make_config(features: &[Feature]) -> CodeGeneratorConfig {
        let mut cfg = CodeGeneratorConfig::new("com.example".to_string());
        cfg.features = features.iter().copied().collect::<BTreeSet<_>>();
        cfg
    }

    #[test]
    fn imports_returns_serde() {
        let cfg = make_config(&[]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<Swift>;
        let imports = plugin.imports(&cfg);

        assert_eq!(imports, vec!["Serde"]);
    }

    #[test]
    fn module_helpers_emit_list_of_t() {
        let cfg = make_config(&[Feature::ListOfT]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<Swift>;

        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin.module_helpers(&mut w, &cfg).unwrap();
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("serializeArray"));
        assert!(output.contains("deserializeArray"));
    }

    #[test]
    fn module_helpers_emit_only_requested_features() {
        let cfg = make_config(&[Feature::SetOfT]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<Swift>;

        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin.module_helpers(&mut w, &cfg).unwrap();
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("serializeSet"));
        assert!(!output.contains("serializeArray"));
        assert!(!output.contains("serializeMap"));
    }

    #[test]
    fn has_type_body_always_true() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

        let plugin = &BincodePlugin as &dyn EmitterPlugin<Swift>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);
        assert!(plugin.has_type_body(&ctx));
    }

    #[test]
    fn type_body_unit_struct() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

        let cfg = make_config(&[]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<Swift>;

        let name = QualifiedTypeName::root("UnitStruct".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin
                .type_body(&mut w as &mut dyn IndentWrite, &ctx)
                .unwrap();
        }
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("public func serialize<S: Serializer>"));
        assert!(output.contains("increase_container_depth"));
        assert!(output.contains("decrease_container_depth"));
        assert!(output.contains("bincodeSerialize"));
        assert!(output.contains("public static func deserialize<D: Deserializer>"));
        assert!(output.contains("return UnitStruct()"));
        assert!(output.contains("bincodeDeserialize"));
    }

    #[test]
    fn type_body_struct_with_fields() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, Format, QualifiedTypeName};

        let cfg = make_config(&[]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<Swift>;

        let name = QualifiedTypeName::root("MyStruct".to_string());
        let fields = vec![
            Named::new(&Format::Str, "label".to_string()),
            Named::new(&Format::I32, "count".to_string()),
        ];
        let format = ContainerFormat::Struct(fields, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin
                .type_body(&mut w as &mut dyn IndentWrite, &ctx)
                .unwrap();
        }
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("serialize_str(value: self.label)"));
        assert!(output.contains("serialize_i32(value: self.count)"));
        assert!(output.contains("bincodeSerialize"));
        assert!(output.contains("let label = try deserializer.deserialize_str()"));
        assert!(output.contains("let count = try deserializer.deserialize_i32()"));
        assert!(output.contains("return MyStruct(label: label, count: count)"));
        assert!(output.contains("bincodeDeserialize"));
    }

    #[test]
    fn type_body_struct_tuple_field_no_extra_depth() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, Format, QualifiedTypeName};

        let cfg = make_config(&[]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<Swift>;

        let name = QualifiedTypeName::root("MyStruct".to_string());
        let fields = vec![Named::new(
            &Format::Tuple(vec![Format::Str, Format::I32]),
            "pair".to_string(),
        )];
        let format = ContainerFormat::Struct(fields, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin
                .type_body(&mut w as &mut dyn IndentWrite, &ctx)
                .unwrap();
        }
        let output = String::from_utf8(buf).unwrap();
        // Bincode: one push/pop pair per method (serialize + deserialize) = 2 total,
        // with no extra push/pop for the tuple field itself
        assert_eq!(output.matches("increase_container_depth").count(), 2);
        // Elements accessed as self.pair.0, self.pair.1
        assert!(output.contains("self.pair.0"));
        assert!(output.contains("self.pair.1"));
        // Deserialization variable names use Field infix
        assert!(output.contains("pairField0"));
        assert!(output.contains("pairField1"));
        assert!(output.contains("let pair = (pairField0, pairField1)"));
    }

    #[test]
    fn type_body_enum_with_variants() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, Format, QualifiedTypeName};

        let cfg = make_config(&[]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<Swift>;

        let mut variants = BTreeMap::new();
        variants.insert(
            0,
            Named {
                name: "unit".to_string(),
                doc: Doc::default(),
                value: VariantFormat::Unit,
            },
        );
        variants.insert(
            1,
            Named {
                name: "withValue".to_string(),
                doc: Doc::default(),
                value: VariantFormat::NewType(Box::new(Format::Str)),
            },
        );
        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin
                .type_body(&mut w as &mut dyn IndentWrite, &ctx)
                .unwrap();
        }
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("serialize_variant_index(value: 0)"));
        assert!(output.contains("serialize_variant_index(value: 1)"));
        assert!(output.contains("deserialize_variant_index()"));
        assert!(output.contains("return .unit"));
        assert!(output.contains("serialize_str(value: x)"));
        assert!(output.contains("return .withValue(x)"));
        assert!(output.contains("bincodeSerialize"));
        assert!(output.contains("bincodeDeserialize"));
    }
}
