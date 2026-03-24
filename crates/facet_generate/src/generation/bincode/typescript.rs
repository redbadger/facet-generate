//! `EmitterPlugin<TypeScript>` implementation for the [`BincodePlugin`].
//!
//! Provides feature helper snippets and full type-body generation
//! (serialize / deserialize methods) for TypeScript Bincode code generation.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `module_helpers` | Feature helper snippets (`ArrayOfT`, `SetOfT`, …) |
//! | `has_type_body` | Always `true` |
//! | `type_body` | `serialize` / `deserialize` methods for structs and enums |
//!
//! # TypeScript serialize/deserialize pattern
//!
//! Both Bincode and JSON encodings use the same `Serializer`/`Deserializer`
//! interface pattern in TypeScript. The difference is purely which runtime
//! library is installed. As a result, this plugin generates identical code to
//! the JSON TypeScript plugin.

use std::collections::BTreeMap;
use std::io;

use heck::ToUpperCamelCase;

use crate::generation::{
    CodeGeneratorConfig, Feature, PackageLocation, SERDE_NAMESPACE,
    indent::{IndentWrite, Newlines, with_block},
    plugin::{EmitContext, EmitterPlugin, RuntimeFile, VariantInfo},
    typescript::TypeScript,
};
use crate::reflection::format::{ContainerFormat, Format, Named, VariantFormat};

use super::BincodePlugin;

// ---------------------------------------------------------------------------
// Inlined feature helper snippets
// ---------------------------------------------------------------------------

const FEATURE_LIST_OF_T: &str = r"function serializeArray<T>(
    value: T[],
    serializer: Serializer,
    serializeElement: (item: T, serializer: Serializer) => void,
): void {
    serializer.serializeLen(value.length);
    value.forEach((item) => {
        serializeElement(item, serializer);
    });
}

function deserializeArray<T>(
    deserializer: Deserializer,
    deserializeElement: (deserializer: Deserializer) => T,
): T[] {
    const length = deserializer.deserializeLen();
    const list: T[] = [];
    for (let i = 0; i < length; i++) {
        list.push(deserializeElement(deserializer));
    }
    return list;
}
";

const FEATURE_SET_OF_T: &str = r"function serializeSet<T>(
    value: T[],
    serializer: Serializer,
    serializeElement: (item: T, serializer: Serializer) => void,
): void {
    serializer.serializeLen(value.length);
    value.forEach((item) => {
        serializeElement(item, serializer);
    });
}

function deserializeSet<T>(
    deserializer: Deserializer,
    deserializeElement: (deserializer: Deserializer) => T,
): T[] {
    const length = deserializer.deserializeLen();
    const list: T[] = [];
    for (let i = 0; i < length; i++) {
        list.push(deserializeElement(deserializer));
    }
    return list;
}
";

const FEATURE_MAP_OF_T: &str = r"function serializeMap<K, V>(
    value: Map<K, V>,
    serializer: Serializer,
    serializeEntry: (key: K, value: V, serializer: Serializer) => void,
): void {
    serializer.serializeLen(value.size);
    const offsets: number[] = [];
    for (const [k, v] of value.entries()) {
        offsets.push(serializer.getBufferOffset());
        serializeEntry(k, v, serializer);
    }
    serializer.sortMapEntries(offsets);
}

function deserializeMap<K, V>(
    deserializer: Deserializer,
    deserializeEntry: (deserializer: Deserializer) => [K, V],
): Map<K, V> {
    const length = deserializer.deserializeLen();
    const obj = new Map<K, V>();
    for (let i = 0; i < length; i++) {
        const [key, value] = deserializeEntry(deserializer);
        obj.set(key, value);
    }
    return obj;
}
";

const FEATURE_OPTION_OF_T: &str = r"function serializeOption<T>(
    value: T | null,
    serializer: Serializer,
    serializeElement: (value: T, serializer: Serializer) => void,
): void {
    if (value !== null) {
        serializer.serializeOptionTag(true);
        serializeElement(value, serializer);
    } else {
        serializer.serializeOptionTag(false);
    }
}

function deserializeOption<T>(
    deserializer: Deserializer,
    deserializeElement: (deserializer: Deserializer) => T,
): T | null {
    const tag = deserializer.deserializeOptionTag();
    if (!tag) {
        return null;
    } else {
        return deserializeElement(deserializer);
    }
}
";

const FEATURE_TUPLE_ARRAY: &str = r"function serializeTupleArray<T>(
    value: T[],
    serializer: Serializer,
    serializeElement: (item: T, serializer: Serializer) => void,
): void {
    value.forEach((item) => {
        serializeElement(item, serializer);
    });
}

function deserializeTupleArray<T>(
    deserializer: Deserializer,
    size: number,
    deserializeElement: (deserializer: Deserializer) => T,
): T[] {
    const list: T[] = [];
    for (let i = 0; i < size; i++) {
        list.push(deserializeElement(deserializer));
    }
    return list;
}
";

// ---------------------------------------------------------------------------
// EmitterPlugin implementation
// ---------------------------------------------------------------------------

impl EmitterPlugin<TypeScript> for BincodePlugin {
    /// Returns the `import { Serializer, Deserializer }` statement needed by
    /// the generated serialize/deserialize methods. The import path is resolved
    /// from `config.external_packages` the same way the module emitter used to
    /// resolve it via `has_encoding()`.
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let import_path = config.external_packages.get(SERDE_NAMESPACE).map_or_else(
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
        vec![format!(
            r#"import {{ Serializer, Deserializer }} from "{import_path}";"#
        )]
    }

    /// Returns the serde and bincode TypeScript runtime sources to be written
    /// into the output directory alongside the generated code.
    fn runtime_files(&self) -> Vec<RuntimeFile> {
        static SERDE: include_dir::Dir<'static> =
            include_dir::include_dir!("$CARGO_MANIFEST_DIR/runtime/typescript-node/serde");
        static BINCODE: include_dir::Dir<'static> =
            include_dir::include_dir!("$CARGO_MANIFEST_DIR/runtime/typescript-node/bincode");

        let mut files: Vec<RuntimeFile> = SERDE
            .files()
            .map(|f| RuntimeFile {
                relative_path: format!("serde/{}", f.path().display()),
                contents: f.contents().to_vec(),
            })
            .collect();
        files.extend(BINCODE.files().map(|f| RuntimeFile {
            relative_path: format!("bincode/{}", f.path().display()),
            contents: f.contents().to_vec(),
        }));
        files
    }

    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        for feature in &config.features {
            match feature {
                Feature::ListOfT => {
                    writeln!(w)?;
                    write!(w, "{FEATURE_LIST_OF_T}")?;
                }
                Feature::OptionOfT => {
                    writeln!(w)?;
                    write!(w, "{FEATURE_OPTION_OF_T}")?;
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
                Feature::BigInt | Feature::Bytes => {}
            }
        }
        Ok(())
    }

    fn has_type_body(&self, _ctx: &EmitContext) -> bool {
        true
    }

    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        if let Some(variant) = &ctx.variant {
            write_variant_type_body(w, variant)
        } else {
            match ctx.container.format {
                ContainerFormat::Enum(variants, _) => write_enum_type_body(w, ctx.name(), variants),
                _ => write_struct_type_body(w, ctx.name(), &ctx.fields()),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Type body writers
// ---------------------------------------------------------------------------

/// Emit `serialize` / `deserialize` for a plain struct or newtype.
fn write_struct_type_body(
    w: &mut dyn IndentWrite,
    name: &str,
    fields: &[Named<Format>],
) -> io::Result<()> {
    writeln!(w)?;
    write!(w, "public serialize(serializer: Serializer): void ")?;
    with_block(w, Newlines::BOTH, |w| {
        for field in fields {
            write_serialize(w, &format!("this.{}", field.name), &field.value)?;
        }
        Ok(())
    })?;
    writeln!(w)?;
    write!(w, "static deserialize(deserializer: Deserializer): {name} ")?;
    with_block(w, Newlines::BOTH, |w| {
        for field in fields {
            write_deserialize(w, Some(&field.name), &field.value)?;
        }
        writeln!(
            w,
            "return new {name}({args});",
            args = fields
                .iter()
                .map(|f| f.name.clone())
                .collect::<Vec<_>>()
                .join(",")
        )
    })?;
    Ok(())
}

/// Emit `serialize` (with variant index) / `load` for an enum variant subclass.
fn write_variant_type_body(w: &mut dyn IndentWrite, variant: &VariantInfo<'_>) -> io::Result<()> {
    let name = variant.name;
    let index = variant.index;
    let parent_name = variant.parent_name;
    let base_variant_name = format!("{parent_name}Variant");
    let fields = variant.fields;

    writeln!(w)?;
    write!(w, "public serialize(serializer: Serializer): void ")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "serializer.serializeVariantIndex({index});")?;
        for field in fields {
            write_serialize(w, &format!("this.{}", field.name), &field.value)?;
        }
        Ok(())
    })?;
    writeln!(w)?;
    write!(
        w,
        "static load(deserializer: Deserializer): {base_variant_name}{name} "
    )?;
    with_block(w, Newlines::BOTH, |w| {
        for field in fields {
            write_deserialize(w, Some(&field.name), &field.value)?;
        }
        writeln!(
            w,
            "return new {base_variant_name}{name}({args});",
            args = fields
                .iter()
                .map(|f| f.name.clone())
                .collect::<Vec<_>>()
                .join(",")
        )
    })?;
    Ok(())
}

/// Emit the abstract class body for an enum: `abstract serialize` declaration
/// plus a `static deserialize` switch dispatch.
fn write_enum_type_body(
    w: &mut dyn IndentWrite,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> io::Result<()> {
    // The trailing `\n` in the format string creates the blank line between
    // the abstract declaration and the static deserialize method.
    writeln!(w, "abstract serialize(serializer: Serializer): void;\n")?;
    write!(w, "static deserialize(deserializer: Deserializer): {name} ")?;
    with_block(w, Newlines::BOTH, |w| {
        writeln!(w, "const index = deserializer.deserializeVariantIndex();")?;
        write!(w, "switch (index) ")?;
        with_block(w, Newlines::BOTH, |w| {
            for (index, variant) in variants {
                writeln!(
                    w,
                    "case {index}: return {name}Variant{vname}.load(deserializer);",
                    vname = variant.name
                )?;
            }
            writeln!(
                w,
                r#"default: throw new Error("Unknown variant index for {name}: " + index);"#
            )
        })
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Serialize helpers
// ---------------------------------------------------------------------------

fn write_serialize(w: &mut dyn IndentWrite, value_expr: &str, format: &Format) -> io::Result<()> {
    match format {
        Format::TypeName(_) => writeln!(w, "{value_expr}.serialize(serializer);"),
        Format::Unit => writeln!(w, "serializer.serializeUnit({value_expr});"),
        Format::Bool => writeln!(w, "serializer.serializeBool({value_expr});"),
        Format::I8 => writeln!(w, "serializer.serializeI8({value_expr});"),
        Format::I16 => writeln!(w, "serializer.serializeI16({value_expr});"),
        Format::I32 => writeln!(w, "serializer.serializeI32({value_expr});"),
        Format::I64 => writeln!(w, "serializer.serializeI64({value_expr});"),
        Format::I128 => writeln!(w, "serializer.serializeI128({value_expr});"),
        Format::U8 => writeln!(w, "serializer.serializeU8({value_expr});"),
        Format::U16 => writeln!(w, "serializer.serializeU16({value_expr});"),
        Format::U32 => writeln!(w, "serializer.serializeU32({value_expr});"),
        Format::U64 => writeln!(w, "serializer.serializeU64({value_expr});"),
        Format::U128 => writeln!(w, "serializer.serializeU128({value_expr});"),
        Format::F32 => writeln!(w, "serializer.serializeF32({value_expr});"),
        Format::F64 => writeln!(w, "serializer.serializeF64({value_expr});"),
        Format::Char => writeln!(w, "serializer.serializeChar({value_expr});"),
        Format::Str => writeln!(w, "serializer.serializeStr({value_expr});"),
        Format::Bytes => writeln!(w, "serializer.serializeBytes({value_expr});"),
        Format::Option(inner) => {
            write!(
                w,
                "serializeOption({value_expr}, serializer, (value, serializer) => "
            )?;
            with_block(w, Newlines::OPEN, |w| write_serialize(w, "value", inner))?;
            writeln!(w, ");")
        }
        Format::Seq(inner) => {
            write!(
                w,
                "serializeArray({value_expr}, serializer, (item, serializer) => "
            )?;
            with_block(w, Newlines::OPEN, |w| write_serialize(w, "item", inner))?;
            writeln!(w, ");")
        }
        Format::Set(inner) => {
            write!(
                w,
                "serializeSet({value_expr}, serializer, (item, serializer) => "
            )?;
            with_block(w, Newlines::OPEN, |w| write_serialize(w, "item", inner))?;
            writeln!(w, ");")
        }
        Format::Map { key, value } => {
            write!(
                w,
                "serializeMap({value_expr}, serializer, (key, value, serializer) => "
            )?;
            with_block(w, Newlines::OPEN, |w| {
                write_serialize(w, "key", key)?;
                write_serialize(w, "value", value)
            })?;
            writeln!(w, ");")
        }
        Format::Tuple(formats) => {
            for (i, fmt) in formats.iter().enumerate() {
                write_serialize(w, &format!("{value_expr}[{i}]"), fmt)?;
            }
            Ok(())
        }
        Format::TupleArray { content, .. } => {
            write!(
                w,
                "serializeTupleArray({value_expr}, serializer, (item, serializer) => "
            )?;
            with_block(w, Newlines::OPEN, |w| {
                write_serialize(w, "item[0]", content)
            })?;
            writeln!(w, ");")
        }
        Format::Variable(_) => panic!("unexpected variable in write_serialize"),
    }
}

// ---------------------------------------------------------------------------
// Deserialize helpers
// ---------------------------------------------------------------------------

/// Renders a TypeScript type expression for `format` without requiring a
/// language tag — the mapping is fixed for TypeScript regardless of encoding.
fn quote_type(format: &Format) -> String {
    match format {
        Format::TypeName(type_) => type_.format(ToUpperCamelCase::to_upper_camel_case, "."),
        Format::Unit => "unit".to_string(),
        Format::Bool => "bool".to_string(),
        Format::I8 => "int8".to_string(),
        Format::I16 => "int16".to_string(),
        Format::I32 => "int32".to_string(),
        Format::I64 => "int64".to_string(),
        Format::I128 => "int128".to_string(),
        Format::U8 => "uint8".to_string(),
        Format::U16 => "uint16".to_string(),
        Format::U32 => "uint32".to_string(),
        Format::U64 => "uint64".to_string(),
        Format::U128 => "uint128".to_string(),
        Format::F32 => "float32".to_string(),
        Format::F64 => "float64".to_string(),
        Format::Char => "char".to_string(),
        Format::Str => "str".to_string(),
        Format::Bytes => "bytes".to_string(),
        Format::Option(inner) => format!("Optional<{}>", quote_type(inner)),
        Format::Seq(inner) | Format::Set(inner) => format!("Seq<{}>", quote_type(inner)),
        Format::Map { key, value } => {
            format!("Map<{},{}>", quote_type(key), quote_type(value))
        }
        Format::Tuple(formats) => {
            let inner = formats
                .iter()
                .map(quote_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("Tuple<[{inner}]>")
        }
        Format::TupleArray { content, .. } => {
            format!("ListTuple<[{}]>", quote_type(content))
        }
        Format::Variable(_) => panic!("unexpected variable in quote_type"),
    }
}

/// Returns the deserialize expression for a primitive or named type.
fn deserialize_primitive_expr(format: &Format) -> String {
    match format {
        Format::TypeName(qualified_name) => format!(
            "{}.deserialize(deserializer)",
            qualified_name.format(ToUpperCamelCase::to_upper_camel_case, ".")
        ),
        Format::Unit => "deserializer.deserializeUnit()".to_string(),
        Format::Bool => "deserializer.deserializeBool()".to_string(),
        Format::I8 => "deserializer.deserializeI8()".to_string(),
        Format::I16 => "deserializer.deserializeI16()".to_string(),
        Format::I32 => "deserializer.deserializeI32()".to_string(),
        Format::I64 => "deserializer.deserializeI64()".to_string(),
        Format::I128 => "deserializer.deserializeI128()".to_string(),
        Format::U8 => "deserializer.deserializeU8()".to_string(),
        Format::U16 => "deserializer.deserializeU16()".to_string(),
        Format::U32 => "deserializer.deserializeU32()".to_string(),
        Format::U64 => "deserializer.deserializeU64()".to_string(),
        Format::U128 => "deserializer.deserializeU128()".to_string(),
        Format::F32 => "deserializer.deserializeF32()".to_string(),
        Format::F64 => "deserializer.deserializeF64()".to_string(),
        Format::Char => "deserializer.deserializeChar()".to_string(),
        Format::Str => "deserializer.deserializeStr()".to_string(),
        Format::Bytes => "deserializer.deserializeBytes()".to_string(),
        _ => panic!("deserialize_primitive_expr called with non-primitive format"),
    }
}

/// Returns `true` for primitive types and named (user-defined) type references.
const fn is_primitive_or_named(format: &Format) -> bool {
    matches!(
        format,
        Format::TypeName(_)
            | Format::Unit
            | Format::Bool
            | Format::I8
            | Format::I16
            | Format::I32
            | Format::I64
            | Format::I128
            | Format::U8
            | Format::U16
            | Format::U32
            | Format::U64
            | Format::U128
            | Format::F32
            | Format::F64
            | Format::Char
            | Format::Str
            | Format::Bytes
    )
}

/// Write a deserialize statement for `format`.
///
/// When `field_name` is `Some`, emits `const <name> = <expr>;`.
/// When `field_name` is `None`, emits `return <expr>;`.
#[allow(clippy::too_many_lines)]
fn write_deserialize(
    w: &mut dyn IndentWrite,
    field_name: Option<&str>,
    format: &Format,
) -> io::Result<()> {
    match format {
        // Primitive and named types — simple single-expression form.
        f if is_primitive_or_named(f) => {
            let expr = deserialize_primitive_expr(f);
            if let Some(name) = field_name {
                writeln!(w, "const {name} = {expr};")
            } else {
                writeln!(w, "return {expr};")
            }
        }

        Format::Option(inner) => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeOption(deserializer, (deserializer) => "
                )?;
            } else {
                write!(
                    w,
                    "return deserializeOption(deserializer, (deserializer) => "
                )?;
            }
            with_block(w, Newlines::OPEN, |w| write_deserialize(w, None, inner))?;
            writeln!(w, ");")
        }

        Format::Seq(inner) => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeArray(deserializer, (deserializer) => "
                )?;
            } else {
                write!(
                    w,
                    "return deserializeArray(deserializer, (deserializer) => "
                )?;
            }
            with_block(w, Newlines::OPEN, |w| write_deserialize(w, None, inner))?;
            writeln!(w, ");")
        }

        Format::Set(inner) => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeSet(deserializer, (deserializer) => "
                )?;
            } else {
                write!(w, "return deserializeSet(deserializer, (deserializer) => ")?;
            }
            with_block(w, Newlines::OPEN, |w| write_deserialize(w, None, inner))?;
            writeln!(w, ");")
        }

        Format::Map { key, value } => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeMap(deserializer, (deserializer) => "
                )?;
            } else {
                write!(w, "return deserializeMap(deserializer, (deserializer) => ")?;
            }
            with_block(w, Newlines::OPEN, |w| {
                if is_primitive_or_named(key) {
                    writeln!(w, "const key = {};", deserialize_primitive_expr(key))?;
                } else {
                    write_deserialize(w, Some("key"), key)?;
                }
                if is_primitive_or_named(value) {
                    writeln!(w, "const value = {};", deserialize_primitive_expr(value))?;
                } else {
                    write_deserialize(w, Some("value"), value)?;
                }
                writeln!(w, "return [key, value];")
            })?;
            writeln!(w, ");")
        }

        Format::Tuple(formats) => {
            for (i, f) in formats.iter().enumerate() {
                write_deserialize(w, Some(&format!("field{i}")), f)?;
            }
            let fields_joined = (0..formats.len())
                .map(|i| format!("field{i}"))
                .collect::<Vec<_>>()
                .join(", ");
            let type_str = formats
                .iter()
                .map(quote_type)
                .collect::<Vec<_>>()
                .join(", ");
            if let Some(name) = field_name {
                writeln!(w, "const {name} = [{fields_joined}] as [{type_str}];")
            } else {
                writeln!(w, "return [{fields_joined}] as [{type_str}];")
            }
        }

        Format::TupleArray { content, size } => {
            if let Some(name) = field_name {
                write!(
                    w,
                    "const {name} = deserializeTupleArray(deserializer, {size}, (deserializer) => "
                )?;
            } else {
                write!(
                    w,
                    "return deserializeTupleArray(deserializer, {size}, (deserializer) => "
                )?;
            }
            with_block(w, Newlines::OPEN, |w| {
                write_deserialize(w, Some("item"), content)?;
                writeln!(w, "return [item];")
            })?;
            writeln!(w, ");")
        }

        Format::Variable(_) => panic!("unexpected variable in write_deserialize"),
        _ => unreachable!(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::generation::{
        CodeGeneratorConfig, Container, Feature,
        indent::{IndentConfig, IndentedWriter},
        plugin::EmitContext,
    };
    use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

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

    // -------------------------------------------------------------------------
    // module_helpers
    // -------------------------------------------------------------------------

    #[test]
    fn module_helpers_emit_list_of_t() {
        let cfg = make_config(&[Feature::ListOfT]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<TypeScript>;
        let out = render(|w| plugin.module_helpers(w, &cfg));
        assert!(
            out.contains("serializeArray"),
            "missing serializeArray:\n{out}"
        );
        assert!(
            out.contains("deserializeArray"),
            "missing deserializeArray:\n{out}"
        );
    }

    #[test]
    fn module_helpers_emit_option_of_t() {
        let cfg = make_config(&[Feature::OptionOfT]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<TypeScript>;
        let out = render(|w| plugin.module_helpers(w, &cfg));
        assert!(
            out.contains("serializeOption"),
            "missing serializeOption:\n{out}"
        );
        assert!(
            out.contains("deserializeOption"),
            "missing deserializeOption:\n{out}"
        );
    }

    #[test]
    fn module_helpers_emit_only_requested_features() {
        let cfg = make_config(&[Feature::ListOfT]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<TypeScript>;
        let out = render(|w| plugin.module_helpers(w, &cfg));
        assert!(
            !out.contains("serializeSet"),
            "unexpected serializeSet:\n{out}"
        );
        assert!(
            !out.contains("serializeMap"),
            "unexpected serializeMap:\n{out}"
        );
        assert!(
            !out.contains("serializeOption"),
            "unexpected serializeOption:\n{out}"
        );
        assert!(
            !out.contains("serializeTupleArray"),
            "unexpected serializeTupleArray:\n{out}"
        );
    }

    #[test]
    fn module_helpers_no_features_emits_nothing() {
        let cfg = make_config(&[]);
        let plugin = &BincodePlugin as &dyn EmitterPlugin<TypeScript>;
        let out = render(|w| plugin.module_helpers(w, &cfg));
        assert!(out.is_empty(), "expected empty output, got:\n{out}");
    }

    // -------------------------------------------------------------------------
    // has_type_body
    // -------------------------------------------------------------------------

    #[test]
    fn has_type_body_always_true() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<TypeScript>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        assert!(plugin.has_type_body(&ctx));
    }

    // -------------------------------------------------------------------------
    // type_body — struct shapes
    // -------------------------------------------------------------------------

    #[test]
    fn type_body_unit_struct() {
        let plugin = &BincodePlugin as &dyn EmitterPlugin<TypeScript>;

        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::UnitStruct(Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let out = render(|w| plugin.type_body(w, &ctx));
        assert!(
            out.contains("public serialize(serializer: Serializer): void"),
            "{out}"
        );
        assert!(
            out.contains("static deserialize(deserializer: Deserializer): Foo"),
            "{out}"
        );
        assert!(out.contains("return new Foo();"), "{out}");
    }

    #[test]
    fn type_body_struct_with_fields() {
        use crate::reflection::format::Format;
        let plugin = &BincodePlugin as &dyn EmitterPlugin<TypeScript>;

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

        let out = render(|w| plugin.type_body(w, &ctx));
        assert!(
            out.contains("serializer.serializeStr(this.label);"),
            "{out}"
        );
        assert!(
            out.contains("serializer.serializeI32(this.count);"),
            "{out}"
        );
        assert!(
            out.contains("const label = deserializer.deserializeStr();"),
            "{out}"
        );
        assert!(
            out.contains("const count = deserializer.deserializeI32();"),
            "{out}"
        );
        assert!(out.contains("return new MyStruct(label,count);"), "{out}");
    }

    // -------------------------------------------------------------------------
    // type_body — enum / variant shapes
    // -------------------------------------------------------------------------

    #[test]
    fn type_body_enum_with_variants() {
        use crate::reflection::format::VariantFormat;
        let plugin = &BincodePlugin as &dyn EmitterPlugin<TypeScript>;

        let mut variants = BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "Alpha".to_string()));
        variants.insert(1u32, Named::new(&VariantFormat::Unit, "Beta".to_string()));

        let name = QualifiedTypeName::root("MyEnum".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container);

        let out = render(|w| plugin.type_body(w, &ctx));
        assert!(
            out.contains("abstract serialize(serializer: Serializer): void;"),
            "{out}"
        );
        assert!(
            out.contains("static deserialize(deserializer: Deserializer): MyEnum"),
            "{out}"
        );
        assert!(
            out.contains("deserializer.deserializeVariantIndex()"),
            "{out}"
        );
        assert!(
            out.contains("case 0: return MyEnumVariantAlpha.load(deserializer);"),
            "{out}"
        );
        assert!(
            out.contains("case 1: return MyEnumVariantBeta.load(deserializer);"),
            "{out}"
        );
    }

    #[test]
    fn type_body_variant_subclass() {
        use crate::reflection::format::{Format, VariantFormat};
        let plugin = &BincodePlugin as &dyn EmitterPlugin<TypeScript>;

        let mut variants = BTreeMap::new();
        let variant_fields = vec![Named::new(&Format::Str, "value".to_string())];
        variants.insert(
            0u32,
            Named::new(
                &VariantFormat::Struct(variant_fields.clone()),
                "Ok".to_string(),
            ),
        );

        let name = QualifiedTypeName::root("Result".to_string());
        let format = ContainerFormat::Enum(variants, Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };

        let variant_info = crate::generation::plugin::VariantInfo {
            name: "Ok",
            index: 0,
            format: &VariantFormat::Struct(variant_fields.clone()),
            fields: &variant_fields,
            parent_name: "Result",
        };
        let ctx = EmitContext::for_variant(&container, variant_info);

        let out = render(|w| plugin.type_body(w, &ctx));
        assert!(
            out.contains("serializer.serializeVariantIndex(0);"),
            "{out}"
        );
        assert!(
            out.contains("serializer.serializeStr(this.value);"),
            "{out}"
        );
        assert!(
            out.contains("static load(deserializer: Deserializer): ResultVariantOk"),
            "{out}"
        );
        assert!(out.contains("return new ResultVariantOk(value);"), "{out}");
    }
}
