//! `EmitterPlugin<Kotlin>` implementation for the [`BincodePlugin`].
//!
//! Provides bincode-specific imports, feature helper snippets, and type-body
//! generation (serialize / deserialize method bodies) for Kotlin code
//! generation.
//!
//! The 12 helper functions (`write_bincode_serialize`, `write_serialize`, etc.)
//! that used to live in the Kotlin emitter are now canonical here and
//! re-exported as `pub(crate)` so the emitter can still reference them during
//! the transition period (the `is_bincode()` branches in the emitter will be
//! removed in a follow-up step).

use std::io::{self, Result, Write};

use heck::ToLowerCamelCase;
use indoc::writedoc;

use crate::generation::{
    CodeGeneratorConfig, Feature,
    indent::{IndentWrite, IndentedWriter, Newlines},
    kotlin::Kotlin,
    plugin::{EmitContext, EmitterPlugin},
};
use crate::reflection::format::{ContainerFormat, Format, Named, VariantFormat};

use super::BincodePlugin;

// Bincode container helper snippets — inlined Kotlin source fragments
// (extension functions on `Serializer` / `Deserializer`) that teach the serde
// runtime how to handle generic containers.  Written into the module header
// when the corresponding [`Feature`] flag is active.
const FEATURE_LIST_OF_T: &str = r"fun <T> List<T>.serialize(
    serializer: Serializer,
    serializeElement: Serializer.(T) -> Unit,
) {
    serializer.serialize_len(size.toLong())
    forEach { element ->
        serializer.serializeElement(element)
    }
}

fun <T> Deserializer.deserializeListOf(deserializeElement: (Deserializer) -> T): List<T> {
    val length = deserialize_len()
    val list = mutableListOf<T>()
    repeat(length.toInt()) {
        list.add(deserializeElement(this))
    }
    return list
}
";

const FEATURE_MAP_OF_T: &str = r"fun <K, V> Map<K, V>.serialize(
    serializer: Serializer,
    serializeEntry: Serializer.(K, V) -> Unit,
) {
    serializer.serialize_len(size.toLong())
    forEach { (key, value) ->
        serializer.serializeEntry(key, value)
    }
}

fun <K, V> Deserializer.deserializeMapOf(deserializeEntry: (Deserializer) -> Pair<K, V>): Map<K, V> {
    val length = deserialize_len()
    val map = mutableMapOf<K, V>()
    repeat(length.toInt()) {
        val (key, value) = deserializeEntry(this)
        map[key] = value
    }
    return map
}
";

const FEATURE_OPTION_OF_T: &str = r"fun <T> T?.serializeOptionOf(
    serializer: Serializer,
    serializeElement: Serializer.(T) -> Unit,
) {
    if (this != null) {
        serializer.serialize_option_tag(true)
        serializer.serializeElement(this)
    } else {
        serializer.serialize_option_tag(false)
    }
}

fun <T> Deserializer.deserializeOptionOf(deserializeElement: (Deserializer) -> T): T? {
    val tag = deserialize_option_tag()
    return if (tag) {
        deserializeElement(this)
    } else {
        null
    }
}
";

const FEATURE_SET_OF_T: &str = r"fun <T> Set<T>.serialize(
    serializer: Serializer,
    serializeElement: Serializer.(T) -> Unit,
) {
    serializer.serialize_len(size.toLong())
    forEach { element ->
        serializer.serializeElement(element)
    }
}

fun <T> Deserializer.deserializeSetOf(deserializeElement: (Deserializer) -> T): Set<T> {
    val length = deserialize_len()
    val set = mutableSetOf<T>()
    repeat(length.toInt()) {
        set.add(deserializeElement(this))
    }
    return set
}
";

fn write_bincode_serialize<W: Write>(w: &mut W) -> Result<()> {
    writedoc!(
        w,
        r"
        fun bincodeSerialize(): ByteArray {{
            val serializer = BincodeSerializer()
            serialize(serializer)
            return serializer.get_bytes()
        }}
        "
    )
}

fn write_bincode_deserialize<W: Write>(w: &mut W, name: &str) -> Result<()> {
    writedoc!(
        w,
        r#"
        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): {name} {{
            if (input == null) {{
                throw DeserializationError("Cannot deserialize null array")
            }}
            val deserializer = BincodeDeserializer(input)
            val value = deserialize(deserializer)
            if (deserializer.get_buffer_offset() < input.size) {{
                throw DeserializationError("Some input bytes were not read")
            }}
            return value
        }}
        "#
    )
}

fn write_serialize<W: IndentWrite>(
    w: &mut W,
    field_name: &str,
    format: &Format,
    level: usize,
) -> Result<()> {
    match format {
        Format::Unit => writeln!(w, "serializer.serialize_unit({field_name})"),
        Format::Bool => writeln!(w, "serializer.serialize_bool({field_name})"),
        Format::I8 => writeln!(w, "serializer.serialize_i8({field_name})"),
        Format::I16 => writeln!(w, "serializer.serialize_i16({field_name})"),
        Format::I32 => writeln!(w, "serializer.serialize_i32({field_name})"),
        Format::I64 => writeln!(w, "serializer.serialize_i64({field_name})"),
        Format::I128 => writeln!(w, "serializer.serialize_i128({field_name})"),
        Format::U8 => writeln!(w, "serializer.serialize_u8({field_name})"),
        Format::U16 => writeln!(w, "serializer.serialize_u16({field_name})"),
        Format::U32 => writeln!(w, "serializer.serialize_u32({field_name})"),
        Format::U64 => writeln!(w, "serializer.serialize_u64({field_name})"),
        Format::U128 => writeln!(w, "serializer.serialize_u128({field_name})"),
        Format::F32 => writeln!(w, "serializer.serialize_f32({field_name})"),
        Format::F64 => writeln!(w, "serializer.serialize_f64({field_name})"),
        Format::Char => writeln!(w, "serializer.serialize_char({field_name})"),
        Format::Str => writeln!(w, "serializer.serialize_str({field_name})"),
        Format::Bytes => writeln!(w, "serializer.serialize_bytes({field_name})"),

        Format::Option(inner_format) => {
            write!(w, "{field_name}.serializeOptionOf(serializer) ")?;
            write_serialize_lambda(w, inner_format, level)?;
            Ok(())
        }

        Format::Seq(inner_format) | Format::Set(inner_format) => {
            write!(w, "{field_name}.serialize(serializer) ")?;
            write_serialize_lambda(w, inner_format, level)?;
            Ok(())
        }

        Format::Map { key, value } => {
            write!(w, "{field_name}.serialize(serializer) ")?;
            write_map_serialize_lambda(w, key, value, level)?;
            Ok(())
        }

        Format::TypeName(..) | Format::TupleArray { .. } => {
            writeln!(w, "{field_name}.serialize(serializer)")
        }

        Format::Tuple(formats) => {
            let len = formats.len();
            match len {
                0 => writeln!(w, "serializer.serialize_unit({field_name})"),
                1 => write_serialize(w, field_name, &formats[0], level),
                2 => {
                    write_serialize(w, &format!("{field_name}.first"), &formats[0], level)?;
                    write_serialize(w, &format!("{field_name}.second"), &formats[1], level)
                }
                3 => {
                    write_serialize(w, &format!("{field_name}.first"), &formats[0], level)?;
                    write_serialize(w, &format!("{field_name}.second"), &formats[1], level)?;
                    write_serialize(w, &format!("{field_name}.third"), &formats[2], level)
                }
                _ => {
                    for (i, format) in formats.iter().enumerate() {
                        write_serialize(
                            w,
                            &format!("{field_name}.component{}()", i + 1),
                            format,
                            level,
                        )?;
                    }
                    Ok(())
                }
            }
        }
        Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
    }
}

fn write_serialize_lambda<W: IndentWrite>(w: &mut W, format: &Format, level: usize) -> Result<()> {
    if format.is_leaf() {
        let mut w = w.block(Newlines::BOTH)?;
        write_serialize(&mut w, "it", format, level + 1)
    } else {
        let param_name = format!("level{}", level + 1);
        let mut w = w.block(Newlines::CLOSE)?;
        writeln!(w, " {param_name} ->")?;
        write_serialize(&mut w, &param_name, format, level + 1)
    }
}

fn write_map_serialize_lambda<W: IndentWrite>(
    w: &mut W,
    key_format: &Format,
    value_format: &Format,
    level: usize,
) -> Result<()> {
    let mut w = w.block(Newlines::CLOSE)?;
    writeln!(w, " key, value ->")?;

    write_serialize(&mut w, "key", key_format, level + 1)?;
    write_serialize(&mut w, "value", value_format, level + 1)
}

#[allow(clippy::too_many_lines)]
fn write_deserialize<W: IndentWrite>(
    w: &mut W,
    field_name: Option<&str>,
    format: &Format,
    newline: bool,
) -> Result<()> {
    let mut indented = false;
    if let Some(field_name) = field_name {
        write!(w, "val {field_name} =")?;
        if matches!(
            format,
            Format::Seq(..) | Format::Option(..) | Format::Set(..) | Format::Map { .. }
        ) {
            writeln!(w)?;
            w.indent();
            indented = true;
        } else {
            write!(w, " ")?;
        }
    }
    match format {
        Format::TypeName(qualified_name) => {
            let fully_qualified_name = qualified_name.format(ToString::to_string, ".");
            write!(w, "{fully_qualified_name}.deserialize(deserializer)")
        }
        Format::Unit => write!(w, "deserializer.deserialize_unit()"),
        Format::Bool => write!(w, "deserializer.deserialize_bool()"),
        Format::I8 => write!(w, "deserializer.deserialize_i8()"),
        Format::I16 => write!(w, "deserializer.deserialize_i16()"),
        Format::I32 => write!(w, "deserializer.deserialize_i32()"),
        Format::I64 => write!(w, "deserializer.deserialize_i64()"),
        Format::I128 => write!(w, "deserializer.deserialize_i128()"),
        Format::U8 => write!(w, "deserializer.deserialize_u8()"),
        Format::U16 => write!(w, "deserializer.deserialize_u16()"),
        Format::U32 => write!(w, "deserializer.deserialize_u32()"),
        Format::U64 => write!(w, "deserializer.deserialize_u64()"),
        Format::U128 => write!(w, "deserializer.deserialize_u128()"),
        Format::F32 => write!(w, "deserializer.deserialize_f32()"),
        Format::F64 => write!(w, "deserializer.deserialize_f64()"),
        Format::Char => write!(w, "deserializer.deserialize_char()"),
        Format::Str => write!(w, "deserializer.deserialize_str()"),
        Format::Bytes => write!(w, "deserializer.deserialize_bytes()"),
        Format::Seq(format) => {
            write!(w, "deserializer.deserializeListOf ")?;
            write_deserialize_lambda(w, format)
        }
        Format::Option(format) => {
            write!(w, "deserializer.deserializeOptionOf ")?;
            write_deserialize_lambda(w, format)
        }
        Format::Set(format) => {
            write!(w, "deserializer.deserializeSetOf ")?;
            write_deserialize_lambda(w, format)
        }
        Format::Map { key, value } => {
            write!(w, "deserializer.deserializeMapOf ")?;
            write_map_deserialize_lambda(w, key, value)
        }
        Format::Tuple(formats) => {
            let len = formats.len();
            match len {
                0 => {
                    write!(w, "deserializer.deserialize_unit()")?;
                    return Ok(());
                }
                1 => {
                    push_deserializer(w)?;
                    write_deserialize(w, Some("value"), &formats[0], true)?;
                    pop_deserializer(w)?;
                    return Ok(());
                }
                2 => {
                    write!(w, "run ")?;
                    let mut w = w.block(Newlines::BOTH)?;
                    write!(w, "val first = ")?;
                    write_deserialize(&mut w, None, &formats[0], true)?;
                    write!(w, "val second = ")?;
                    write_deserialize(&mut w, None, &formats[1], true)?;
                    writeln!(w, "Pair(first, second)")?;
                }
                3 => {
                    write!(w, "run ")?;
                    let mut w = w.block(Newlines::BOTH)?;
                    write!(w, "val first = ")?;
                    write_deserialize(&mut w, None, &formats[0], true)?;
                    write!(w, "val second = ")?;
                    write_deserialize(&mut w, None, &formats[1], true)?;
                    write!(w, "val third = ")?;
                    write_deserialize(&mut w, None, &formats[2], true)?;
                    writeln!(w, "Triple(first, second, third)")?;
                }
                _ => {
                    let typename = format!("NTuple{len}");
                    write!(w, "run ")?;
                    let mut w = w.block(Newlines::BOTH)?;
                    for (i, format) in formats.iter().enumerate() {
                        write!(w, "val v{i} = ")?;
                        write_deserialize(&mut w, None, format, true)?;
                    }
                    write!(w, "{typename}(")?;
                    for i in 0..len {
                        if i > 0 {
                            write!(w, ", ")?;
                        }
                        write!(w, "v{i}")?;
                    }
                    writeln!(w, ")")?;
                }
            }
            Ok(())
        }
        Format::TupleArray { content, size } => {
            write!(w, "buildList({size}) {{ repeat({size}) {{ add(")?;
            write_deserialize(w, None, content, false)?;
            write!(w, ") }} }}")
        }
        Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
    }?;

    if newline
        && !matches!(
            format,
            Format::Seq(..)
                | Format::Option(..)
                | Format::Set(..)
                | Format::Map { .. }
                | Format::Tuple(..)
        )
    {
        writeln!(w)?;
    }

    if indented {
        w.unindent();
    }

    Ok(())
}

fn write_deserialize_lambda<W: IndentWrite>(w: &mut W, format: &Format) -> Result<()> {
    let mut w = w.block(Newlines::BOTH)?;
    write_deserialize(&mut w, None, format, true)
}

fn write_map_deserialize_lambda<W: IndentWrite>(
    w: &mut W,
    key_format: &Format,
    value_format: &Format,
) -> Result<()> {
    let mut w = w.block(Newlines::BOTH)?;
    write!(w, "val key =")?;
    if key_format.is_leaf() {
        write!(w, " ")?;
        write_deserialize(&mut w, None, key_format, true)?;
    } else {
        writeln!(w)?;
        w.indent();
        write_deserialize(&mut w, None, key_format, true)?;
        w.unindent();
    }
    write!(w, "val value =")?;
    if value_format.is_leaf() {
        write!(w, " ")?;
        write_deserialize(&mut w, None, value_format, true)?;
    } else {
        writeln!(w)?;
        w.indent();
        write_deserialize(&mut w, None, value_format, true)?;
        w.unindent();
    }
    writeln!(w, "Pair(key, value)")
}

fn push_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "serializer.increase_container_depth()")
}

fn pop_serializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "serializer.decrease_container_depth()")
}

fn push_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "deserializer.increase_container_depth()")
}

fn pop_deserializer<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w, "deserializer.decrease_container_depth()")
}

/// Write the bincode type body for a top-level `data object` (unit struct or
/// empty struct).
fn write_data_object_top_level<W: IndentWrite>(w: &mut W, name: &str) -> Result<()> {
    write!(w, "fun serialize(serializer: Serializer) ")?;
    let _ = w.block(Newlines::CLOSE)?;
    writeln!(w)?;

    write_bincode_serialize(w)?;
    writeln!(w)?;

    write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "return {name}")?;
    }
    writeln!(w)?;
    write_bincode_deserialize(w, name)?;
    Ok(())
}

/// Write the bincode type body for a variant `data object` (unit variant
/// inside a sealed interface).
fn write_data_object_variant<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variant_index: usize,
) -> Result<()> {
    write!(w, "override fun serialize(serializer: Serializer) ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        push_serializer(&mut w)?;
        writeln!(w, "serializer.serialize_variant_index({variant_index})")?;
        pop_serializer(&mut w)?;
    }
    writeln!(w)?;

    write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "return {name}")?;
    }
    Ok(())
}

/// Write the bincode type body for a top-level `data class` (struct with
/// fields).
fn write_data_class_top_level<W: IndentWrite>(
    w: &mut W,
    name: &str,
    fields: &[Named<Format>],
) -> Result<()> {
    // serialize
    write!(w, "fun serialize(serializer: Serializer) ")?;
    if fields.is_empty() {
        let _ = w.block(Newlines::CLOSE)?;
    } else {
        let mut w = w.block(Newlines::BOTH)?;
        push_serializer(&mut w)?;
        for field in fields {
            write_serialize(&mut w, &field.name.to_lower_camel_case(), &field.value, 0)?;
        }
        pop_serializer(&mut w)?;
    }
    writeln!(w)?;

    write_bincode_serialize(w)?;
    writeln!(w)?;

    // companion object
    write!(w, "companion object ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            if fields.is_empty() {
                writeln!(w, "return {name}()")?;
            } else {
                push_deserializer(&mut w)?;
                for field in fields {
                    write_deserialize(
                        &mut w,
                        Some(&field.name.to_lower_camel_case()),
                        &field.value,
                        true,
                    )?;
                }
                pop_deserializer(&mut w)?;
                write!(w, "return {name}(")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    write!(w, "{}", field.name.to_lower_camel_case())?;
                }
                writeln!(w, ")")?;
            }
        }
        writeln!(w)?;
        write_bincode_deserialize(&mut w, name)?;
    }
    Ok(())
}

/// Write the bincode type body for a variant `data class` (variant with
/// fields inside a sealed interface).
fn write_data_class_variant<W: IndentWrite>(
    w: &mut W,
    name: &str,
    fields: &[Named<Format>],
    variant_index: usize,
) -> Result<()> {
    // serialize (override)
    write!(w, "override fun serialize(serializer: Serializer) ")?;
    if fields.is_empty() {
        let _ = w.block(Newlines::CLOSE)?;
    } else {
        let mut w = w.block(Newlines::BOTH)?;
        push_serializer(&mut w)?;
        writeln!(w, "serializer.serialize_variant_index({variant_index})")?;
        for field in fields {
            write_serialize(&mut w, &field.name.to_lower_camel_case(), &field.value, 0)?;
        }
        pop_serializer(&mut w)?;
    }
    writeln!(w)?;

    // companion object (deserialize only, no bincodeDeserialize)
    write!(w, "companion object ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            if fields.is_empty() {
                writeln!(w, "return {name}()")?;
            } else {
                push_deserializer(&mut w)?;
                for field in fields {
                    write_deserialize(
                        &mut w,
                        Some(&field.name.to_lower_camel_case()),
                        &field.value,
                        true,
                    )?;
                }
                pop_deserializer(&mut w)?;
                write!(w, "return {name}(")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(w, ", ")?;
                    }
                    write!(w, "{}", field.name.to_lower_camel_case())?;
                }
                writeln!(w, ")")?;
            }
        }
    }
    Ok(())
}

/// Write the bincode type body for an all-unit `enum class`.
fn write_enum_class_body<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &std::collections::BTreeMap<u32, Named<VariantFormat>>,
) -> Result<()> {
    writeln!(w)?;
    write!(w, "fun serialize(serializer: Serializer) ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        push_serializer(&mut w)?;
        writeln!(w, "serializer.serialize_variant_index(ordinal)")?;
        pop_serializer(&mut w)?;
    }
    writeln!(w)?;

    write_bincode_serialize(w)?;
    writeln!(w)?;

    write!(w, "companion object ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;

        writeln!(w, "@Throws(DeserializationError::class)")?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            push_deserializer(&mut w)?;
            writeln!(w, "val index = deserializer.deserialize_variant_index()")?;
            pop_deserializer(&mut w)?;
            write!(w, "return when (index) ")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                for (i, variant) in variants {
                    let upper = variant.name.to_uppercase();
                    writeln!(w, "{i} -> {upper}")?;
                }
                writeln!(
                    w,
                    r#"else -> throw DeserializationError("Unknown variant index for {name}: $index")"#
                )?;
            }
        }
        writeln!(w)?;
        write_bincode_deserialize(&mut w, name)?;
    }
    Ok(())
}

/// Write the bincode type body (companion object) for a `sealed interface`,
/// emitted after all variants.
fn write_sealed_interface_body<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &std::collections::BTreeMap<u32, Named<VariantFormat>>,
) -> Result<()> {
    writeln!(w)?;
    write!(w, "companion object ")?;
    {
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "@Throws(DeserializationError::class)")?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, "val index = deserializer.deserialize_variant_index()")?;
            write!(w, "return when (index) ")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                for (i, variant) in variants {
                    let vname = &variant.name;
                    writeln!(w, "{i} -> {vname}.deserialize(deserializer)")?;
                }
                writeln!(
                    w,
                    r#"else -> throw DeserializationError("Unknown variant index for {name}: $index")"#
                )?;
            }
        }

        writeln!(w)?;
        write_bincode_deserialize(&mut w, name)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// EmitterPlugin implementation
// ---------------------------------------------------------------------------

impl EmitterPlugin<Kotlin> for BincodePlugin {
    /// Bincode / serde imports for a Kotlin module.
    ///
    /// Returns the base set of imports that every bincode-enabled module
    /// needs, plus feature-specific imports (e.g. `Bytes`, `Int128`).
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let bp = &self.bincode_package;
        let sp = &self.serde_package;

        let mut imports = vec![
            format!("import {bp}.BincodeDeserializer"),
            format!("import {bp}.BincodeSerializer"),
            format!("import {sp}.DeserializationError"),
            format!("import {sp}.Deserializer"),
            format!("import {sp}.Serializer"),
        ];

        // Feature-driven imports
        for feature in &config.features {
            match feature {
                Feature::Bytes => {
                    imports.push(format!("import {sp}.Bytes"));
                }
                Feature::BigInt => {
                    // BigInteger is JVM-only; kept for backward compat.
                    imports.push("import java.math.BigInteger".to_string());
                    imports.push(format!("import {sp}.Int128"));
                }
                // Other features add helper *code* (via module_helpers),
                // not imports.
                _ => {}
            }
        }

        imports
    }

    /// Bincode feature helper snippets for a Kotlin module.
    ///
    /// These are small Kotlin source fragments (extension functions on
    /// `Serializer` / `Deserializer`) that teach the serde runtime how to
    /// handle generic containers (`List<T>`, `Set<T>`, `Map<K,V>`,
    /// `Optional<T>`).  They are written into the module header, after
    /// imports but before any type declarations.
    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        for feature in &config.features {
            match feature {
                Feature::ListOfT => {
                    write!(w, "{FEATURE_LIST_OF_T}")?;
                    writeln!(w)?;
                }
                Feature::OptionOfT => {
                    write!(w, "{FEATURE_OPTION_OF_T}")?;
                    writeln!(w)?;
                }
                Feature::SetOfT => {
                    write!(w, "{FEATURE_SET_OF_T}")?;
                    writeln!(w)?;
                }
                Feature::MapOfT => {
                    write!(w, "{FEATURE_MAP_OF_T}")?;
                    writeln!(w)?;
                }
                // BigInt and Bytes add imports (handled above); TupleArray is
                // encoding-independent and stays in the emitter.
                _ => {}
            }
        }
        Ok(())
    }

    /// Bincode always contributes a type body (serialize/deserialize methods).
    fn has_type_body(&self, _ctx: &EmitContext) -> bool {
        true
    }

    /// Preamble for sealed interfaces: abstract `serialize` declaration and
    /// `bincodeSerialize()` convenience wrapper.
    ///
    /// Only emits code for non-all-unit `Enum` containers at top level (i.e.
    /// sealed interfaces). All other entity types get their serialize
    /// declarations from [`type_body`](Self::type_body).
    fn type_body_preamble(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        if ctx.is_variant() {
            return Ok(());
        }

        if let ContainerFormat::Enum(variants, _) = ctx.container.format {
            let all_unit = variants
                .values()
                .all(|v| matches!(v.value, VariantFormat::Unit));

            if !all_unit {
                // Sealed interface preamble
                {
                    let config = w.config();
                    let mut iw = IndentedWriter::new(&mut *w, config);
                    writeln!(iw, "fun serialize(serializer: Serializer)")?;
                    writeln!(iw)?;
                    write_bincode_serialize(&mut iw)?;
                    writeln!(iw)?;
                }
            }
        }

        Ok(())
    }

    /// Main type-body generation for bincode.
    ///
    /// Produces serialize / deserialize methods (and companion objects) for
    /// every kind of Kotlin entity: `data object`, `data class`,
    /// `enum class`, and `sealed interface`.
    ///
    /// # Note on the transition period
    ///
    /// The Kotlin emitter's `data_object`, `data_class`, `enum_class`, and
    /// `sealed_interface` functions still contain their own `is_bincode()`
    /// branches that produce the same code directly. During this transition
    /// only the `enum_class` and `sealed_interface` emitter functions call
    /// `plugin.type_body()`, so this method skips those two cases to avoid
    /// duplication. Once the `is_bincode()` branches are removed and all
    /// emitter functions call the plugin hooks, the guard will be removed.
    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        let name = ctx.name();
        let fields = ctx.fields();

        // ---- Variant inside a sealed interface ----
        //
        // Check variant first, because callers (data_class / data_object)
        // may construct a temporary Container with a non-Enum format while
        // still setting `ctx.variant`.
        if let Some(variant_info) = &ctx.variant {
            let variant_index = variant_info.index;

            {
                let config = w.config();
                let mut iw = IndentedWriter::new(&mut *w, config);
                if fields.is_empty() {
                    write_data_object_variant(&mut iw, name, variant_index)?;
                } else {
                    write_data_class_variant(&mut iw, name, &fields, variant_index)?;
                }
            }
            return Ok(());
        }

        // ---- Top-level enum (enum class / sealed interface) ----
        if let ContainerFormat::Enum(variants, _) = ctx.container.format {
            let all_unit = variants
                .values()
                .all(|v| matches!(v.value, VariantFormat::Unit));

            {
                let config = w.config();
                let mut iw = IndentedWriter::new(&mut *w, config);
                if all_unit {
                    write_enum_class_body(&mut iw, name, variants)?;
                } else {
                    write_sealed_interface_body(&mut iw, name, variants)?;
                }
            }
            return Ok(());
        }

        // ---- Non-enum containers (data object / data class) ----
        {
            let config = w.config();
            let mut iw = IndentedWriter::new(&mut *w, config);
            if fields.is_empty() {
                write_data_object_top_level(&mut iw, name)?;
            } else {
                write_data_class_top_level(&mut iw, name, &fields)?;
            }
        }

        Ok(())
    }
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
    fn base_imports_are_present() {
        let cfg = make_config(&[]);
        let plugin = BincodePlugin::from_config(&cfg);
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("BincodeSerializer")));
        assert!(imports.iter().any(|i| i.contains("BincodeDeserializer")));
        assert!(imports.iter().any(|i| i.contains("Serializer")));
        assert!(imports.iter().any(|i| i.contains("Deserializer")));
        assert!(imports.iter().any(|i| i.contains("DeserializationError")));
    }

    #[test]
    fn bytes_feature_adds_import() {
        let cfg = make_config(&[Feature::Bytes]);
        let plugin = BincodePlugin::from_config(&cfg);
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("Bytes")));
    }

    #[test]
    fn bigint_feature_adds_imports() {
        let cfg = make_config(&[Feature::BigInt]);
        let plugin = BincodePlugin::from_config(&cfg);
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("BigInteger")));
        assert!(imports.iter().any(|i| i.contains("Int128")));
    }

    #[test]
    fn module_helpers_emit_list_of_t() {
        let cfg = make_config(&[Feature::ListOfT]);
        let plugin = BincodePlugin::from_config(&cfg);

        let mut buf = Vec::new();
        {
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin.module_helpers(&mut w, &cfg).unwrap();
        }

        let output = String::from_utf8(buf).unwrap();
        // The ListOfT.kt snippet defines extension functions — just check
        // it's non-empty and contains a recognizable marker.
        assert!(!output.is_empty());
    }

    #[test]
    fn has_type_body_always_true() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

        let cfg = make_config(&[]);
        let plugin = BincodePlugin::from_config(&cfg);

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
    fn type_body_preamble_sealed_interface() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, Format, QualifiedTypeName};
        use std::collections::BTreeMap;

        let cfg = make_config(&[]);
        let plugin = BincodePlugin::from_config(&cfg);

        // Non-all-unit enum → sealed interface
        let mut variants = BTreeMap::new();
        variants.insert(
            0,
            Named {
                name: "A".to_string(),
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
                .type_body_preamble(&mut w as &mut dyn IndentWrite, &ctx)
                .unwrap();
        }
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("fun serialize(serializer: Serializer)"));
        assert!(output.contains("fun bincodeSerialize(): ByteArray"));
    }

    #[test]
    fn type_body_preamble_noop_for_enum_class() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};
        use std::collections::BTreeMap;

        let cfg = make_config(&[]);
        let plugin = BincodePlugin::from_config(&cfg);

        // All-unit enum → enum class
        let mut variants = BTreeMap::new();
        variants.insert(
            0,
            Named {
                name: "A".to_string(),
                doc: Doc::default(),
                value: VariantFormat::Unit,
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
                .type_body_preamble(&mut w as &mut dyn IndentWrite, &ctx)
                .unwrap();
        }
        let output = String::from_utf8(buf).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn type_body_data_object_top_level() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

        let cfg = make_config(&[]);
        let plugin = BincodePlugin::from_config(&cfg);

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
        assert!(output.contains("fun serialize(serializer: Serializer)"));
        assert!(output.contains("fun bincodeSerialize(): ByteArray"));
        assert!(output.contains("fun deserialize(deserializer: Deserializer): UnitStruct"));
        assert!(output.contains("fun bincodeDeserialize(input: ByteArray?): UnitStruct"));
    }

    #[test]
    fn type_body_data_class_top_level() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

        let cfg = make_config(&[]);
        let plugin = BincodePlugin::from_config(&cfg);

        let name = QualifiedTypeName::root("MyStruct".to_string());
        let fields = vec![
            Named::new(&Format::Str, "name".to_string()),
            Named::new(&Format::I32, "age".to_string()),
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
        assert!(output.contains("fun serialize(serializer: Serializer)"));
        assert!(output.contains("serializer.serialize_str(name)"));
        assert!(output.contains("serializer.serialize_i32(age)"));
        assert!(output.contains("companion object"));
        assert!(output.contains("fun deserialize(deserializer: Deserializer): MyStruct"));
        assert!(output.contains("return MyStruct(name, age)"));
        assert!(output.contains("fun bincodeDeserialize(input: ByteArray?): MyStruct"));
    }

    #[test]
    fn type_body_enum_top_level_skips_to_avoid_duplication() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};
        use std::collections::BTreeMap;

        let cfg = make_config(&[]);
        let plugin = BincodePlugin::from_config(&cfg);

        let mut variants = BTreeMap::new();
        variants.insert(
            0,
            Named {
                name: "A".to_string(),
                doc: Doc::default(),
                value: VariantFormat::Unit,
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
        // Now that the emitter delegates to the plugin, top-level enums
        // produce their serialize/deserialize/companion-object body here.
        assert!(!output.is_empty());
        assert!(output.contains("fun serialize(serializer: Serializer)"));
        assert!(output.contains("fun deserialize(deserializer: Deserializer)"));
    }
}
