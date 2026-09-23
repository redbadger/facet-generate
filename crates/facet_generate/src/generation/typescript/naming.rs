//! TypeScript reserved words.
//!
//! Source: ECMA-262 — `ReservedWord`, plus the words reserved only in strict
//! mode (which modules always are) and the two identifiers that strict mode
//! forbids as binding names, `arguments` and `eval`.
//!
//! These are illegal only as *binding* identifiers (parameters, `const`
//! declarations). They are perfectly legal as property names, so the emitter
//! keeps property and wire names untouched and renames bindings instead.

use std::borrow::Cow;

use heck::ToUpperCamelCase;

use crate::{
    generation::{
        config::CodeGeneratorConfig,
        naming::{EscapeStyle, ForbiddenNames, NamingRules, qualify},
    },
    reflection::format::QualifiedTypeName,
};

/// TypeScript reserved words, sorted.
pub(crate) const KEYWORDS: &[&str] = &[
    "arguments",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Global types the emitter writes bare, and the fully qualified form to use
/// instead when the module declares a type of the same name. Sorted by the
/// bare name.
///
/// A module-level `class Map` shadows the global `Map` for the whole module,
/// including type positions; `globalThis.Map<K, V>` reaches the global one.
pub(crate) const QUALIFIED: &[(&str, &str)] = &[
    ("Error", "globalThis.Error"),
    ("Map", "globalThis.Map"),
    ("Uint8Array", "globalThis.Uint8Array"),
];

/// Type names the generated module already uses for something else, with the
/// clause that names what each collides with. Sorted by name.
///
/// Both the verbatim registry name and its `UpperCamelCase` form are checked,
/// because the emitter upper-camel-cases type references.
pub(crate) const FORBIDDEN_TYPES: ForbiddenNames = &[
    (
        "BinaryDeserializer",
        "the runtime type `BinaryDeserializer`",
    ),
    ("BinarySerializer", "the runtime type `BinarySerializer`"),
    (
        "BincodeDeserializer",
        "the runtime type `BincodeDeserializer`",
    ),
    ("BincodeSerializer", "the runtime type `BincodeSerializer`"),
    ("Deserializer", "the `Deserializer` import"),
    ("ListTuple", "the module-level `ListTuple<T>` alias"),
    ("Optional", "the module-level `Optional<T>` alias"),
    ("Seq", "the module-level `Seq<T>` alias"),
    ("Serializer", "the `Serializer` import"),
    ("Tuple", "the module-level `Tuple<T>` alias"),
    ("Uuid", "the module-level `Uuid` alias"),
    ("bool", "the TypeScript type alias `bool`"),
    ("bytes", "the TypeScript type alias `bytes`"),
    ("char", "the TypeScript type alias `char`"),
    ("float32", "the TypeScript type alias `float32`"),
    ("float64", "the TypeScript type alias `float64`"),
    ("int128", "the TypeScript type alias `int128`"),
    ("int16", "the TypeScript type alias `int16`"),
    ("int32", "the TypeScript type alias `int32`"),
    ("int64", "the TypeScript type alias `int64`"),
    ("int8", "the TypeScript type alias `int8`"),
    ("str", "the TypeScript type alias `str`"),
    ("uint128", "the TypeScript type alias `uint128`"),
    ("uint16", "the TypeScript type alias `uint16`"),
    ("uint32", "the TypeScript type alias `uint32`"),
    ("uint64", "the TypeScript type alias `uint64`"),
    ("uint8", "the TypeScript type alias `uint8`"),
    ("unit", "the TypeScript type alias `unit`"),
];

/// Property names the generated code cannot accommodate, with the clause
/// explaining why. Sorted by name.
pub(crate) const FORBIDDEN_MEMBERS: ForbiddenNames = &[
    (
        "constructor",
        "is the constructor of every JavaScript class",
    ),
    (
        "deserializer",
        "shadows the `deserializer` parameter in the generated deserialize method",
    ),
    ("prototype", "is reserved on every JavaScript class"),
    (
        "serializer",
        "shadows the `serializer` parameter in the generated serialize method",
    ),
];

fn type_case(name: &str) -> String {
    name.to_upper_camel_case()
}

/// TypeScript property names are written verbatim.
fn member_case(name: &str) -> String {
    name.to_string()
}

/// The naming rules for this language.
pub(crate) const RULES: NamingRules = NamingRules {
    language: "TypeScript",
    reserved_words: KEYWORDS,
    escape_style: EscapeStyle::UnderscoreSuffix,
    forbidden_types: FORBIDDEN_TYPES,
    forbidden_members: FORBIDDEN_MEMBERS,
    format_bound_types: &[],
    type_case,
    member_case,
    variants_are_types: false,
    member_equals_type_forbidden: false,
    numbered_components_forbidden: false,
};

/// Returns `true` if the module declares a type whose `UpperCamelCase` name is
/// `name`.
pub(crate) fn shadows(name: &str, config: &CodeGeneratorConfig) -> bool {
    config
        .declared_type_names
        .iter()
        .any(|declared| declared.to_upper_camel_case() == name)
}

/// The TypeScript spelling of the builtin type `name`: reached through
/// `globalThis` when a declaration in the generated module shadows it, and bare
/// otherwise.
pub(crate) fn builtin<'a>(name: &'a str, config: &CodeGeneratorConfig) -> Cow<'a, str> {
    qualify(name, QUALIFIED, |n| shadows(n, config))
}

/// A reference to the standalone function `{prefix}{Name}` that the plugins
/// emit beside an enum (`serializeColor`, `deserializeColor`), qualified the
/// same way as a reference to the enum itself: bare within its own module,
/// and through the namespace import otherwise (`Kit.serializeColor`).
pub(crate) fn enum_function(prefix: &str, name: &QualifiedTypeName) -> String {
    QualifiedTypeName {
        namespace: name.namespace.clone(),
        name: format!("{prefix}{}", name.name),
    }
    .format(ToUpperCamelCase::to_upper_camel_case, ".")
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
    fn renames_reserved_words_with_a_trailing_underscore() {
        assert_eq!(RULES.escape("let"), "let_");
        assert_eq!(RULES.escape("default"), "default_");
        assert_eq!(RULES.escape("class"), "class_");
        assert_eq!(RULES.escape("arguments"), "arguments_");
    }

    #[test]
    fn leaves_ordinary_words_alone() {
        assert_eq!(RULES.escape("value"), "value");
        assert_eq!(RULES.escape("field0"), "field0");
        assert_eq!(RULES.escape("type"), "type");
        assert_eq!(RULES.escape("of"), "of");
    }

    #[test]
    fn escaping_is_idempotent() {
        assert_eq!(RULES.escape("let_"), "let_");
    }
}
