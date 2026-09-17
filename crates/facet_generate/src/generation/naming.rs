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

use std::{borrow::Cow, io};

use crate::{
    Registry,
    reflection::format::{ContainerFormat, Format, Named, VariantFormat},
};

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

/// A forbidden name together with the clause explaining why, used to build the
/// pre-pass error message. Entries are sorted by name for `binary_search`.
pub(crate) type ForbiddenNames = &'static [(&'static str, &'static str)];

/// The naming rules for one target language.
///
/// Keeping the rules in one struct per language means call sites do not have to
/// know which list they need: Tier A escaping reads `reserved_words` /
/// `escape_style`, and the Tier C pre-pass reads everything else.
pub(crate) struct NamingRules {
    /// The language name as it appears at the start of a pre-pass error.
    pub language: &'static str,
    /// Words that cannot be used as a bare identifier, sorted for
    /// `binary_search`.
    pub reserved_words: &'static [&'static str],
    /// How a reserved word is escaped.
    pub escape_style: EscapeStyle,
    /// Type names the generated module already uses for something else, each
    /// with the clause naming what it collides with.
    pub forbidden_types: ForbiddenNames,
    /// Member names the generated code cannot accommodate, each with the clause
    /// explaining why.
    pub forbidden_members: ForbiddenNames,
    /// How a container or variant name is cased in the generated source.
    pub type_case: fn(&str) -> String,
    /// How a field name is cased in the generated source.
    pub member_case: fn(&str) -> String,
    /// Whether each variant of a data-carrying enum becomes a type of its own
    /// (a nested class in Kotlin, a record in C#).
    pub variants_are_types: bool,
    /// Whether a member may not share its name with the type that encloses it
    /// (C# CS0542).
    pub member_equals_type_forbidden: bool,
    /// Whether `component1`..`componentN` are generated for an N-field type
    /// (Kotlin data classes).
    pub numbered_components_forbidden: bool,
}

/// The key under which the reason for a `componentN` clash is looked up in
/// [`NamingRules::forbidden_members`].
const COMPONENT_N: &str = "componentN";

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

    /// The reason clause for a forbidden type name, if it is forbidden.
    fn forbidden_type(&self, name: &str) -> Option<&'static str> {
        self.forbidden_types
            .binary_search_by_key(&name, |(n, _)| *n)
            .ok()
            .map(|i| self.forbidden_types[i].1)
    }

    /// The reason clause for a forbidden member name, if it is forbidden.
    fn forbidden_member(&self, name: &str) -> Option<&'static str> {
        self.forbidden_members
            .binary_search_by_key(&name, |(n, _)| *n)
            .ok()
            .map(|i| self.forbidden_members[i].1)
    }

    fn reason_for(&self, key: &str) -> &'static str {
        self.forbidden_member(key)
            .expect("forbidden member reason must be present")
    }
}

// ---------------------------------------------------------------------------
// Tier B — qualifying a builtin that a declaration shadows
// ---------------------------------------------------------------------------

/// The fully qualified spelling of `name`, when the generated module declares a
/// type of that name and would otherwise resolve the bare word to the
/// declaration instead of the builtin.
///
/// `table` maps the bare builtin to its qualified form and is sorted by the
/// bare name; `is_shadowed` decides, in the target language's own casing,
/// whether a declaration of that name exists.
pub(crate) fn qualify<'a>(
    name: &'a str,
    table: &[(&'static str, &'static str)],
    is_shadowed: impl Fn(&str) -> bool,
) -> Cow<'a, str> {
    match table.binary_search_by_key(&name, |(bare, _)| *bare) {
        Ok(i) if is_shadowed(name) => Cow::Borrowed(table[i].1),
        _ => Cow::Borrowed(name),
    }
}

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Rewrite every shadowed builtin in a module-level helper snippet to its
/// qualified form.
///
/// Replacement is word-boundary aware: `List<T>` is rewritten but
/// `ArrayList<T>` is not (it has its own table entry), nor is `List` inside a
/// longer identifier such as `deserializeListOf`. A name already preceded by a
/// `.` is part of a qualified path and is left alone.
pub(crate) fn qualify_helper<'a>(
    src: &'a str,
    table: &[(&'static str, &'static str)],
    is_shadowed: impl Fn(&str) -> bool,
) -> Cow<'a, str> {
    let shadowed: Vec<(&str, &str)> = table
        .iter()
        .filter(|(bare, _)| is_shadowed(bare))
        .copied()
        .collect();
    if shadowed.is_empty() {
        return Cow::Borrowed(src);
    }

    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    let mut at_word_start = true;
    while !rest.is_empty() {
        let matched = if at_word_start {
            shadowed.iter().find(|(bare, _)| {
                rest.starts_with(*bare)
                    && rest[bare.len()..]
                        .chars()
                        .next()
                        .is_none_or(|c| !is_identifier_char(c))
            })
        } else {
            None
        };

        if let Some((bare, qualified)) = matched {
            out.push_str(qualified);
            rest = &rest[bare.len()..];
            at_word_start = false;
        } else {
            let ch = rest.chars().next().expect("rest is not empty");
            out.push(ch);
            rest = &rest[ch.len_utf8()..];
            at_word_start = !is_identifier_char(ch) && ch != '.';
        }
    }
    Cow::Owned(out)
}

// ---------------------------------------------------------------------------
// Tier C — the reserved-name pre-pass
// ---------------------------------------------------------------------------

/// Reject any registry name that the generated code cannot accommodate by
/// escaping (Tier A) or by qualifying the builtin it shadows (Tier B).
///
/// Run before anything is written, so a failing registry produces no output.
///
/// # Errors
///
/// Returns [`io::ErrorKind::InvalidInput`] naming the offending type or field.
pub(crate) fn check_reserved_names(registry: &Registry, rules: &NamingRules) -> io::Result<()> {
    for (type_name, container) in registry {
        check_type_name(&type_name.name, rules)?;

        match container {
            ContainerFormat::UnitStruct(_) => {}
            ContainerFormat::NewTypeStruct(..) => {
                check_member(&type_name.name, "value", rules, 1)?;
            }
            ContainerFormat::TupleStruct(formats, _) => {
                for i in 0..formats.len() {
                    check_member(&type_name.name, &format!("field{i}"), rules, formats.len())?;
                }
            }
            ContainerFormat::Struct(fields, _) => {
                check_fields(&type_name.name, fields, rules)?;
            }
            ContainerFormat::Enum(variants, _, _) => {
                for variant in variants.values() {
                    check_variant(&type_name.name, variant, rules)?;
                }
            }
        }
    }

    Ok(())
}

fn check_type_name(name: &str, rules: &NamingRules) -> io::Result<()> {
    let cased = (rules.type_case)(name);
    let reason = rules
        .forbidden_type(name)
        .or_else(|| rules.forbidden_type(&cased));
    if let Some(what) = reason {
        return Err(invalid(format!(
            "{lang}: type `{name}` collides with {what} used by the generated code; \
             rename it with #[facet(rename = \"...\")]",
            lang = rules.language,
        )));
    }
    Ok(())
}

fn check_variant(
    enum_name: &str,
    variant: &Named<VariantFormat>,
    rules: &NamingRules,
) -> io::Result<()> {
    if rules.variants_are_types {
        check_type_name(&variant.name, rules)?;
    }

    // The type that encloses a variant's members is the variant itself where
    // variants become types, and the enum otherwise.
    let owner = if rules.variants_are_types {
        &variant.name
    } else {
        enum_name
    };

    match &variant.value {
        VariantFormat::Unit | VariantFormat::Variable(_) => Ok(()),
        VariantFormat::NewType(_) => check_member(owner, "value", rules, 1),
        VariantFormat::Tuple(formats) => {
            for i in 0..formats.len() {
                check_member(owner, &format!("field{i}"), rules, formats.len())?;
            }
            Ok(())
        }
        VariantFormat::Struct(fields) => check_fields(owner, fields, rules),
    }
}

fn check_fields(owner: &str, fields: &[Named<Format>], rules: &NamingRules) -> io::Result<()> {
    for field in fields {
        check_member(owner, &field.name, rules, fields.len())?;
    }
    Ok(())
}

fn check_member(
    owner: &str,
    rust_name: &str,
    rules: &NamingRules,
    field_count: usize,
) -> io::Result<()> {
    let ident = (rules.member_case)(rust_name);

    if let Some(reason) = rules.forbidden_member(&ident) {
        return Err(invalid(format!(
            "{lang}: field `{rust_name}` of `{owner}` would become `{ident}`, which {reason}; \
             rename it with #[facet(rename = \"...\")]",
            lang = rules.language,
        )));
    }

    if rules.numbered_components_forbidden
        && let Some(n) = ident
            .strip_prefix("component")
            .and_then(|n| n.parse::<usize>().ok())
        && n >= 1
        && n <= field_count
    {
        return Err(invalid(format!(
            "{lang}: field `{rust_name}` of `{owner}` would become `{ident}`, which {reason}; \
             rename it with #[facet(rename = \"...\")]",
            lang = rules.language,
            reason = rules.reason_for(COMPONENT_N),
        )));
    }

    if rules.member_equals_type_forbidden && ident == (rules.type_case)(owner) {
        return Err(invalid(format!(
            "{lang}: field `{rust_name}` of `{owner}` would become property `{ident}`, \
             the same name as its enclosing type (CS0542); \
             rename it with #[facet(rename = \"...\")]",
            lang = rules.language,
        )));
    }

    Ok(())
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: &[&str] = &["class", "default", "let"];

    fn identity(name: &str) -> String {
        name.to_string()
    }

    const fn rules(escape_style: EscapeStyle) -> NamingRules {
        NamingRules {
            language: "Test",
            reserved_words: WORDS,
            escape_style,
            forbidden_types: &[],
            forbidden_members: &[],
            type_case: identity,
            member_case: identity,
            variants_are_types: false,
            member_equals_type_forbidden: false,
            numbered_components_forbidden: false,
        }
    }

    const BACKTICKS: NamingRules = rules(EscapeStyle::Backticks);
    const AT_PREFIX: NamingRules = rules(EscapeStyle::AtPrefix);
    const UNDERSCORE: NamingRules = rules(EscapeStyle::UnderscoreSuffix);

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

    const TABLE: &[(&str, &str)] = &[
        ("ArrayList", "kotlin.collections.ArrayList"),
        ("List", "kotlin.collections.List"),
        ("Set", "kotlin.collections.Set"),
    ];

    fn shadowed<'a>(names: &'a [&'a str]) -> impl Fn(&str) -> bool + 'a {
        move |name: &str| names.contains(&name)
    }

    #[test]
    fn qualify_rewrites_only_shadowed_entries() {
        assert_eq!(
            qualify("Set", TABLE, shadowed(&["Set"])),
            "kotlin.collections.Set"
        );
        assert_eq!(qualify("List", TABLE, shadowed(&["Set"])), "List");
        assert_eq!(qualify("Unit", TABLE, shadowed(&["Unit"])), "Unit");
    }

    #[test]
    fn qualify_helper_is_a_no_op_when_nothing_is_shadowed() {
        let src = "fun <T> List<T>.serialize()";
        assert!(matches!(
            qualify_helper(src, TABLE, shadowed(&[])),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn qualify_helper_respects_word_boundaries() {
        assert_eq!(
            qualify_helper("fun <T> List<T>.serialize()", TABLE, shadowed(&["List"])),
            "fun <T> kotlin.collections.List<T>.serialize()"
        );
        // A longer identifier that merely starts with the bare name is left
        // alone — `ArrayList` has its own entry.
        assert_eq!(
            qualify_helper(
                "val list = ArrayList<T>(capacity)",
                TABLE,
                shadowed(&["List"])
            ),
            "val list = ArrayList<T>(capacity)"
        );
        assert_eq!(
            qualify_helper(
                "val list = ArrayList<T>(capacity)",
                TABLE,
                shadowed(&["ArrayList"])
            ),
            "val list = kotlin.collections.ArrayList<T>(capacity)"
        );
        // Nor is a bare name that sits inside a longer identifier.
        assert_eq!(
            qualify_helper(
                "deserializer.deserializeListOf {}",
                TABLE,
                shadowed(&["List"])
            ),
            "deserializer.deserializeListOf {}"
        );
        // An already-qualified path is not qualified again.
        assert_eq!(
            qualify_helper("kotlin.collections.List<T>", TABLE, shadowed(&["List"])),
            "kotlin.collections.List<T>"
        );
    }

    #[test]
    fn qualify_helper_rewrites_every_occurrence() {
        assert_eq!(
            qualify_helper("Set<T> to Set<U>", TABLE, shadowed(&["Set"])),
            "kotlin.collections.Set<T> to kotlin.collections.Set<U>"
        );
    }

    #[test]
    fn word_lists_are_sorted_for_binary_search() {
        for word in WORDS {
            assert!(BACKTICKS.is_reserved(word));
        }
    }
}
