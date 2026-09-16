//! Swift reserved words.
//!
//! Source: *The Swift Programming Language* — Language Reference → Lexical
//! Structure → Keywords and Punctuation. Keywords used in declarations, in
//! statements, and in expressions and types.
//!
//! `#`-prefixed keywords cannot collide with a generated identifier, and
//! contextual keywords (`get`, `set`, `Type`, `Protocol`, …) are legal
//! identifiers, so neither group is listed.

use crate::generation::naming::{EscapeStyle, NamingRules};

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

/// The naming rules for this language.
pub(crate) const RULES: NamingRules = NamingRules {
    reserved_words: KEYWORDS,
    escape_style: EscapeStyle::Backticks,
};

#[cfg(test)]
mod tests {
    use super::*;

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
