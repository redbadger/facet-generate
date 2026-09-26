//! Kotlin reserved words.
//!
//! Source: kotlinlang.org — Keywords and operators → *Hard keywords*. Only
//! hard keywords are listed: soft keywords and modifier keywords (`value`,
//! `field`, `import`, `data`, `by`, `get`, `set`, …) are legal identifiers,
//! and two of them — `value` and `field` — are the synthetic names this
//! generator gives newtype and tuple members, so escaping them would change
//! existing output. `default` is not a Kotlin keyword either.

use std::borrow::Cow;

use heck::ToLowerCamelCase;

use crate::{
    generation::{
        config::CodeGeneratorConfig,
        naming::{EscapeStyle, ForbiddenNames, FormatBoundNames, NamingRules, qualify},
    },
    reflection::format::Format,
};

/// Kotlin hard keywords, sorted.
pub(crate) const KEYWORDS: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
];

/// Prelude types the emitter writes bare, and the fully qualified form to use
/// instead when the module declares a type of the same name. Sorted by the bare
/// name.
///
/// A declaration in the same package outranks the implicit `kotlin.*` and
/// `kotlin.collections.*` imports, so `Set<String>` in a module that also
/// declares `data class Set` would resolve to the data class.
pub(crate) const QUALIFIED: &[(&str, &str)] = &[
    ("ArrayList", "kotlin.collections.ArrayList"),
    ("Boolean", "kotlin.Boolean"),
    ("Byte", "kotlin.Byte"),
    ("ByteArray", "kotlin.ByteArray"),
    ("Double", "kotlin.Double"),
    ("Float", "kotlin.Float"),
    ("Int", "kotlin.Int"),
    ("List", "kotlin.collections.List"),
    ("Long", "kotlin.Long"),
    ("Map", "kotlin.collections.Map"),
    ("MutableList", "kotlin.collections.MutableList"),
    ("Pair", "kotlin.Pair"),
    ("Set", "kotlin.collections.Set"),
    ("Short", "kotlin.Short"),
    ("String", "kotlin.String"),
    ("Throws", "kotlin.jvm.Throws"),
    ("Triple", "kotlin.Triple"),
    ("UByte", "kotlin.UByte"),
    ("UInt", "kotlin.UInt"),
    ("ULong", "kotlin.ULong"),
    ("UShort", "kotlin.UShort"),
    ("Unit", "kotlin.Unit"),
];

/// Type names the generated module already uses for something else, with the
/// clause that names what each collides with. Sorted by name.
///
/// These are explicit imports and runtime types, which outrank a same-package
/// declaration, so the collision cannot be resolved by qualifying the builtin.
/// The ones that are only written beside a field of a particular format are
/// listed again in [`FORMAT_BOUND_TYPES`], which limits when they apply.
pub(crate) const FORBIDDEN_TYPES: ForbiddenNames = &[
    ("Any", "the Kotlin root type `kotlin.Any`"),
    ("BigInteger", "the `java.math.BigInteger` import"),
    (
        "BigIntegerSerializer",
        "the generated `BigIntegerSerializer` object",
    ),
    (
        "BinaryDeserializer",
        "the runtime type `com.novi.serde.BinaryDeserializer`",
    ),
    (
        "BinarySerializer",
        "the runtime type `com.novi.serde.BinarySerializer`",
    ),
    ("BincodeDeserializer", "the `BincodeDeserializer` import"),
    ("BincodeSerializer", "the `BincodeSerializer` import"),
    ("Bytes", "the `Bytes` import"),
    ("BytesSerializer", "the generated `BytesSerializer` object"),
    (
        "Decoder",
        "the `kotlinx.serialization.encoding.Decoder` import",
    ),
    ("DeserializationError", "the `DeserializationError` import"),
    ("Deserializer", "the `Deserializer` import"),
    (
        "EncodeDefault",
        "the `kotlinx.serialization.EncodeDefault` import",
    ),
    (
        "Encoder",
        "the `kotlinx.serialization.encoding.Encoder` import",
    ),
    (
        "ExperimentalSerializationApi",
        "the `kotlinx.serialization.ExperimentalSerializationApi` import",
    ),
    ("Int128", "the `Int128` import"),
    (
        "JsonDecoder",
        "the `kotlinx.serialization.json.JsonDecoder` import",
    ),
    (
        "JsonElementSerializer",
        "the runtime type `com.novi.serde.JsonElementSerializer`",
    ),
    (
        "JsonEncoder",
        "the `kotlinx.serialization.json.JsonEncoder` import",
    ),
    (
        "JsonNewTypeSerializer",
        "the runtime type `com.novi.serde.JsonNewTypeSerializer`",
    ),
    (
        "JsonPairSerializer",
        "the runtime type `com.novi.serde.JsonPairSerializer`",
    ),
    ("JsonSerializer", "the generated `JsonSerializer` object"),
    (
        "JsonTripleSerializer",
        "the runtime type `com.novi.serde.JsonTripleSerializer`",
    ),
    (
        "JsonUnitSerializer",
        "the runtime type `com.novi.serde.JsonUnitSerializer`",
    ),
    (
        "JsonUnquotedLiteral",
        "the `kotlinx.serialization.json.JsonUnquotedLiteral` import",
    ),
    (
        "KSerializer",
        "the `kotlinx.serialization.KSerializer` import",
    ),
    ("NTuple4", "the generated tuple type `NTuple4`"),
    ("NTuple5", "the generated tuple type `NTuple5`"),
    ("NTuple6", "the generated tuple type `NTuple6`"),
    ("Nothing", "the Kotlin bottom type `kotlin.Nothing`"),
    (
        "PrimitiveKind",
        "the `kotlinx.serialization.descriptors.PrimitiveKind` import",
    ),
    (
        "PrimitiveSerialDescriptor",
        "the `kotlinx.serialization.descriptors.PrimitiveSerialDescriptor` import",
    ),
    (
        "SerdeByteArrayOutput",
        "the runtime type `com.novi.serde.SerdeByteArrayOutput`",
    ),
    (
        "SerialName",
        "the `kotlinx.serialization.SerialName` import",
    ),
    (
        "Serializable",
        "the `kotlinx.serialization.Serializable` import",
    ),
    ("SerializationError", "the `SerializationError` import"),
    ("Serializer", "the `Serializer` import"),
    ("Slice", "the runtime type `com.novi.serde.Slice`"),
    ("Tuple4", "the runtime type `com.novi.serde.Tuple4`"),
    ("Tuple5", "the runtime type `com.novi.serde.Tuple5`"),
    ("Tuple6", "the runtime type `com.novi.serde.Tuple6`"),
    ("UInt128", "the runtime type `com.novi.serde.UInt128`"),
    ("UUID", "the `java.util.UUID` import"),
    ("UUIDSerializer", "the generated `UUIDSerializer` object"),
];

/// The entries of [`FORBIDDEN_TYPES`] that the generated code mentions only
/// beside a field of a particular format: `Bytes` where a `#[facet(bytes)]`
/// field is serialized, `UUID` beside a `Uuid`, `BigInteger` beside a 128-bit
/// integer, `NTupleN` beside an N-tuple. Elsewhere the import, alias or helper
/// is not even written, so a declaration of that name only collides within the
/// scope that has such a field — the whole module for a top-level type, and the
/// enclosing enum for a variant, since a nested class outranks the file's
/// imports inside the class that declares it. Sorted by name.
pub(crate) const FORMAT_BOUND_TYPES: FormatBoundNames = &[
    ("BigInteger", is_128_bit),
    ("BigIntegerSerializer", is_128_bit),
    ("Bytes", is_bytes_or_uuid),
    ("BytesSerializer", is_bytes_or_uuid),
    ("Int128", is_128_bit),
    ("NTuple4", is_tuple_of_4),
    ("NTuple5", is_tuple_of_5),
    ("NTuple6", is_tuple_of_6),
    ("UInt128", is_128_bit),
    ("UUID", is_uuid),
    ("UUIDSerializer", is_uuid),
];

const fn is_128_bit(format: &Format) -> bool {
    matches!(format, Format::I128 | Format::U128)
}

/// The bincode plugin imports `Bytes` for a `Uuid` field as well, since a UUID
/// is serialized through it.
const fn is_bytes_or_uuid(format: &Format) -> bool {
    matches!(format, Format::Bytes | Format::Uuid)
}

const fn is_uuid(format: &Format) -> bool {
    matches!(format, Format::Uuid)
}

fn is_tuple_of_4(format: &Format) -> bool {
    matches!(format, Format::Tuple(formats) if formats.len() == 4)
}

fn is_tuple_of_5(format: &Format) -> bool {
    matches!(format, Format::Tuple(formats) if formats.len() == 5)
}

fn is_tuple_of_6(format: &Format) -> bool {
    matches!(format, Format::Tuple(formats) if formats.len() == 6)
}

/// Property names the generated code cannot accommodate, with the clause
/// explaining why. Sorted by name.
pub(crate) const FORBIDDEN_MEMBERS: ForbiddenNames = &[
    ("componentN", "Kotlin generates for every data class"),
    ("copy", "Kotlin generates for every data class"),
    (
        "deserializer",
        "shadows the `deserializer` parameter in the generated deserialize method",
    ),
    ("equals", "Kotlin generates for every data class"),
    ("hashCode", "Kotlin generates for every data class"),
    (
        "index",
        "shadows the `index` local in the generated deserialize method",
    ),
    (
        "serializer",
        "shadows the `serializer` parameter in the generated serialize method",
    ),
    ("toString", "Kotlin generates for every data class"),
];

/// Kotlin writes container and variant names verbatim.
fn type_case(name: &str) -> String {
    name.to_string()
}

fn member_case(name: &str) -> String {
    name.to_lower_camel_case()
}

/// The naming rules for this language.
pub(crate) const RULES: NamingRules = NamingRules {
    language: "Kotlin",
    reserved_words: KEYWORDS,
    escape_style: EscapeStyle::Backticks,
    forbidden_types: FORBIDDEN_TYPES,
    forbidden_members: FORBIDDEN_MEMBERS,
    format_bound_types: FORMAT_BOUND_TYPES,
    type_case,
    member_case,
    variants_are_types: true,
    member_equals_type_forbidden: false,
    numbered_components_forbidden: true,
};

/// Returns `true` if the module declares a type spelled `name`.
pub(crate) fn shadows(name: &str, config: &CodeGeneratorConfig) -> bool {
    config.declared_type_names.contains(name)
}

/// The Kotlin spelling of the builtin type `name`: fully qualified when a
/// declaration in the generated module shadows it, and bare otherwise.
pub(crate) fn builtin<'a>(name: &'a str, config: &CodeGeneratorConfig) -> Cow<'a, str> {
    qualify(name, QUALIFIED, |n| shadows(n, config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_tables_are_sorted_for_binary_search() {
        assert!(
            QUALIFIED.windows(2).all(|w| w[0].0 < w[1].0),
            "QUALIFIED must be sorted by the bare name"
        );
        assert!(
            FORBIDDEN_TYPES.windows(2).all(|w| w[0].0 < w[1].0),
            "FORBIDDEN_TYPES must be sorted by name"
        );
        assert!(
            FORBIDDEN_MEMBERS.windows(2).all(|w| w[0].0 < w[1].0),
            "FORBIDDEN_MEMBERS must be sorted by name"
        );
        assert!(
            FORMAT_BOUND_TYPES.windows(2).all(|w| w[0].0 < w[1].0),
            "FORMAT_BOUND_TYPES must be sorted by name"
        );
    }

    #[test]
    fn every_format_bound_type_is_also_forbidden() {
        for (name, _) in FORMAT_BOUND_TYPES {
            assert!(
                FORBIDDEN_TYPES
                    .binary_search_by_key(name, |(n, _)| *n)
                    .is_ok(),
                "`{name}` is format-bound but not in FORBIDDEN_TYPES"
            );
        }
    }

    #[test]
    fn keywords_are_sorted_and_unique() {
        assert!(
            KEYWORDS.windows(2).all(|w| w[0] < w[1]),
            "KEYWORDS must be sorted"
        );
    }

    #[test]
    fn escapes_hard_keywords_with_backticks() {
        assert_eq!(RULES.escape("in"), "`in`");
        assert_eq!(RULES.escape("object"), "`object`");
        assert_eq!(RULES.escape("fun"), "`fun`");
        assert_eq!(RULES.escape("when"), "`when`");
    }

    #[test]
    fn leaves_soft_keywords_alone() {
        // `value` and `field` are the synthetic names the emitter gives
        // newtype and tuple members; both are soft keywords and must stay bare.
        assert_eq!(RULES.escape("value"), "value");
        assert_eq!(RULES.escape("field"), "field");
        assert_eq!(RULES.escape("field0"), "field0");
        assert_eq!(RULES.escape("import"), "import");
        assert_eq!(RULES.escape("default"), "default");
        assert_eq!(RULES.escape("data"), "data");
    }

    #[test]
    fn escaping_is_idempotent() {
        assert_eq!(RULES.escape("`in`"), "`in`");
    }
}
