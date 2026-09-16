//! Kotlin reserved words.
//!
//! Source: kotlinlang.org — Keywords and operators → *Hard keywords*. Only
//! hard keywords are listed: soft keywords and modifier keywords (`value`,
//! `field`, `import`, `data`, `by`, `get`, `set`, …) are legal identifiers,
//! and two of them — `value` and `field` — are the synthetic names this
//! generator gives newtype and tuple members, so escaping them would change
//! existing output. `default` is not a Kotlin keyword either.

use crate::generation::naming::{EscapeStyle, NamingRules};

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
