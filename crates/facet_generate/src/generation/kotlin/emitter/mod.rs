//! AST-to-Kotlin source rendering.
//!
//! This module implements [`Emitter<Kotlin>`](super::super::Emitter) for each
//! node type in the format AST, turning abstract type descriptions into
//! idiomatic Kotlin code.
//!
//! # Emitter implementations
//!
//! | AST node | Kotlin output |
//! |---|---|
//! | [`Module`] | `package` declaration, `import` statements, feature helpers |
//! | [`Container`] | `data class`, `data object`, `sealed interface`, or `enum class` |
//! | [`Named<Format>`](Named) | A single `val` property declaration |
//! | [`Format`] | An inline type expression (`Int`, `List<String>`, `Pair<A, B>`, …) |
//! | [`Doc`] | `///` doc comments |
//! | `(Named<VariantFormat>, VariantContext)` | An enum/sealed-interface variant |
//!
//! # Kotlin type mapping
//!
//! The [`Format`] emitter maps Rust/reflection types to Kotlin equivalents —
//! for example `I32` → `Int`, `Seq(T)` → `List<T>`, `Option(T)` → `T?`,
//! tuples of size 2/3 → `Pair`/`Triple`, and larger tuples to `NTupleN<…>`.
//!
//! # Encoding-dependent output
//!
//! The [`Kotlin`] language tag carries the active [`Encoding`]. When encoding
//! is `Json`, types get `@Serializable` / `@SerialName` annotations. When
//! encoding is `Bincode`, each type gets `serialize` / `deserialize` methods
//! and convenience `bincodeSerialize` / `bincodeDeserialize` wrappers. When
//! encoding is `None`, only plain type declarations are emitted.
//!
//! # Feature helpers (`features/` directory)
//!
//! Kotlin has `List`, `Set`, `Map`, and nullable types built in, but the
//! bincode `Serializer`/`Deserializer` runtime only handles primitives and
//! user-defined types (which get their own `serialize`/`deserialize` methods).
//! The feature helpers are Kotlin extension functions that bridge this gap —
//! they teach the serde runtime how to length-prefix and iterate over generic
//! containers.
//!
//! For example, the generated code for a `List<Foo>` field calls:
//! ```kotlin
//! myList.serialize(serializer) { it.serialize(serializer) }
//! ```
//! where `List<T>.serialize` (from `ListOfT.kt`) writes the length, then
//! delegates each element to the lambda.
//!
//! | Helper | What it provides | When included |
//! |---|---|---|
//! | `ListOfT.kt` | `List<T>.serialize` / `Deserializer.deserializeListOf` | Bincode + `Seq` type used |
//! | `SetOfT.kt` | `Set<T>.serialize` / `Deserializer.deserializeSetOf` | Bincode + `Set` type used |
//! | `MapOfT.kt` | `Map<K,V>.serialize` / `Deserializer.deserializeMapOf` | Bincode + `Map` type used |
//! | `OptionOfT.kt` | `T?.serializeOptionOf` / `Deserializer.deserializeOptionOf` | Bincode + `Option` type used |
//! | `BigInt.kt` | `KSerializer<BigInteger>` for kotlinx.serialization | JSON + `I128`/`U128` type used |
//! | `TupleArray.kt` | `buildList` polyfill for Kotlin < 1.6.0 | `TupleArray` type used (any encoding) |
//!
//! These `.kt` snippets are embedded at compile time via `include_bytes!` and
//! written into the file header by the [`Module`] emitter when the
//! corresponding [`Feature`] flag is active (discovered automatically by
//! [`CodeGeneratorConfig::update_from`]).

use std::{
    collections::BTreeMap,
    io::{Result, Write},
    string::ToString,
};

use heck::ToLowerCamelCase;
use indoc::writedoc;

use crate::{
    generation::{
        BINCODE_NAMESPACE, CodeGeneratorConfig, Container, Emitter, Encoding, Feature,
        PackageLocation, SERDE_NAMESPACE,
        indent::{IndentWrite, Newlines},
        module::Module,
    },
    reflection::format::{ContainerFormat, Doc, Format, Named, QualifiedTypeName, VariantFormat},
};

const FEATURE_BIGINT: &[u8] = include_bytes!("features/BigInt.kt");
const FEATURE_LIST_OF_T: &[u8] = include_bytes!("features/ListOfT.kt");
const FEATURE_MAP_OF_T: &[u8] = include_bytes!("features/MapOfT.kt");
const FEATURE_OPTION_OF_T: &[u8] = include_bytes!("features/OptionOfT.kt");
const FEATURE_SET_OF_T: &[u8] = include_bytes!("features/SetOfT.kt");
const FEATURE_TUPLE_ARRAY: &[u8] = include_bytes!("features/TupleArray.kt");

/// Language tag for Kotlin code generation.
///
/// Passed as the `L` parameter to every [`Emitter<L>`](super::super::Emitter)
/// call. Carries the target [`Encoding`] so emitters can conditionally
/// produce serialization code.
#[derive(Debug, Clone)]
pub struct Kotlin {
    pub encoding: Encoding,
}

impl Kotlin {
    #[must_use]
    pub fn new(encoding: Encoding) -> Self {
        Self { encoding }
    }

    /// Create a [`Kotlin`] language tag for the given encoding and registry.
    ///
    /// Currently delegates to [`new`](Self::new) — the registry is not used
    /// for Kotlin generation but the method signature mirrors [`Swift::for_encoding`]
    /// so that the `emit!` test macro can call a uniform constructor.
    #[must_use]
    pub fn for_encoding(encoding: Encoding, _registry: &crate::Registry) -> Self {
        Self::new(encoding)
    }
}

impl Module {
    fn get_package_name(&self, namespace: &str) -> String {
        let external_packages = &self.config().external_packages;
        external_packages
            .get(namespace)
            .and_then(|pkg| {
                if let PackageLocation::Path(path) = &pkg.location {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("com.novi.{namespace}"))
    }

    fn bincode_package(&self) -> String {
        self.get_package_name(BINCODE_NAMESPACE)
    }

    fn serde_package(&self) -> String {
        self.get_package_name(SERDE_NAMESPACE)
    }
}

impl Emitter<Kotlin> for Module {
    #[allow(clippy::too_many_lines)]
    fn write<W: Write>(&self, w: &mut W, _lang: &Kotlin) -> Result<()> {
        let CodeGeneratorConfig {
            module_name,
            encoding,
            features,
            ..
        } = self.config();

        writeln!(w, "package {module_name}")?;
        writeln!(w)?;

        // For bincode encoding, we use imports that are compatible with Kotlin Multiplatform.
        // The serde types (Serializer, Deserializer, etc.) use the package from the
        // external_packages configuration for the "serde" namespace.
        let mut imports = match encoding {
            Encoding::Json => vec![
                "import kotlinx.serialization.Serializable".to_string(),
                "import kotlinx.serialization.SerialName".to_string(),
            ],
            Encoding::Bincode => {
                let bincode_package = self.bincode_package();
                let serde_package = self.serde_package();

                vec![
                    format!("import {bincode_package}.BincodeDeserializer"),
                    format!("import {bincode_package}.BincodeSerializer"),
                    format!("import {serde_package}.DeserializationError"),
                    format!("import {serde_package}.Deserializer"),
                    format!("import {serde_package}.Serializer"),
                ]
            }
            Encoding::None => vec![],
        };

        let mut features_out = vec![];
        for feature in features {
            match feature {
                Feature::Bytes => {
                    if encoding == &Encoding::Bincode {
                        imports.push(format!("import {}.Bytes", self.serde_package()));
                    }
                }
                Feature::BigInt => {
                    // Note: BigInteger is JVM-only. For KMP, you'd need a multiplatform BigInt library.
                    // This is kept for backward compatibility with JVM-only projects.
                    imports.push("import java.math.BigInteger".to_string());
                    match encoding {
                        Encoding::Json => {
                            imports.extend([
                                "import kotlinx.serialization.KSerializer".to_string(),
                                "import kotlinx.serialization.descriptors.PrimitiveKind".to_string(),
                                "import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor".to_string(),
                                "import kotlinx.serialization.encoding.Decoder".to_string(),
                                "import kotlinx.serialization.encoding.Encoder".to_string(),
                                "import kotlinx.serialization.json.JsonDecoder".to_string(),
                                "import kotlinx.serialization.json.JsonEncoder".to_string(),
                                "import kotlinx.serialization.json.JsonUnquotedLiteral".to_string(),
                                "import kotlinx.serialization.json.jsonPrimitive".to_string(),
                            ]);
                            features_out.write_all(FEATURE_BIGINT)?;
                            writeln!(features_out)?;
                        }
                        Encoding::Bincode => {
                            imports.push(format!("import {}.Int128", self.serde_package()));
                        }
                        Encoding::None => (),
                    }
                }
                Feature::ListOfT => {
                    if encoding == &Encoding::Bincode {
                        features_out.write_all(FEATURE_LIST_OF_T)?;
                        writeln!(features_out)?;
                    }
                }
                Feature::OptionOfT => {
                    if encoding == &Encoding::Bincode {
                        features_out.write_all(FEATURE_OPTION_OF_T)?;
                        writeln!(features_out)?;
                    }
                }
                Feature::SetOfT => {
                    if encoding == &Encoding::Bincode {
                        features_out.write_all(FEATURE_SET_OF_T)?;
                        writeln!(features_out)?;
                    }
                }
                Feature::MapOfT => {
                    if encoding == &Encoding::Bincode {
                        features_out.write_all(FEATURE_MAP_OF_T)?;
                        writeln!(features_out)?;
                    }
                }
                Feature::TupleArray => {
                    features_out.write_all(FEATURE_TUPLE_ARRAY)?;
                    writeln!(features_out)?;
                }
            }
        }

        let mut imports = imports
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        imports.sort_unstable();
        imports.dedup();
        if !imports.is_empty() {
            for import in imports {
                writeln!(w, "{import}")?;
            }
            writeln!(w)?;
        }

        w.write_all(&features_out)?;

        Ok(())
    }
}

impl Emitter<Kotlin> for Container<'_> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        let Container {
            name: QualifiedTypeName { namespace: _, name },
            format,
            ..
        } = self;
        match format {
            ContainerFormat::UnitStruct(doc) => {
                data_object(w, name, None, doc, lang, None)?;
            }
            ContainerFormat::NewTypeStruct(format, doc) => {
                data_class(
                    w,
                    name,
                    None,
                    &[Named::new(format, "value".to_string())],
                    doc,
                    lang,
                    None,
                )?;
            }
            ContainerFormat::TupleStruct(formats, doc) => {
                data_class(w, name, None, &named(formats), doc, lang, None)?;
            }
            ContainerFormat::Struct(fields, doc) => {
                if fields.is_empty() {
                    data_object(w, name, None, doc, lang, None)?;
                } else {
                    data_class(w, name, None, fields, doc, lang, None)?;
                }
            }
            ContainerFormat::Enum(variants, doc) => {
                let variant_list: Vec<_> = variants.values().cloned().collect();

                let all_unit_variants = variants
                    .values()
                    .all(|variant| matches!(variant.value, VariantFormat::Unit));

                if all_unit_variants {
                    enum_class(w, name, variants, doc, lang)?;
                } else {
                    sealed_interface(w, name, &variant_list, doc, lang)?;
                }
            }
        }

        Ok(())
    }
}

impl Emitter<Kotlin> for Named<Format> {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        self.doc.write(w, lang)?;

        let name = &self.name.to_lower_camel_case();
        write!(w, "val {name}: ")?;

        self.value.write(w, lang)?;

        // Add = null default only for top-level Option types
        if matches!(self.value, Format::Option(_)) {
            write!(w, " = null")?;
        }

        writeln!(w, ",")
    }
}

impl Emitter<Kotlin> for Doc {
    fn write<W: IndentWrite>(&self, w: &mut W, _lang: &Kotlin) -> Result<()> {
        for comment in self.comments() {
            writeln!(w, "/// {comment}")?;
        }

        Ok(())
    }
}

/// Tells a variant emitter whether it is being written inside a
/// `sealed interface` or an `enum class`, since the Kotlin syntax differs.
#[derive(Clone)]
pub enum VariantContext {
    /// Variant inside a `sealed interface` — carries the interface name and
    /// the variant's zero-based index (used as the bincode discriminant).
    SealedInterface(String, usize),
    /// Variant inside an `enum class` (all-unit variants only).
    EnumClass,
}

impl Emitter<Kotlin> for (&Named<VariantFormat>, &VariantContext) {
    #[allow(clippy::too_many_lines)]
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        let (
            Named {
                name,
                doc,
                value: format,
            },
            context,
        ) = self;

        match (&format, context) {
            (VariantFormat::Variable(_), _) => {
                unreachable!("placeholders should not get this far")
            }
            (VariantFormat::Unit, VariantContext::SealedInterface(interface_name, index)) => {
                data_object(w, name, Some(interface_name), doc, lang, Some(*index))?;
            }
            (VariantFormat::Unit, VariantContext::EnumClass) => {
                doc.write(w, lang)?;
                let name_upper = name.to_uppercase();
                if lang.encoding.is_json() {
                    write!(w, r#"@SerialName("{name}") {name_upper}"#)?;
                } else {
                    write!(w, "{name_upper}")?;
                }
            }
            (
                VariantFormat::NewType(inner),
                VariantContext::SealedInterface(interface_name, index),
            ) => {
                data_class(
                    w,
                    name,
                    Some(interface_name),
                    &[Named::new(inner, "value".to_string())],
                    doc,
                    lang,
                    Some(*index),
                )?;
            }
            (VariantFormat::NewType(_format), VariantContext::EnumClass) => {
                unreachable!("NewType variants are not supported in enum classes")
            }
            (
                VariantFormat::Tuple(formats),
                VariantContext::SealedInterface(interface_name, index),
            ) => {
                data_class(
                    w,
                    name,
                    Some(interface_name),
                    &named(formats),
                    doc,
                    lang,
                    Some(*index),
                )?;
            }
            (VariantFormat::Tuple(_formats), VariantContext::EnumClass) => {
                unreachable!("Tuple variants are not supported in enum classes")
            }
            (
                VariantFormat::Struct(fields),
                VariantContext::SealedInterface(interface_name, index),
            ) => {
                data_class(
                    w,
                    name,
                    Some(interface_name),
                    fields,
                    doc,
                    lang,
                    Some(*index),
                )?;
            }
            (VariantFormat::Struct(_fields), VariantContext::EnumClass) => {
                unreachable!("Struct variants are not supported in enum classes")
            }
        }

        Ok(())
    }
}

impl Format {
    fn is_native(&self) -> bool {
        match self {
            Format::Unit
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
            | Format::Bytes => true,
            Format::Variable(..)
            | Format::TypeName(..)
            | Format::Option(..)
            | Format::Seq(..)
            | Format::Set(..)
            | Format::Map { .. }
            | Format::Tuple(..)
            | Format::TupleArray { .. } => false,
        }
    }

    fn is_leaf(&self) -> bool {
        self.is_native() || matches!(self, Format::TypeName(..))
    }
}

impl Emitter<Kotlin> for Format {
    fn write<W: IndentWrite>(&self, w: &mut W, lang: &Kotlin) -> Result<()> {
        match &self {
            Format::Variable(_variable) => unreachable!("placeholders should not get this far"),
            Format::TypeName(qualified_type_name) => {
                write!(
                    w,
                    "{ty}",
                    ty = qualified_type_name.format(ToString::to_string, ".")
                )
            }
            Format::Unit => write!(w, "Unit"),
            Format::Bool => write!(w, "Boolean"),
            Format::I8 => write!(w, "Byte"),
            Format::I16 => write!(w, "Short"),
            Format::I32 => write!(w, "Int"),
            Format::I64 => write!(w, "Long"),
            Format::U8 => write!(w, "UByte"),
            Format::U16 => write!(w, "UShort"),
            Format::U32 => write!(w, "UInt"),
            Format::U64 => write!(w, "ULong"),
            Format::I128 | Format::U128 => write!(w, "BigInteger"),
            Format::F32 => write!(w, "Float"),
            Format::F64 => write!(w, "Double"),
            Format::Char | Format::Str => write!(w, "String"),
            Format::Bytes => write!(w, "Bytes"),

            Format::Option(format) => {
                format.write(w, lang)?;
                write!(w, "?")
            }
            Format::Seq(format) => {
                write!(w, "List<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Format::Set(format) => {
                write!(w, "Set<")?;
                format.write(w, lang)?;
                write!(w, ">")
            }
            Format::Map { key, value } => {
                write!(w, "Map<")?;
                key.write(w, lang)?;
                write!(w, ", ")?;
                value.write(w, lang)?;
                write!(w, ">")
            }
            Format::Tuple(formats) => {
                let len = formats.len();
                match len {
                    0 => write!(w, "Unit"),
                    1 => {
                        // A single-element tuple is just the element itself
                        formats[0].write(w, lang)
                    }
                    2 => {
                        write!(w, "Pair<")?;
                        formats[0].write(w, lang)?;
                        write!(w, ", ")?;
                        formats[1].write(w, lang)?;
                        write!(w, ">")
                    }
                    3 => {
                        write!(w, "Triple<")?;
                        formats[0].write(w, lang)?;
                        write!(w, ", ")?;
                        formats[1].write(w, lang)?;
                        write!(w, ", ")?;
                        formats[2].write(w, lang)?;
                        write!(w, ">")
                    }
                    _ => {
                        // For larger tuples, we'll use a data class NTupleN
                        write!(w, "NTuple{len}<")?;
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
            Format::TupleArray { content, size: _ } => {
                write!(w, "List<")?;
                content.write(w, lang)?;
                write!(w, ">")
            }
        }
    }
}

/// Emits a Kotlin `data object` — used for unit structs and unit variants.
///
/// When `interface` is `Some`, the object implements it (i.e. it is a variant
/// inside a `sealed interface`). When bincode encoding is active, serialize
/// and deserialize methods are generated.
fn data_object<W: IndentWrite>(
    w: &mut W,
    name: &str,
    interface: Option<&str>,
    doc: &Doc,
    lang: &Kotlin,
    variant_index: Option<usize>,
) -> Result<()> {
    doc.write(w, lang)?;

    if lang.encoding.is_json() {
        writeln!(w, "@Serializable")?;
        writeln!(w, r#"@SerialName("{name}")"#)?;
    }

    write!(w, "data object {name}")?;

    if let Some(interface) = interface {
        write!(w, ": {interface}")?;
    }

    if lang.encoding.is_bincode() {
        write!(w, " ")?;
        let mut w = w.block(Newlines::BOTH)?;
        if variant_index.is_some() {
            write!(w, "override ")?;
        }
        write!(w, "fun serialize(serializer: Serializer) ")?;
        if let Some(index) = variant_index {
            let mut w = w.block(Newlines::BOTH)?;
            push_serializer(&mut w)?;
            writeln!(w, "serializer.serialize_variant_index({index})")?;
            pop_serializer(&mut w)?;
        } else {
            let _ = w.block(Newlines::CLOSE)?;
        }
        writeln!(w)?;

        if variant_index.is_none() {
            write_bincode_serialize(&mut w)?;
            writeln!(w)?;
        }
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, "return {name}")?;
        }
        if variant_index.is_none() {
            writeln!(w)?;
            write_bincode_deserialize(&mut w, name)?;
        }
    } else {
        writeln!(w)?;
    }

    Ok(())
}

/// Emits a Kotlin `data class` — used for structs (with fields), newtype
/// structs, tuple structs, and non-unit sealed-interface variants.
///
/// When `interface` is `Some`, the class implements it. When bincode encoding
/// is active, `serialize` / `deserialize` methods and a `companion object`
/// are generated.
fn data_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    interface: Option<&str>,
    fields: &[Named<Format>],
    doc: &Doc,
    lang: &Kotlin,
    variant_index: Option<usize>,
) -> Result<()> {
    doc.write(w, lang)?;

    if lang.encoding.is_json() {
        writeln!(w, "@Serializable")?;
        writeln!(w, r#"@SerialName("{name}")"#)?;
    }

    writeln!(w, "data class {name}(")?;

    w.indent();
    for field in fields {
        field.write(w, lang)?;
    }
    w.unindent();

    write!(w, ")")?;

    if let Some(interface) = interface {
        write!(w, " : {interface}")?;
    }

    if lang.encoding.is_bincode() {
        write!(w, " ")?;

        let mut w = w.block(Newlines::BOTH)?;
        if variant_index.is_some() {
            write!(w, "override ")?;
        }
        write!(w, "fun serialize(serializer: Serializer) ")?;
        if fields.is_empty() {
            let _ = w.block(Newlines::CLOSE)?;
        } else {
            let mut w = w.block(Newlines::BOTH)?;
            push_serializer(&mut w)?;
            if let Some(index) = variant_index {
                writeln!(w, "serializer.serialize_variant_index({index})")?;
            }
            for field in fields {
                write_serialize(&mut w, &field.name.to_lower_camel_case(), &field.value, 0)?;
            }
            pop_serializer(&mut w)?;
        }
        writeln!(w)?;

        if variant_index.is_none() {
            write_bincode_serialize(&mut w)?;
            writeln!(w)?;
        }
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
            if variant_index.is_none() {
                writeln!(w)?;
                write_bincode_deserialize(&mut w, name)?;
            }
        }
    } else {
        writeln!(w)?;
    }

    Ok(())
}

/// Emits a Kotlin `enum class` — used when all variants are unit variants.
///
/// For JSON encoding, each variant gets `@SerialName`. For bincode, the
/// ordinal is used as the variant index.
fn enum_class<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    doc: &Doc,
    lang: &Kotlin,
) -> Result<()> {
    doc.write(w, lang)?;

    if lang.encoding.is_json() {
        writeln!(w, "@Serializable")?;
        writeln!(w, r#"@SerialName("{name}")"#)?;
    }

    write!(w, "enum class {name} ")?;
    let mut w = w.block(Newlines::BOTH)?;

    for (i, variant) in variants {
        if *i > 0 {
            writeln!(w, ",")?;
        }

        (variant, &VariantContext::EnumClass).write(&mut w, lang)?;
    }
    writeln!(w, ";")?;

    match lang.encoding {
        Encoding::Json => {
            writeln!(w)?;
            writedoc!(
                w,
                "
                val serialName: String
                    get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
                "
            )?;
        }
        Encoding::Bincode => {
            writeln!(w)?;
            write!(w, "fun serialize(serializer: Serializer) ")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                push_serializer(&mut w)?;
                writeln!(w, "serializer.serialize_variant_index(ordinal)")?;
                pop_serializer(&mut w)?;
            }
            writeln!(w)?;

            write_bincode_serialize(&mut w)?;
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
                            write!(w, "{i} -> ")?;
                            (&variant.without_docs(), &VariantContext::EnumClass)
                                .write(&mut w, lang)?;
                            writeln!(w)?;
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
        }
        Encoding::None => (),
    }

    Ok(())
}

/// Emits a Kotlin `sealed interface` — used when at least one variant
/// carries data (newtype, tuple, or struct variant).
///
/// Each variant becomes a nested `data class` or `data object` that
/// implements the interface. For bincode, a `companion object` with
/// `deserialize` dispatches on the variant index.
fn sealed_interface<W: IndentWrite>(
    w: &mut W,
    name: &str,
    variants: &[Named<VariantFormat>],
    doc: &Doc,
    lang: &Kotlin,
) -> Result<()> {
    doc.write(w, lang)?;

    if lang.encoding.is_json() {
        writeln!(w, "@Serializable")?;
        writeln!(w, r#"@SerialName("{name}")"#)?;
    }
    write!(w, "sealed interface {name} ")?;
    let mut w = w.block(Newlines::BOTH)?;

    if lang.encoding.is_bincode() {
        writeln!(w, "fun serialize(serializer: Serializer)")?;
        writeln!(w)?;
        write_bincode_serialize(&mut w)?;
        writeln!(w)?;
    }

    for (index, variant) in variants.iter().enumerate() {
        if index > 0 {
            writeln!(w)?;
        }
        let ctx = VariantContext::SealedInterface(name.to_string(), index);
        (variant, &ctx).write(&mut w, lang)?;
    }

    if lang.encoding.is_bincode() {
        writeln!(w)?;
        write!(w, "companion object ")?;
        let mut w = w.block(Newlines::BOTH)?;
        writeln!(w, "@Throws(DeserializationError::class)")?;
        write!(w, "fun deserialize(deserializer: Deserializer): {name} ")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, "val index = deserializer.deserialize_variant_index()")?;
            write!(w, "return when (index) ")?;
            {
                let mut w = w.block(Newlines::BOTH)?;
                for (i, variant) in variants.iter().enumerate() {
                    let name = &variant.name;
                    writeln!(w, "{i} -> {name}.deserialize(deserializer)")?;
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

fn named<Format: Clone>(formats: &[Format]) -> Vec<Named<Format>> {
    formats
        .iter()
        .enumerate()
        .map(|(i, f)| Named::new(f, format!("field{i}")))
        .collect()
}

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
        // For unsigned types, use Kotlin's native unsigned types directly.
        // No @Unsigned annotations needed - Kotlin has first-class UByte, UShort, UInt, ULong.
        Format::U8 => writeln!(w, "serializer.serialize_u8({field_name})"),
        Format::U16 => writeln!(w, "serializer.serialize_u16({field_name})"),
        Format::U32 => writeln!(w, "serializer.serialize_u32({field_name})"),
        Format::U64 => writeln!(w, "serializer.serialize_u64({field_name})"),
        Format::U128 => writeln!(w, "serializer.serialize_u128({field_name})"),
        Format::F32 => writeln!(w, "serializer.serialize_f32({field_name})"),
        Format::F64 => writeln!(w, "serializer.serialize_f64({field_name})"),
        Format::Char => writeln!(w, "serializer.serialize_char({field_name})"),
        Format::Str => writeln!(w, "serializer.serialize_str({field_name})"),
        // Use direct byte array - no Bytes wrapper needed for KMP compatibility
        Format::Bytes => writeln!(w, "serializer.serialize_bytes({field_name})"),

        // Container types - these generate method calls with lambdas
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
            // Kotlin's Pair/Triple don't have serialize methods, so we need to serialize inline
            let len = formats.len();
            match len {
                0 => writeln!(w, "serializer.serialize_unit({field_name})"),
                1 => write_serialize(w, field_name, &formats[0], level),
                2 => {
                    // Pair<A, B> - serialize first and second
                    write_serialize(w, &format!("{field_name}.first"), &formats[0], level)?;
                    write_serialize(w, &format!("{field_name}.second"), &formats[1], level)
                }
                3 => {
                    // Triple<A, B, C> - serialize first, second, third
                    write_serialize(w, &format!("{field_name}.first"), &formats[0], level)?;
                    write_serialize(w, &format!("{field_name}.second"), &formats[1], level)?;
                    write_serialize(w, &format!("{field_name}.third"), &formats[2], level)
                }
                _ => {
                    // NTupleN - use component accessors
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

    // For key, just call write_serialize directly - it handles all format types
    write_serialize(&mut w, "key", key_format, level + 1)?;

    // For value, just call write_serialize directly - it handles all format types
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
        // KMP serde returns native unsigned types directly - no conversion needed
        Format::U8 => write!(w, "deserializer.deserialize_u8()"),
        Format::U16 => write!(w, "deserializer.deserialize_u16()"),
        Format::U32 => write!(w, "deserializer.deserialize_u32()"),
        Format::U64 => write!(w, "deserializer.deserialize_u64()"),
        Format::U128 => write!(w, "deserializer.deserialize_u128()"),
        Format::F32 => write!(w, "deserializer.deserialize_f32()"),
        Format::F64 => write!(w, "deserializer.deserialize_f64()"),
        Format::Char => write!(w, "deserializer.deserialize_char()"),
        Format::Str => write!(w, "deserializer.deserialize_str()"),
        // Return byte array directly - no Bytes wrapper for KMP compatibility
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
                    // Pair<A, B> - deserialize inline and construct Pair
                    write!(w, "run ")?;
                    let mut w = w.block(Newlines::BOTH)?;
                    write!(w, "val first = ")?;
                    write_deserialize(&mut w, None, &formats[0], true)?;
                    write!(w, "val second = ")?;
                    write_deserialize(&mut w, None, &formats[1], true)?;
                    writeln!(w, "Pair(first, second)")?;
                }
                3 => {
                    // Triple<A, B, C> - deserialize inline and construct Triple
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
                    // NTupleN - deserialize and construct
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

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bincode;
#[cfg(test)]
mod tests_json;
