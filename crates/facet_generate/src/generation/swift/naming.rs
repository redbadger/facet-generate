//! Swift reserved words.
//!
//! Source: *The Swift Programming Language* — Language Reference → Lexical
//! Structure → Keywords and Punctuation. Keywords used in declarations, in
//! statements, and in expressions and types.
//!
//! `#`-prefixed keywords cannot collide with a generated identifier, and
//! contextual keywords (`get`, `set`, `Type`, `Protocol`, …) are legal
//! identifiers, so neither group is listed.

use std::borrow::Cow;

use heck::ToLowerCamelCase;

use crate::generation::{
    config::CodeGeneratorConfig,
    naming::{EscapeStyle, ForbiddenNames, NamingRules, qualify},
};

/// Swift keywords, sorted.
pub(crate) const KEYWORDS: &[&str] = &[
    "Any",
    "Self",
    "as",
    "associatedtype",
    "await",
    "borrowing",
    "break",
    "case",
    "catch",
    "class",
    "consuming",
    "continue",
    "default",
    "defer",
    "deinit",
    "do",
    "else",
    "enum",
    "extension",
    "fallthrough",
    "false",
    "fileprivate",
    "for",
    "func",
    "guard",
    "if",
    "import",
    "in",
    "init",
    "inout",
    "internal",
    "is",
    "let",
    "nil",
    "nonisolated",
    "open",
    "operator",
    "precedencegroup",
    "private",
    "protocol",
    "public",
    "repeat",
    "rethrows",
    "return",
    "self",
    "static",
    "struct",
    "subscript",
    "super",
    "switch",
    "throw",
    "throws",
    "true",
    "try",
    "typealias",
    "var",
    "where",
    "while",
];

/// Standard-library types the emitter writes bare, and the fully qualified
/// form to use instead when the module declares a type of the same name.
/// Sorted by the bare name.
///
/// A declaration in the generated module outranks the implicit `Swift` module,
/// so `Set<String>` in a module that also declares `struct Set` would resolve
/// to the struct.
pub(crate) const QUALIFIED: &[(&str, &str)] = &[
    ("Bool", "Swift.Bool"),
    ("Character", "Swift.Character"),
    ("Double", "Swift.Double"),
    ("Equatable", "Swift.Equatable"),
    ("Float", "Swift.Float"),
    ("Hashable", "Swift.Hashable"),
    ("Int16", "Swift.Int16"),
    ("Int32", "Swift.Int32"),
    ("Int64", "Swift.Int64"),
    ("Int8", "Swift.Int8"),
    ("Set", "Swift.Set"),
    ("String", "Swift.String"),
    ("UInt16", "Swift.UInt16"),
    ("UInt32", "Swift.UInt32"),
    ("UInt64", "Swift.UInt64"),
    ("UInt8", "Swift.UInt8"),
    ("UUID", "Foundation.UUID"),
    ("Void", "Swift.Void"),
];

/// Type names the generated module already uses for something else, with the
/// clause that names what each collides with. Sorted by name.
pub(crate) const FORBIDDEN_TYPES: ForbiddenNames = &[
    ("Any", "the Swift type `Any`"),
    (
        "BinaryDeserializer",
        "the runtime type `Serde.BinaryDeserializer`",
    ),
    (
        "BinarySerializer",
        "the runtime type `Serde.BinarySerializer`",
    ),
    (
        "BincodeDeserializer",
        "the runtime type `Serde.BincodeDeserializer`",
    ),
    (
        "BincodeSerializer",
        "the runtime type `Serde.BincodeSerializer`",
    ),
    (
        "DeserializationError",
        "the runtime type `Serde.DeserializationError`",
    ),
    ("Deserializer", "the runtime protocol `Serde.Deserializer`"),
    ("Foundation", "the `Foundation` module"),
    ("Indirect", "the generated `@Indirect` property wrapper"),
    ("Int128", "the Swift type `Int128`"),
    (
        "JsonDeserializer",
        "the runtime type `Serde.JsonDeserializer`",
    ),
    ("JsonSerializer", "the runtime type `Serde.JsonSerializer`"),
    ("Protocol", "the Swift metatype keyword `Protocol`"),
    ("Self", "the Swift implicit type reference `Self`"),
    ("Serde", "the `Serde` module"),
    (
        "SerializationError",
        "the runtime type `Serde.SerializationError`",
    ),
    ("Serializer", "the runtime protocol `Serde.Serializer`"),
    ("Type", "the Swift metatype keyword `Type`"),
    ("UInt128", "the Swift type `UInt128`"),
];

/// Property names the generated code cannot accommodate, with the clause
/// explaining why. Sorted by name.
pub(crate) const FORBIDDEN_MEMBERS: ForbiddenNames = &[
    (
        "Self",
        "is the implicit type reference inside every Swift type",
    ),
    (
        "deserializer",
        "shadows the `deserializer` parameter in the generated deserialize method",
    ),
    ("hashValue", "Swift synthesises for every `Hashable` type"),
    (
        "index",
        "shadows the `index` local in the generated deserialize method",
    ),
    ("init", "is the initializer keyword in every Swift type"),
    ("self", "is the implicit receiver inside every Swift type"),
    (
        "serializer",
        "shadows the `serializer` parameter in the generated serialize method",
    ),
];

/// Swift writes container and case-carrying variant names verbatim.
fn type_case(name: &str) -> String {
    name.to_string()
}

fn member_case(name: &str) -> String {
    name.to_lower_camel_case()
}

/// The naming rules for this language.
pub(crate) const RULES: NamingRules = NamingRules {
    language: "Swift",
    reserved_words: KEYWORDS,
    escape_style: EscapeStyle::Backticks,
    forbidden_types: FORBIDDEN_TYPES,
    forbidden_members: FORBIDDEN_MEMBERS,
    format_bound_types: &[],
    type_case,
    member_case,
    variants_are_types: false,
    member_equals_type_forbidden: false,
    numbered_components_forbidden: false,
};

/// Returns `true` if the module declares a type spelled `name`.
pub(crate) fn shadows(name: &str, config: &CodeGeneratorConfig) -> bool {
    config.declared_type_names.contains(name)
}

/// The Swift spelling of the builtin type `name`: fully qualified when a
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
    }

    #[test]
    fn keywords_are_sorted_and_unique() {
        assert!(
            KEYWORDS.windows(2).all(|w| w[0] < w[1]),
            "KEYWORDS must be sorted"
        );
    }

    #[test]
    fn escapes_keywords_with_backticks() {
        assert_eq!(RULES.escape("default"), "`default`");
        assert_eq!(RULES.escape("case"), "`case`");
        assert_eq!(RULES.escape("in"), "`in`");
        assert_eq!(RULES.escape("Self"), "`Self`");
    }

    #[test]
    fn leaves_ordinary_and_contextual_words_alone() {
        assert_eq!(RULES.escape("value"), "value");
        assert_eq!(RULES.escape("field0"), "field0");
        assert_eq!(RULES.escape("get"), "get");
        assert_eq!(RULES.escape("set"), "set");
        assert_eq!(RULES.escape("type"), "type");
    }

    #[test]
    fn escaping_is_idempotent() {
        assert_eq!(RULES.escape("`default`"), "`default`");
    }
}
