//! Language-agnostic naming support.
//!
//! Every target language has words that cannot be used as a bare identifier.
//! Each language module owns its own word list (`<lang>/naming.rs`); this
//! module owns the shared driver that turns a word list plus an escaping
//! style into an escape function, so all four languages behave identically
//! apart from the syntax they use.
//!
//! Escaping is applied to the **post-casing** identifier: a Rust variant
//! `Default` becomes the Swift case `default` and only then is escaped to
//! `` `default` ``.

use std::borrow::Cow;

/// How a target language makes a reserved word usable as an identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EscapeStyle {
    /// Wrap in backticks — Swift and Kotlin. Pure quoting: the identifier's
    /// spelling (and therefore any wire name derived from it) is unchanged.
    Backticks,
    /// Prefix with `@` — C# verbatim identifiers. Also pure quoting.
    AtPrefix,
    /// Append an underscore — TypeScript, where reserved words are illegal in
    /// binding positions and cannot be quoted, so the identifier is renamed.
    UnderscoreSuffix,
}

/// The naming rules for one target language.
///
/// Tier B (shadow qualification) and Tier C (the reserved-name pre-pass) will
/// add further fields here; keeping the rules in one struct per language means
/// call sites do not have to know which list they need.
pub(crate) struct NamingRules {
    /// Words that cannot be used as a bare identifier, sorted for
    /// `binary_search`.
    pub reserved_words: &'static [&'static str],
    /// How a reserved word is escaped.
    pub escape_style: EscapeStyle,
}

impl NamingRules {
    /// Returns `true` if `identifier` cannot be written bare in this language.
    pub(crate) fn is_reserved(&self, identifier: &str) -> bool {
        self.reserved_words.binary_search(&identifier).is_ok()
    }

    /// Escape `identifier` if it is reserved, otherwise return it unchanged.
    ///
    /// Idempotent: an already-escaped identifier is never reserved, so
    /// escaping it a second time is a no-op.
    pub(crate) fn escape<'a>(&self, identifier: &'a str) -> Cow<'a, str> {
        if self.is_reserved(identifier) {
            Cow::Owned(match self.escape_style {
                EscapeStyle::Backticks => format!("`{identifier}`"),
                EscapeStyle::AtPrefix => format!("@{identifier}"),
                EscapeStyle::UnderscoreSuffix => format!("{identifier}_"),
            })
        } else {
            Cow::Borrowed(identifier)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: &[&str] = &["class", "default", "let"];

    const BACKTICKS: NamingRules = NamingRules {
        reserved_words: WORDS,
        escape_style: EscapeStyle::Backticks,
    };
    const AT_PREFIX: NamingRules = NamingRules {
        reserved_words: WORDS,
        escape_style: EscapeStyle::AtPrefix,
    };
    const UNDERSCORE: NamingRules = NamingRules {
        reserved_words: WORDS,
        escape_style: EscapeStyle::UnderscoreSuffix,
    };

    #[test]
    fn escapes_only_reserved_words() {
        assert_eq!(BACKTICKS.escape("default"), "`default`");
        assert_eq!(BACKTICKS.escape("name"), "name");
        assert_eq!(AT_PREFIX.escape("default"), "@default");
        assert_eq!(AT_PREFIX.escape("name"), "name");
        assert_eq!(UNDERSCORE.escape("let"), "let_");
        assert_eq!(UNDERSCORE.escape("name"), "name");
    }

    #[test]
    fn escaping_is_idempotent() {
        for rules in [&BACKTICKS, &AT_PREFIX, &UNDERSCORE] {
            let once = rules.escape("default").into_owned();
            assert_eq!(rules.escape(&once), once);
        }
    }

    #[test]
    fn non_reserved_words_are_borrowed() {
        assert!(matches!(BACKTICKS.escape("name"), Cow::Borrowed(_)));
    }

    #[test]
    fn word_lists_are_sorted_for_binary_search() {
        for word in WORDS {
            assert!(BACKTICKS.is_reserved(word));
        }
    }
}
