//! `EmitterPlugin<Kotlin>` implementation for the [`JsonPlugin`].
//!
//! Generated types are `@Serializable` and are encoded with
//! kotlinx.serialization's `Json`, in the shape `serde_json` gives the same
//! Rust types — so JSON written on either side reads on the other.
//!
//! # What this plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `kotlinx.serialization` imports, and the runtime's JSON serializers |
//! | `module_helpers` | serializers for `Bytes`, `UUID` and `BigInteger` |
//! | `module_declarations` | the `Bytes` and `UUID` aliases, so no import hides them |
//! | `type_annotations` | `@Serializable`, naming the type's own serializer where it has one |
//! | `field_annotations` | `@SerialName` with the field's wire name (and `@EncodeDefault` on an optional field) |
//! | `enum_variant_annotations` | `@SerialName` with the variant's wire name |
//! | `has_type_body` / `type_body` | the `serialName` accessor of an `enum class`, and a nested `JsonSerializer` object where the type needs one |
//! | `runtime_files` | the serde runtime, and `JsonCoding.kt` |
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
//! | `char`, `Uuid` | string |
//! | 64- and 128-bit integer | number, written and read exactly |
//! | enum | externally tagged — `"Unit"`, `{"NewType": …}`, `{"Tuple": […]}`, `{"Struct": {…}}` — or internally / adjacently tagged per `#[facet(tag, content)]` |
//!
//! # Generated and hand-written serializers
//!
//! A struct with named fields whose every field kotlinx already encodes the
//! way Rust does is left to the serializer the compiler plugin generates, with
//! `@SerialName` giving each field its wire name. An `enum class` of unit
//! variants is too, when it is externally tagged. Every other type — unit,
//! newtype and tuple structs, a struct holding `()`, a tuple or a 128-bit
//! integer, and every enum with data or a tag — names a nested
//! `JsonSerializer` object, built on the runtime's `JsonElementSerializer` or
//! `JsonNewTypeSerializer`.

use std::collections::BTreeMap;
use std::io;

use crate::generation::{
    CodeGeneratorConfig, Feature, PackageLocation, SERDE_NAMESPACE,
    indent::IndentWrite,
    kotlin::{Kotlin, enum_constant_name, naming, property_name, render_type, variant_class_name},
    plugin::{EmitContext, EmitterPlugin, RuntimeFile},
};
use crate::reflection::format::{ContainerFormat, EnumTagging, Format, Named, VariantFormat};

use super::JsonPlugin;

/// The `Bytes` JSON helper — a custom `KSerializer<com.novi.serde.Bytes>` that
/// writes the serde runtime's `Bytes` value class as a JSON array of numbers
/// from 0 to 255, as `serde_json` writes a `Vec<u8>`, plus the `Bytes` alias
/// that binds it to the emitted type name.
///
/// The emitter writes a bare `Bytes` for [`Format::Bytes`](crate::reflection::format::Format::Bytes),
/// and `com.novi.serde.Bytes` is not itself `@Serializable`, so without this
/// alias kotlinx.serialization has neither a name nor a serializer for it.
const FEATURE_BYTES: &str = r"private object BytesSerializer : KSerializer<com.novi.serde.Bytes> {
    private val delegate = ListSerializer(kotlin.UByte.serializer())
    override val descriptor = delegate.descriptor
    override fun deserialize(decoder: Decoder): com.novi.serde.Bytes =
        com.novi.serde.Bytes(decoder.decodeSerializableValue(delegate).map { it.toByte() }.toByteArray())
    override fun serialize(encoder: Encoder, value: com.novi.serde.Bytes) =
        encoder.encodeSerializableValue(delegate, value.content.map { it.toUByte() })
}

typealias Bytes = @Serializable(with = BytesSerializer::class) com.novi.serde.Bytes
";

/// The `UUID` JSON helper — a custom `KSerializer<java.util.UUID>` that
/// round-trips `UUID` values through JSON string literals.
const FEATURE_UUID: &str = r#"private object UUIDSerializer : KSerializer<java.util.UUID> {
    override val descriptor = PrimitiveSerialDescriptor("UUID", PrimitiveKind.STRING)
    override fun deserialize(decoder: Decoder): java.util.UUID = java.util.UUID.fromString(decoder.decodeString())
    override fun serialize(encoder: Encoder, value: java.util.UUID) = encoder.encodeString(value.toString())
}

typealias UUID = @Serializable(with = UUIDSerializer::class) java.util.UUID
"#;

/// The `BigInteger` JSON helper — a custom `KSerializer<BigInteger>` that
/// round-trips a 128-bit integer through an unquoted JSON number, as
/// `serde_json` writes it.
///
/// The types that hold one name it in their own serializers: a `typealias`
/// cannot bind it, since the module's `import java.math.BigInteger` outranks
/// a same-package declaration.
const FEATURE_BIGINT: &str = r#"@OptIn(ExperimentalSerializationApi::class)
private object BigIntegerSerializer : KSerializer<BigInteger> {
    override val descriptor =
        PrimitiveSerialDescriptor("java.math.BigInteger", PrimitiveKind.STRING)

    override fun deserialize(decoder: Decoder): BigInteger =
        when (decoder) {
            is JsonDecoder -> decoder.decodeJsonElement().jsonPrimitive.content.toBigInteger()
            else -> decoder.decodeString().toBigInteger()
        }

    override fun serialize(encoder: Encoder, value: BigInteger) =
        when (encoder) {
            is JsonEncoder -> encoder.encodeJsonElement(JsonUnquotedLiteral(value.toString()))
            else -> encoder.encodeString(value.toString())
        }
}
"#;

/// The name of the serializer object nested in each type that has one.
const SERIALIZER: &str = "JsonSerializer";

/// The runtime's JSON serializers, which every module imports.
const RUNTIME_SERIALIZERS: &[&str] = &[
    "JsonElementSerializer",
    "JsonNewTypeSerializer",
    "JsonPairSerializer",
    "JsonTripleSerializer",
    "JsonUnitSerializer",
];

impl EmitterPlugin<Kotlin> for JsonPlugin {
    /// Returns the serde Kotlin runtime sources, and the JSON serializers the
    /// generated types use.
    fn runtime_files(&self) -> Vec<RuntimeFile> {
        static SERDE: include_dir::Dir<'static> =
            include_dir::include_dir!("$CARGO_MANIFEST_DIR/runtime/kotlin/com/novi/serde");
        let mut files: Vec<RuntimeFile> = SERDE
            .files()
            .map(|f| RuntimeFile {
                relative_path: format!("com/novi/serde/{}", f.path().display()),
                contents: f.contents().to_vec(),
            })
            .collect();
        files.push(RuntimeFile {
            relative_path: "com/novi/serde/JsonCoding.kt".to_string(),
            contents: include_bytes!("../../../runtime/kotlin-json/com/novi/serde/JsonCoding.kt")
                .to_vec(),
        });
        files
    }

    /// Returns the kotlinx-serialization-json Gradle dependency needed for
    /// JSON encoding.
    fn manifest_dependencies(&self) -> Vec<String> {
        vec![
            r#"    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")"#
                .to_string(),
        ]
    }

    /// JSON / kotlinx.serialization imports for a Kotlin module.
    ///
    /// The annotations, the builtin serializers and the runtime's JSON
    /// serializers, plus the ones only a module whose types hold a particular
    /// format uses.
    fn imports(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let serde = serde_package(config);
        let mut imports = vec![
            "import kotlinx.serialization.Serializable".to_string(),
            "import kotlinx.serialization.SerialName".to_string(),
            "import kotlinx.serialization.builtins.serializer".to_string(),
        ];
        imports.extend(
            RUNTIME_SERIALIZERS
                .iter()
                .map(|name| format!("import {serde}.{name}")),
        );

        let features = &config.features;
        if features.contains(&Feature::OptionOfT) {
            imports.extend([
                "import kotlinx.serialization.EncodeDefault".to_string(),
                "import kotlinx.serialization.ExperimentalSerializationApi".to_string(),
                "import kotlinx.serialization.builtins.nullable".to_string(),
            ]);
        }
        if features.contains(&Feature::ListOfT) || features.contains(&Feature::TupleArray) {
            imports.push("import kotlinx.serialization.builtins.ListSerializer".to_string());
        }
        if features.contains(&Feature::SetOfT) {
            imports.push("import kotlinx.serialization.builtins.SetSerializer".to_string());
        }
        if features.contains(&Feature::MapOfT) {
            imports.push("import kotlinx.serialization.builtins.MapSerializer".to_string());
        }

        // Bytes JSON-specific imports
        if features.contains(&Feature::Bytes) {
            imports.extend([
                "import kotlinx.serialization.KSerializer".to_string(),
                "import kotlinx.serialization.builtins.ListSerializer".to_string(),
                "import kotlinx.serialization.encoding.Decoder".to_string(),
                "import kotlinx.serialization.encoding.Encoder".to_string(),
            ]);
        }

        // UUID JSON-specific imports
        if features.contains(&Feature::Uuid) {
            imports.extend([
                "import kotlinx.serialization.KSerializer".to_string(),
                "import kotlinx.serialization.descriptors.PrimitiveKind".to_string(),
                "import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor".to_string(),
                "import kotlinx.serialization.encoding.Decoder".to_string(),
                "import kotlinx.serialization.encoding.Encoder".to_string(),
            ]);
        }

        // BigInt JSON-specific imports
        if features.contains(&Feature::BigInt) {
            imports.extend([
                "import kotlinx.serialization.ExperimentalSerializationApi".to_string(),
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
        }

        imports
    }

    /// JSON helper snippets for a Kotlin module.
    ///
    /// Emits the custom `KSerializer`s (and, where the underlying type is not
    /// `@Serializable`, the `typealias` that binds them to the emitted type
    /// name) for whichever of `Bytes`, `UUID` and `BigInteger` the module uses.
    fn module_helpers(
        &self,
        w: &mut dyn IndentWrite,
        config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        if config.features.contains(&Feature::Bytes) {
            write!(w, "{FEATURE_BYTES}")?;
            writeln!(w)?;
        }
        if config.features.contains(&Feature::Uuid) {
            write!(w, "{FEATURE_UUID}")?;
            writeln!(w)?;
        }
        if config.features.contains(&Feature::BigInt) {
            write!(w, "{FEATURE_BIGINT}")?;
            writeln!(w)?;
        }
        Ok(())
    }

    /// The `Bytes` and `UUID` aliases from [`module_helpers`](Self::module_helpers).
    fn module_declarations(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        let mut names = vec![];
        if config.features.contains(&Feature::Bytes) {
            names.push("Bytes".to_string());
        }
        if config.features.contains(&Feature::Uuid) {
            names.push("UUID".to_string());
        }
        names
    }

    /// The annotations above each type.
    ///
    /// A type whose JSON the compiler plugin's serializer writes gets
    /// `@Serializable` and `@SerialName("…")` (and an opt-in to
    /// `@EncodeDefault`, when it has an optional field); any other names its
    /// nested `JsonSerializer`. The variants of a `sealed interface` get none:
    /// the interface's serializer writes them.
    fn type_annotations(&self, ctx: &EmitContext) -> Vec<String> {
        if ctx.is_variant() {
            return vec![];
        }
        let name = ctx.name();
        if has_serializer(ctx.container.format) {
            return vec![format!("@Serializable(with = {name}.{SERIALIZER}::class)")];
        }
        let mut annotations = vec![];
        if let ContainerFormat::Struct(fields, _) = ctx.container.format
            && fields.iter().any(|f| is_optional(&f.value))
        {
            annotations.push("@OptIn(ExperimentalSerializationApi::class)".to_string());
        }
        annotations.push("@Serializable".to_string());
        annotations.push(format!("@SerialName({})", literal(name)));
        annotations
    }

    /// `@SerialName("…")` with the field's wire name, on each property of a
    /// struct the compiler plugin serializes, and `@EncodeDefault` on an
    /// optional one, since Rust writes `null` where kotlinx would leave out a
    /// property that holds its default.
    fn field_annotations(&self, field: &Named<Format>, ctx: &EmitContext) -> Vec<String> {
        if ctx.is_variant() {
            return vec![];
        }
        match ctx.container.format {
            ContainerFormat::Struct(..) if !has_serializer(ctx.container.format) => {
                let mut annotations = vec![format!("@SerialName({})", literal(&field.name))];
                if is_optional(&field.value) {
                    annotations.push("@EncodeDefault".to_string());
                }
                annotations
            }
            _ => vec![],
        }
    }

    /// `@SerialName("…")` inline annotation for each all-unit enum class
    /// variant.
    ///
    /// Emitted on the same line as the uppercased variant name, e.g.:
    ///
    /// ```text
    /// @SerialName("Variant1") VARIANT1,
    /// ```
    ///
    /// This preserves the original Rust variant name as the serialized tag,
    /// while Kotlin's convention uppercases the entry identifier.
    fn enum_variant_annotations(&self, name: &str) -> Vec<String> {
        vec![format!("@SerialName({})", literal(name))]
    }

    /// A `data class` or `data object` needs a body for its `JsonSerializer`.
    fn has_type_body(&self, ctx: &EmitContext) -> bool {
        !ctx.is_variant() && has_serializer(ctx.container.format)
    }

    /// The `serialName` accessor of an all-unit enum class, and the nested
    /// `JsonSerializer` of a type that has one.
    fn type_body(&self, w: &mut dyn IndentWrite, ctx: &EmitContext) -> io::Result<()> {
        if ctx.is_variant() {
            return Ok(());
        }

        let format = ctx.container.format;
        if let ContainerFormat::Enum(variants, _, _) = format
            && is_unit_enum(variants)
        {
            writeln!(w)?;
            writeln!(w, "val serialName: String")?;
            writeln!(
                w,
                "    get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value"
            )?;
        }

        if !has_serializer(format) {
            return Ok(());
        }
        let name = ctx.name();
        let names = Names::new(ctx.config);
        match format {
            ContainerFormat::UnitStruct(_) => write_unit_struct(w, name),
            ContainerFormat::NewTypeStruct(format, _) => {
                write_newtype_struct(w, name, format, &names)
            }
            ContainerFormat::TupleStruct(formats, _) => {
                write_tuple_struct(w, name, formats, &names)
            }
            ContainerFormat::Struct(fields, _) => write_struct(w, name, fields, &names),
            ContainerFormat::Enum(variants, tagging, _) => {
                writeln!(w)?;
                write_enum(w, name, variants, tagging, &names)
            }
        }
    }
}

/// The package of the serde runtime: an external package's path, or the one
/// the installer writes.
fn serde_package(config: &CodeGeneratorConfig) -> String {
    config
        .external_packages
        .get(SERDE_NAMESPACE)
        .and_then(|pkg| match &pkg.location {
            PackageLocation::Path(path) => Some(path.clone()),
            PackageLocation::Url(_) => None,
        })
        .unwrap_or_else(|| "com.novi.serde".to_string())
}

// ---------------------------------------------------------------------------
// Classification
// ---------------------------------------------------------------------------

/// Whether a value of this format encodes, through the serializer
/// kotlinx.serialization finds for its Kotlin type, exactly the way
/// `serde_json` encodes the Rust value.
///
/// kotlinx writes `Unit` as `{}` and a `Pair` or `Triple` as an object, and
/// has no serializer for `BigInteger`. It writes a map's string, number,
/// boolean and enum keys as strings, as Rust does.
fn is_native(format: &Format) -> bool {
    match format {
        Format::Variable(_) => unreachable!("placeholders should not get this far"),
        Format::Unit | Format::I128 | Format::U128 => false,
        Format::Tuple(formats) => formats.len() == 1 && is_native(&formats[0]),
        Format::Option(inner)
        | Format::Seq(inner)
        | Format::Set(inner)
        | Format::TupleArray { content: inner, .. } => is_native(inner),
        Format::Map { key, value } => is_native(key) && is_native(value),
        _ => true,
    }
}

/// Whether a field of this format is optional: the emitter gives its property
/// the default `null`, and Rust reads a missing key as `None`.
const fn is_optional(format: &Format) -> bool {
    matches!(format, Format::Option(_))
}

/// Whether the Kotlin type of this format is already nullable.
fn is_nullable(format: &Format) -> bool {
    match format {
        Format::Option(_) => true,
        Format::Tuple(formats) if formats.len() == 1 => is_nullable(&formats[0]),
        _ => false,
    }
}

fn is_unit_enum(variants: &BTreeMap<u32, Named<VariantFormat>>) -> bool {
    variants
        .values()
        .all(|v| matches!(v.value, VariantFormat::Unit))
}

/// Whether a type of this format has a nested `JsonSerializer`, because the
/// compiler plugin's serializer would not write the JSON Rust does.
fn has_serializer(format: &ContainerFormat) -> bool {
    match format {
        ContainerFormat::UnitStruct(_)
        | ContainerFormat::NewTypeStruct(..)
        | ContainerFormat::TupleStruct(..) => true,
        ContainerFormat::Struct(fields, _) => !fields.iter().all(|f| is_native(&f.value)),
        ContainerFormat::Enum(variants, tagging, _) => {
            !is_unit_enum(variants)
                || (!variants.is_empty() && !matches!(tagging, EnumTagging::External))
        }
    }
}

// ---------------------------------------------------------------------------
// Serializer expressions
// ---------------------------------------------------------------------------

/// The spellings of the builtin types the generated serializers name,
/// qualified where the module shadows them.
struct Names<'a> {
    config: &'a CodeGeneratorConfig,
}

impl<'a> Names<'a> {
    const fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self { config }
    }

    fn builtin(&self, name: &'static str) -> String {
        naming::builtin(name, self.config).into_owned()
    }

    fn render(&self, format: &Format) -> String {
        render_type(format, self.config)
    }

    /// A Kotlin expression for the serializer that writes a value of `format`
    /// the way `serde_json` writes the Rust value.
    fn serializer(&self, format: &Format) -> String {
        let builtin = |name| format!("{}.serializer()", self.builtin(name));
        match format {
            Format::Variable(_) => unreachable!("placeholders should not get this far"),
            Format::TypeName(_) => format!("{}.serializer()", self.render(format)),
            Format::Unit => "JsonUnitSerializer".to_string(),
            Format::Bool => builtin("Boolean"),
            Format::I8 => builtin("Byte"),
            Format::I16 => builtin("Short"),
            Format::I32 => builtin("Int"),
            Format::I64 => builtin("Long"),
            Format::U8 => builtin("UByte"),
            Format::U16 => builtin("UShort"),
            Format::U32 => builtin("UInt"),
            Format::U64 => builtin("ULong"),
            Format::I128 | Format::U128 => "BigIntegerSerializer".to_string(),
            Format::F32 => builtin("Float"),
            Format::F64 => builtin("Double"),
            Format::Char | Format::Str => builtin("String"),
            Format::Bytes => "BytesSerializer".to_string(),
            Format::Uuid => "UUIDSerializer".to_string(),
            // Kotlin has no `T??`: an optional option is the same `T?`.
            Format::Option(inner) if is_nullable(inner) => self.serializer(inner),
            Format::Option(inner) => format!("{}.nullable", self.serializer(inner)),
            Format::Seq(inner) | Format::TupleArray { content: inner, .. } => {
                format!("ListSerializer({})", self.serializer(inner))
            }
            Format::Set(inner) => format!("SetSerializer({})", self.serializer(inner)),
            Format::Map { key, value } => format!(
                "MapSerializer({}, {})",
                self.serializer(key),
                self.serializer(value)
            ),
            Format::Tuple(formats) => match formats.as_slice() {
                [] => "JsonUnitSerializer".to_string(),
                [format] => self.serializer(format),
                _ => {
                    let serializers = formats
                        .iter()
                        .map(|f| self.serializer(f))
                        .collect::<Vec<_>>()
                        .join(", ");
                    match formats.len() {
                        2 => format!("JsonPairSerializer({serializers})"),
                        3 => format!("JsonTripleSerializer({serializers})"),
                        len => format!("NTuple{len}.serializer({serializers})"),
                    }
                }
            },
        }
    }
}

/// A Kotlin string literal holding `value`.
fn literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '$' => out.push_str("\\$"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ---------------------------------------------------------------------------
// Serializers
// ---------------------------------------------------------------------------

/// Writes `object JsonSerializer : JsonElementSerializer<name>(…)`, whose
/// `toJson` and `fromJson` lambdas `to_json` and `from_json` write the bodies
/// of; `tagging` is an enum's.
fn write_element_serializer(
    w: &mut dyn IndentWrite,
    name: &str,
    tagging: Option<&EnumTagging>,
    to_json: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>,
    from_json: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>,
) -> io::Result<()> {
    writeln!(w, "object {SERIALIZER} : JsonElementSerializer<{name}>(")?;
    w.indent();
    writeln!(w, "{},", literal(name))?;
    match tagging {
        None | Some(EnumTagging::External) => {}
        Some(EnumTagging::Internal { tag }) => writeln!(w, "tag = {},", literal(tag))?,
        Some(EnumTagging::Adjacent { tag, content }) => {
            writeln!(w, "tag = {},", literal(tag))?;
            writeln!(w, "content = {},", literal(content))?;
        }
    }
    write_lambda(w, "toJson", "value", to_json)?;
    write_lambda(w, "fromJson", "element", from_json)?;
    w.unindent();
    writeln!(w, ")")
}

/// Writes `label = { param ->`, the body `body` writes, and `},`.
fn write_lambda(
    w: &mut dyn IndentWrite,
    label: &str,
    param: &str,
    body: impl FnOnce(&mut dyn IndentWrite) -> io::Result<()>,
) -> io::Result<()> {
    writeln!(w, "{label} = {{ {param} ->")?;
    w.indent();
    body(w)?;
    w.unindent();
    writeln!(w, "}},")
}

/// Writes `call(`, one argument per line (the last followed by `suffix`), and
/// `)`. Writes `call()` when there are no arguments.
fn write_call(
    w: &mut dyn IndentWrite,
    call: &str,
    args: &[String],
    suffix: &str,
) -> io::Result<()> {
    if args.is_empty() {
        return writeln!(w, "{call}(){suffix}");
    }
    writeln!(w, "{call}(")?;
    w.indent();
    for arg in args {
        writeln!(w, "{arg},")?;
    }
    w.unindent();
    writeln!(w, "){suffix}")
}

/// `encode(serializer, expr)`: the JSON of `expr`, a value of `format`.
fn encode(format: &Format, expr: &str, names: &Names) -> String {
    format!("encode({}, {expr})", names.serializer(format))
}

/// `decode(serializer, element)`: the value of `format` the JSON `element`
/// holds.
fn decode(format: &Format, element: &str, names: &Names) -> String {
    format!("decode({}, {element})", names.serializer(format))
}

/// The `"wire" to encode(…)` entries of an object holding `fields`, read from
/// `value`.
fn encode_fields(fields: &[Named<Format>], value: &str, names: &Names) -> Vec<String> {
    fields
        .iter()
        .map(|field| {
            let property = format!("{value}.{}", property_name(&field.name));
            format!(
                "{} to {}",
                literal(&field.name),
                encode(&field.value, &property, names)
            )
        })
        .collect()
}

/// The `property = decode(…)` arguments constructing a value with `fields`,
/// read from the `JsonFields` named `fields`.
fn decode_fields(fields: &[Named<Format>], names: &Names) -> Vec<String> {
    fields
        .iter()
        .map(|field| {
            let lookup = if is_optional(&field.value) {
                "optional"
            } else {
                "required"
            };
            let element = format!("fields.{lookup}({})", literal(&field.name));
            format!(
                "{} = {}",
                property_name(&field.name),
                decode(&field.value, &element, names)
            )
        })
        .collect()
}

/// The `encode(…)` elements of an array holding `formats`, read from the
/// properties `field0`, … of `value`.
fn encode_elements(formats: &[Format], value: &str, names: &Names) -> Vec<String> {
    formats
        .iter()
        .enumerate()
        .map(|(i, format)| encode(format, &format!("{value}.field{i}"), names))
        .collect()
}

/// The `decode(…)` arguments constructing a value from the elements of the
/// list named `items`.
fn decode_elements(formats: &[Format], names: &Names) -> Vec<String> {
    formats
        .iter()
        .enumerate()
        .map(|(i, format)| decode(format, &format!("items[{i}]"), names))
        .collect()
}

/// Rust writes a unit struct as `null`.
///
/// The registry also records a braced struct with no fields (or none left
/// once skipped ones are dropped) as a unit struct, which Rust writes as `{}`,
/// so that reads too.
fn write_unit_struct(w: &mut dyn IndentWrite, name: &str) -> io::Result<()> {
    write_element_serializer(
        w,
        name,
        None,
        |w| writeln!(w, "unit()"),
        |w| {
            writeln!(w, "unit(element)")?;
            writeln!(w, "{name}")
        },
    )
}

/// Rust writes a newtype struct as the value it wraps.
fn write_newtype_struct(
    w: &mut dyn IndentWrite,
    name: &str,
    format: &Format,
    names: &Names,
) -> io::Result<()> {
    writeln!(
        w,
        "object {SERIALIZER} : JsonNewTypeSerializer<{name}, {}>(",
        names.render(format)
    )?;
    w.indent();
    writeln!(w, "serializer = {{ {} }},", names.serializer(format))?;
    writeln!(w, "wrap = {{ {name}(it) }},")?;
    writeln!(w, "unwrap = {{ it.value }},")?;
    w.unindent();
    writeln!(w, ")")
}

/// Rust writes a tuple struct as an array.
fn write_tuple_struct(
    w: &mut dyn IndentWrite,
    name: &str,
    formats: &[Format],
    names: &Names,
) -> io::Result<()> {
    write_element_serializer(
        w,
        name,
        None,
        |w| write_call(w, "array", &encode_elements(formats, "value", names), ""),
        |w| {
            writeln!(w, "val items = tuple(element, {})", formats.len())?;
            write_call(w, name, &decode_elements(formats, names), "")
        },
    )
}

/// A struct with named fields is a JSON object keyed by the fields' wire
/// names.
fn write_struct(
    w: &mut dyn IndentWrite,
    name: &str,
    fields: &[Named<Format>],
    names: &Names,
) -> io::Result<()> {
    write_element_serializer(
        w,
        name,
        None,
        |w| write_call(w, "obj", &encode_fields(fields, "value", names), ""),
        |w| {
            writeln!(w, "val fields = fields(element)")?;
            write_call(w, name, &decode_fields(fields, names), "")
        },
    )
}

/// An enum is written as its variant, tagged as the registry records:
/// externally — `"Unit"`, `{"NewType": …}`, `{"Tuple": […]}`,
/// `{"Struct": {…}}` — or internally or adjacently, per
/// `#[facet(tag, content)]`.
fn write_enum(
    w: &mut dyn IndentWrite,
    name: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
    tagging: &EnumTagging,
    names: &Names,
) -> io::Result<()> {
    let unit_enum = is_unit_enum(variants);
    // The Kotlin spelling of a variant: an `enum class` constant, or a class
    // (or object) nested in the `sealed interface`.
    let kotlin_name = |variant: &str| {
        if unit_enum {
            enum_constant_name(variant)
        } else {
            variant_class_name(variant)
        }
    };

    write_element_serializer(
        w,
        name,
        Some(tagging),
        |w| {
            writeln!(w, "when (value) {{")?;
            w.indent();
            for variant in variants.values() {
                let wire = literal(&variant.name);
                let kotlin = kotlin_name(&variant.name);
                let case = if unit_enum {
                    kotlin
                } else {
                    format!("is {kotlin}")
                };
                match &variant.value {
                    VariantFormat::Variable(_) => {
                        unreachable!("placeholders should not get this far")
                    }
                    VariantFormat::Unit => writeln!(w, "{case} -> variant({wire})")?,
                    VariantFormat::NewType(format) => writeln!(
                        w,
                        "{case} -> variant({wire}, {})",
                        encode(format, "value.value", names)
                    )?,
                    VariantFormat::Tuple(formats) => {
                        writeln!(w, "{case} -> variant(")?;
                        w.indent();
                        writeln!(w, "{wire},")?;
                        write_call(w, "array", &encode_elements(formats, "value", names), ",")?;
                        w.unindent();
                        writeln!(w, ")")?;
                    }
                    VariantFormat::Struct(fields) => {
                        writeln!(w, "{case} -> variant(")?;
                        w.indent();
                        writeln!(w, "{wire},")?;
                        write_call(w, "obj", &encode_fields(fields, "value", names), ",")?;
                        w.unindent();
                        writeln!(w, ")")?;
                    }
                }
            }
            w.unindent();
            writeln!(w, "}}")
        },
        |w| {
            let content = if unit_enum { "_" } else { "content" };
            writeln!(w, "val (tag, {content}) = variant(element)")?;
            writeln!(w, "when (tag) {{")?;
            w.indent();
            for variant in variants.values() {
                let wire = literal(&variant.name);
                let kotlin = kotlin_name(&variant.name);
                match &variant.value {
                    VariantFormat::Variable(_) => {
                        unreachable!("placeholders should not get this far")
                    }
                    VariantFormat::Unit => writeln!(w, "{wire} -> {kotlin}")?,
                    VariantFormat::NewType(format) => writeln!(
                        w,
                        "{wire} -> {kotlin}({})",
                        decode(format, "payload(content)", names)
                    )?,
                    VariantFormat::Tuple(formats) => {
                        writeln!(w, "{wire} -> {{")?;
                        w.indent();
                        writeln!(w, "val items = tuple(content, {})", formats.len())?;
                        write_call(w, &kotlin, &decode_elements(formats, names), "")?;
                        w.unindent();
                        writeln!(w, "}}")?;
                    }
                    VariantFormat::Struct(fields) => {
                        writeln!(w, "{wire} -> {{")?;
                        w.indent();
                        writeln!(w, "val fields = fields(content)")?;
                        write_call(w, &kotlin, &decode_fields(fields, names), "")?;
                        w.unindent();
                        writeln!(w, "}}")?;
                    }
                }
            }
            writeln!(w, "else -> unknownVariant(tag)")?;
            w.unindent();
            writeln!(w, "}}")
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::CodeGeneratorConfig;
    use std::collections::BTreeSet;

    fn make_config(features: &[Feature]) -> CodeGeneratorConfig {
        let mut cfg = CodeGeneratorConfig::new("com.example".to_string());
        cfg.features = features.iter().copied().collect::<BTreeSet<_>>();
        cfg
    }

    #[test]
    fn base_imports_are_present() {
        let cfg = make_config(&[]);
        let plugin = &JsonPlugin as &dyn EmitterPlugin<Kotlin>;
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("Serializable")));
        assert!(imports.iter().any(|i| i.contains("SerialName")));
    }

    #[test]
    fn bytes_adds_json_imports_and_alias() {
        let cfg = make_config(&[Feature::Bytes]);
        let plugin = &JsonPlugin as &dyn EmitterPlugin<Kotlin>;
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("KSerializer")));
        assert!(imports.iter().any(|i| i.contains("ListSerializer")));
        assert!(imports.iter().any(|i| i.contains("encoding.Decoder")));
        assert!(imports.iter().any(|i| i.contains("encoding.Encoder")));

        let mut buf = Vec::new();
        {
            use crate::generation::indent::IndentedWriter;
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin.module_helpers(&mut w, &cfg).unwrap();
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains(
            "typealias Bytes = @Serializable(with = BytesSerializer::class) com.novi.serde.Bytes"
        ));
    }

    #[test]
    fn bigint_adds_json_imports() {
        let cfg = make_config(&[Feature::BigInt]);
        let plugin = &JsonPlugin as &dyn EmitterPlugin<Kotlin>;
        let imports = plugin.imports(&cfg);

        assert!(imports.iter().any(|i| i.contains("KSerializer")));
        assert!(imports.iter().any(|i| i.contains("PrimitiveKind")));
        assert!(imports.iter().any(|i| i.contains("JsonDecoder")));
        assert!(imports.iter().any(|i| i.contains("JsonEncoder")));
        assert!(imports.iter().any(|i| i.contains("JsonUnquotedLiteral")));
    }

    #[test]
    fn bigint_module_helpers_emit_feature() {
        let cfg = make_config(&[Feature::BigInt]);
        let plugin = &JsonPlugin as &dyn EmitterPlugin<Kotlin>;

        let mut buf = Vec::new();
        {
            use crate::generation::indent::IndentedWriter;
            let mut w = IndentedWriter::new(&mut buf, cfg.indent);
            plugin.module_helpers(&mut w, &cfg).unwrap();
        }

        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn type_annotations_include_serializable_and_serial_name() {
        use crate::generation::Container;
        use crate::reflection::format::{ContainerFormat, Doc, QualifiedTypeName};

        let config = make_config(&[]);
        let name = QualifiedTypeName::root("Foo".to_string());
        let format = ContainerFormat::Struct(vec![], Doc::default());
        let container = Container {
            name: &name,
            format: &format,
        };
        let ctx = EmitContext::top_level(&container, &config);
        let plugin = &JsonPlugin as &dyn EmitterPlugin<Kotlin>;
        let annotations = plugin.type_annotations(&ctx);

        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0], "@Serializable");
        assert_eq!(annotations[1], r#"@SerialName("Foo")"#);
    }

    #[test]
    fn literal_escapes_kotlin_templates() {
        assert_eq!(literal(r#"$ref "a\b""#), r#""\$ref \"a\\b\"""#);
    }

    #[test]
    fn serializers_compose_serde_shapes() {
        let cfg = make_config(&[]);
        let names = Names::new(&cfg);
        let pair = Format::Tuple(vec![Format::U8, Format::Unit]);
        assert_eq!(
            names.serializer(&Format::Seq(Box::new(pair))),
            "ListSerializer(JsonPairSerializer(UByte.serializer(), JsonUnitSerializer))"
        );
        let map = Format::Map {
            key: Box::new(Format::U128),
            value: Box::new(Format::Option(Box::new(Format::Option(Box::new(
                Format::Str,
            ))))),
        };
        assert_eq!(
            names.serializer(&map),
            "MapSerializer(BigIntegerSerializer, String.serializer().nullable)"
        );
    }

    #[test]
    fn only_structs_kotlinx_writes_like_serde_keep_the_plugin_serializer() {
        use crate::reflection::format::Doc;

        let struct_of = |format: Format| {
            ContainerFormat::Struct(vec![Named::new(&format, "f".to_string())], Doc::default())
        };
        let map = Format::Map {
            key: Box::new(Format::U32),
            value: Box::new(Format::Option(Box::new(Format::Str))),
        };
        assert!(!has_serializer(&struct_of(map)));
        assert!(has_serializer(&struct_of(Format::Unit)));
        assert!(has_serializer(&struct_of(Format::I128)));
        assert!(has_serializer(&struct_of(Format::Tuple(vec![
            Format::U8,
            Format::U8
        ]))));
        assert!(!has_serializer(&ContainerFormat::Struct(
            vec![],
            Doc::default()
        )));
        assert!(has_serializer(&ContainerFormat::UnitStruct(Doc::default())));
    }
}
