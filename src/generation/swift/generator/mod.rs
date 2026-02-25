use std::{
    collections::BTreeMap,
    io::{Result, Write},
};

use crate::{
    Registry,
    generation::{
        CodeGen, CodeGeneratorConfig, Container, Emitter, common,
        indent::{IndentConfig, IndentWrite, IndentedWriter},
        module::Module,
        swift::emitter::Swift,
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for code-generation in Swift.
pub struct CodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
}

impl<'a> CodeGen<'a> for CodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        CodeGenerator::new(config)
    }

    fn write_output<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        registry: &Registry,
    ) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> CodeGenerator<'a> {
    /// Create a Swift code generator for the given config.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self { config }
    }

    /// Output class definitions for `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, IndentConfig::Space(4));

        let mut config = self.config.clone();
        config.update_from(registry);

        let lang = Swift::new(config.encoding);

        Module::new(&config).write(w, lang)?;

        let updated_registry = Self::update_qualified_names(&config, registry);
        for (i, container) in updated_registry.iter().map(Container::from).enumerate() {
            if i > 0 {
                writeln!(w)?;
            }
            container.write(w, lang)?;
        }

        if config.has_encoding() {
            writeln!(w)?;
            Self::output_trait_helpers(w, registry, lang)?;
        }

        Ok(())
    }

    /// Updates `QualifiedTypeName` instances so external types include their namespace prefix.
    /// For example, a type `Tree` in namespace `foo` becomes `Foo.Tree` in the output.
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            let _ = container_format.visit_mut(&mut |format| {
                if let Format::TypeName(qualified_name) = format {
                    if let Namespace::Named(namespace) = &qualified_name.namespace {
                        if config.external_definitions.contains_key(namespace) {
                            *qualified_name = QualifiedTypeName::namespaced(
                                namespace.clone(),
                                qualified_name.name.clone(),
                            );
                        }
                    }
                }
                Ok(())
            });
        }

        updated_registry
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

    fn output_trait_helpers<W: IndentWrite>(
        w: &mut W,
        registry: &Registry,
        lang: Swift,
    ) -> Result<()> {
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
            Self::output_serialization_helper(w, mangled_name, subtype, lang)?;
            Self::output_deserialization_helper(w, mangled_name, subtype, lang)?;
        }
        Ok(())
    }

    fn quote_serialize_value(value: &str, format: &Format) -> String {
        match format {
            Format::TypeName(_) => {
                format!("try {value}.serialize(serializer: serializer)")
            }
            Format::Option(_)
            | Format::Seq(_)
            | Format::Set(_)
            | Format::Map { .. }
            | Format::Tuple(_)
            | Format::TupleArray { .. } => format!(
                "try serialize_{}(value: {value}, serializer: serializer)",
                common::mangle_type(format),
            ),
            primitive => {
                let identifier = format!("{primitive:?}").to_lowercase();
                format!("try serializer.serialize_{identifier}(value: {value})")
            }
        }
    }

    fn quote_type(format: &Format, lang: Swift) -> String {
        let mut buf = Vec::new();
        Emitter::<Swift>::write(
            format,
            &mut IndentedWriter::new(&mut buf, IndentConfig::Space(4)),
            lang,
        )
        .expect("write to Vec should not fail");
        String::from_utf8(buf).expect("Swift type names should be UTF-8")
    }

    fn output_serialization_helper<W: IndentWrite>(
        w: &mut W,
        name: &str,
        format0: &Format,
        lang: Swift,
    ) -> Result<()> {
        let type_str = Self::quote_type(format0, lang);
        write!(
            w,
            "func serialize_{name}<S: Serializer>(value: {type_str}, serializer: S) throws {{",
        )?;
        w.indent();
        match format0 {
            Format::Option(format) => {
                write!(
                    w,
                    r"
if let value = value {{
    try serializer.serialize_option_tag(value: true)
    {}
}} else {{
    try serializer.serialize_option_tag(value: false)
}}
",
                    Self::quote_serialize_value("value", format)
                )?;
            }

            Format::Seq(format) | Format::Set(format) => {
                write!(
                    w,
                    r"
try serializer.serialize_len(value: value.count)
for item in value {{
    {}
}}
",
                    Self::quote_serialize_value("item", format)
                )?;
            }

            Format::Map { key, value } => {
                write!(
                    w,
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
                    Self::quote_serialize_value("key", key),
                    Self::quote_serialize_value("value", value)
                )?;
            }

            Format::Tuple(format_list) => {
                writeln!(w)?;
                for (index, format) in format_list.iter().enumerate() {
                    let expr = format!("value.field{index}");
                    writeln!(w, "{}", Self::quote_serialize_value(&expr, format))?;
                }
            }

            Format::TupleArray { content, size: _ } => {
                write!(
                    w,
                    r"
for item in value {{
    {}
}}
",
                    Self::quote_serialize_value("item", content),
                )?;
            }

            _ => panic!("unexpected case"),
        }
        w.unindent();
        writeln!(w, "}}\n")
    }

    fn output_deserialization_helper<W: IndentWrite>(
        w: &mut W,
        name: &str,
        format0: &Format,
        lang: Swift,
    ) -> Result<()> {
        let type_str = Self::quote_type(format0, lang);
        write!(
            w,
            "func deserialize_{name}<D: Deserializer>(deserializer: D) throws -> {type_str} {{",
        )?;
        w.indent();
        match format0 {
            Format::Option(format) => {
                write!(
                    w,
                    r"
let tag = try deserializer.deserialize_option_tag()
if tag {{
    return {}
}} else {{
    return nil
}}
",
                    Self::quote_deserialize(format),
                )?;
            }

            Format::Seq(format) | Format::Set(format) => {
                let inner_type = Self::quote_type(format, lang);
                write!(
                    w,
                    r"
let length = try deserializer.deserialize_len()
var obj : [{inner_type}] = []
for _ in 0..<length {{
    obj.append({deser})
}}
return obj
",
                    deser = Self::quote_deserialize(format)
                )?;
            }

            Format::Map { key, value } => {
                let key_type = Self::quote_type(key, lang);
                let value_type = Self::quote_type(value, lang);
                write!(
                    w,
                    r"
let length = try deserializer.deserialize_len()
var obj : [{key_type}: {value_type}] = [:]
var previous_slice = Slice(start: 0, end: 0)
for i in 0..<length {{
    var slice = Slice(start: 0, end: 0)
    slice.start = deserializer.get_buffer_offset()
    let key = {key_deser}
    slice.end = deserializer.get_buffer_offset()
    if i > 0 {{
        try deserializer.check_that_key_slices_are_increasing(key1: previous_slice, key2: slice)
    }}
    previous_slice = slice
    obj[key] = {val_deser}
}}
return obj
",
                    key_deser = Self::quote_deserialize(key),
                    val_deser = Self::quote_deserialize(value),
                )?;
            }

            Format::Tuple(format_list) => {
                write!(
                    w,
                    r"
return Tuple{}.init({})
",
                    format_list.len(),
                    format_list
                        .iter()
                        .map(Self::quote_deserialize)
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }

            Format::TupleArray { content, size } => {
                let inner_type = Self::quote_type(content, lang);
                write!(
                    w,
                    r"
var obj : [{inner_type}] = []
for _ in 0..<{size} {{
    obj.append({deser})
}}
return obj
",
                    deser = Self::quote_deserialize(content)
                )?;
            }

            _ => panic!("unexpected case"),
        }
        w.unindent();
        writeln!(w, "}}\n")
    }

    fn quote_deserialize(format: &Format) -> String {
        match format {
            Format::TypeName(name) => {
                format!("try {}.deserialize(deserializer: deserializer)", &name.name)
            }
            Format::Option(_)
            | Format::Seq(_)
            | Format::Set(_)
            | Format::Map { .. }
            | Format::Tuple(_)
            | Format::TupleArray { .. } => format!(
                "try deserialize_{}(deserializer: deserializer)",
                common::mangle_type(format)
            ),
            primitive => {
                let identifier = format!("{primitive:?}").to_lowercase();
                format!("try deserializer.deserialize_{identifier}()")
            }
        }
    }
}

#[cfg(test)]
mod tests;
