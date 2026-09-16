//! C# reserved words.
//!
//! Source: learn.microsoft.com — C# reference → Keywords. Contextual keywords
//! are intentionally omitted: the generated identifiers that need escaping are
//! method locals and parameters, where those words are valid unescaped.

use crate::generation::naming::{EscapeStyle, NamingRules};

/// C# keywords, sorted.
pub(crate) const KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "base",
    "bool",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "checked",
    "class",
    "const",
    "continue",
    "decimal",
    "default",
    "delegate",
    "do",
    "double",
    "else",
    "enum",
    "event",
    "explicit",
    "extern",
    "false",
    "finally",
    "fixed",
    "float",
    "for",
    "foreach",
    "goto",
    "if",
    "implicit",
    "in",
    "int",
    "interface",
    "internal",
    "is",
    "lock",
    "long",
    "namespace",
    "new",
    "null",
    "object",
    "operator",
    "out",
    "override",
    "params",
    "private",
    "protected",
    "public",
    "readonly",
    "ref",
    "return",
    "sbyte",
    "sealed",
    "short",
    "sizeof",
    "stackalloc",
    "static",
    "string",
    "struct",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "uint",
    "ulong",
    "unchecked",
    "unsafe",
    "ushort",
    "using",
    "virtual",
    "void",
    "volatile",
    "while",
];

/// The naming rules for this language.
pub(crate) const RULES: NamingRules = NamingRules {
    reserved_words: KEYWORDS,
    escape_style: EscapeStyle::AtPrefix,
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
    fn escapes_keywords_with_an_at_sign() {
        assert_eq!(RULES.escape("event"), "@event");
        assert_eq!(RULES.escape("class"), "@class");
        assert_eq!(RULES.escape("default"), "@default");
    }

    #[test]
    fn leaves_ordinary_and_contextual_words_alone() {
        assert_eq!(RULES.escape("value"), "value");
        assert_eq!(RULES.escape("field0"), "field0");
        assert_eq!(RULES.escape("var"), "var");
        assert_eq!(RULES.escape("record"), "record");
    }

    #[test]
    fn escaping_is_idempotent() {
        assert_eq!(RULES.escape("@class"), "@class");
    }
}
