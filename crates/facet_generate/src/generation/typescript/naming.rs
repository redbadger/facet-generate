//! TypeScript reserved words.
//!
//! Source: ECMA-262 — `ReservedWord`, plus the words reserved only in strict
//! mode (which modules always are) and the two identifiers that strict mode
//! forbids as binding names, `arguments` and `eval`.
//!
//! These are illegal only as *binding* identifiers (parameters, `const`
//! declarations). They are perfectly legal as property names, so the emitter
//! keeps property and wire names untouched and renames bindings instead.

use crate::generation::naming::{EscapeStyle, NamingRules};

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

/// The naming rules for this language.
pub(crate) const RULES: NamingRules = NamingRules {
    reserved_words: KEYWORDS,
    escape_style: EscapeStyle::UnderscoreSuffix,
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
