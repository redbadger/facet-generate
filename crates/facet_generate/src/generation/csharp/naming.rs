//! C# reserved words.
//!
//! Source: learn.microsoft.com — C# reference → Keywords. Contextual keywords
//! are intentionally omitted: the generated identifiers that need escaping are
//! method locals and parameters, where those words are valid unescaped.

use std::borrow::Cow;

use heck::ToUpperCamelCase;

use crate::{
    generation::{
        config::CodeGeneratorConfig,
        naming::{EscapeStyle, ForbiddenNames, FormatBoundNames, NamingRules, qualify},
    },
    reflection::format::{Format, Namespace},
};

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

/// Types the emitter writes bare through a `using` directive, and the fully
/// qualified form to use instead when the module declares a type of the same
/// name. Sorted by the bare name.
///
/// A declaration in the generated namespace outranks a `using`-imported type,
/// so `HashSet<string>` in a namespace that also declares a `HashSet` record
/// would resolve to the record. Lower-case keyword types (`int`, `string`, …)
/// cannot be shadowed and are not listed.
pub(crate) const QUALIFIED: &[(&str, &str)] = &[
    (
        "Dictionary",
        "global::System.Collections.Generic.Dictionary",
    ),
    ("Guid", "global::System.Guid"),
    ("HashSet", "global::System.Collections.Generic.HashSet"),
    ("Int128", "global::System.Int128"),
    (
        "ObservableCollection",
        "global::System.Collections.ObjectModel.ObservableCollection",
    ),
    ("UInt128", "global::System.UInt128"),
    ("Unit", "global::Facet.Runtime.Serde.Unit"),
];

/// The formats each entry of [`QUALIFIED`] is written for, so a registry that
/// has none of them never writes that builtin. Sorted by the bare name.
///
/// A namespace named after a builtin cannot be qualified around, as a type
/// can: it is a member of the root package's namespace, which every generated
/// module is nested in, so the installer rejects it when the registry has a
/// format that writes the builtin bare.
pub(crate) const QUALIFIED_FORMATS: FormatBoundNames = &[
    ("Dictionary", |format| matches!(format, Format::Map { .. })),
    ("Guid", |format| matches!(format, Format::Uuid)),
    ("HashSet", |format| matches!(format, Format::Set(_))),
    ("Int128", |format| matches!(format, Format::I128)),
    ("ObservableCollection", |format| {
        matches!(format, Format::Seq(_))
    }),
    ("UInt128", |format| matches!(format, Format::U128)),
    ("Unit", |format| match format {
        Format::Unit => true,
        Format::Tuple(formats) => formats.is_empty(),
        _ => false,
    }),
];

/// Type names the generated module already uses for something else, with the
/// clause that names what each collides with. Sorted by name.
pub(crate) const FORBIDDEN_TYPES: ForbiddenNames = &[
    (
        "BincodeDeserializer",
        "the runtime type `Facet.Runtime.Bincode.BincodeDeserializer`",
    ),
    (
        "BincodeSerializer",
        "the runtime type `Facet.Runtime.Bincode.BincodeSerializer`",
    ),
    (
        "DeserializationError",
        "the runtime type `Facet.Runtime.Serde.DeserializationError`",
    ),
    ("Enum", "the .NET base type `System.Enum`"),
    ("Facet", "the `Facet` root namespace"),
    (
        "FacetHelpers",
        "the runtime type `Facet.Runtime.Serde.FacetHelpers`",
    ),
    (
        "IDeserializer",
        "the runtime interface `Facet.Runtime.Serde.IDeserializer`",
    ),
    (
        "IFacetDeserializable",
        "the runtime interface `Facet.Runtime.Serde.IFacetDeserializable`",
    ),
    (
        "IFacetSerializable",
        "the runtime interface `Facet.Runtime.Serde.IFacetSerializable`",
    ),
    (
        "ISerializer",
        "the runtime interface `Facet.Runtime.Serde.ISerializer`",
    ),
    (
        "JsonConverter",
        "the `System.Text.Json.Serialization.JsonConverter` attribute",
    ),
    (
        "JsonDerivedType",
        "the `System.Text.Json.Serialization.JsonDerivedType` attribute",
    ),
    (
        "JsonPolymorphic",
        "the `System.Text.Json.Serialization.JsonPolymorphic` attribute",
    ),
    (
        "JsonPropertyName",
        "the `System.Text.Json.Serialization.JsonPropertyName` attribute",
    ),
    (
        "JsonSerde",
        "the runtime type `Facet.Runtime.Json.JsonSerde`",
    ),
    (
        "JsonStringEnumConverter",
        "the `System.Text.Json.Serialization.JsonStringEnumConverter` converter",
    ),
    ("Object", "the .NET base type `System.Object`"),
    (
        "ObservableObject",
        "the `CommunityToolkit.Mvvm.ComponentModel.ObservableObject` base class",
    ),
    (
        "ObservableProperty",
        "the `CommunityToolkit.Mvvm.ComponentModel.ObservableProperty` attribute",
    ),
    (
        "SerializationError",
        "the runtime type `Facet.Runtime.Serde.SerializationError`",
    ),
    ("String", "the .NET type `System.String`"),
    ("System", "the `System` root namespace"),
    ("Task", "the .NET type `System.Threading.Tasks.Task`"),
    (
        "UuidSerde",
        "the runtime type `Facet.Runtime.Serde.UuidSerde`",
    ),
];

/// Property names the generated code cannot accommodate, with the clause
/// explaining why. Sorted by name.
pub(crate) const FORBIDDEN_MEMBERS: ForbiddenNames = &[
    (
        "Deserializer",
        "yields a `deserializer` local that shadows the parameter of the generated Deserialize method",
    ),
    ("Equals", "hides an inherited member of every C# object"),
    (
        "GetHashCode",
        "hides an inherited member of every C# object",
    ),
    ("GetType", "hides an inherited member of every C# object"),
    (
        "Serializer",
        "yields a `serializer` local that shadows the parameter of the generated Serialize method",
    ),
    ("ToString", "hides an inherited member of every C# object"),
];

fn type_case(name: &str) -> String {
    name.to_upper_camel_case()
}

fn member_case(name: &str) -> String {
    name.to_upper_camel_case()
}

/// The naming rules for this language.
pub(crate) const RULES: NamingRules = NamingRules {
    language: "C#",
    reserved_words: KEYWORDS,
    escape_style: EscapeStyle::AtPrefix,
    forbidden_types: FORBIDDEN_TYPES,
    forbidden_members: FORBIDDEN_MEMBERS,
    format_bound_types: &[],
    type_case,
    member_case,
    variants_are_types: true,
    member_equals_type_forbidden: true,
    numbered_components_forbidden: false,
};

/// Returns `true` if a type whose `UpperCamelCase` name is `name` is in scope
/// in the module: one it declares, or a ROOT type, which a namespaced module
/// sees because its namespace is nested inside the root module's.
pub(crate) fn shadows(name: &str, config: &CodeGeneratorConfig) -> bool {
    config
        .declared_type_names
        .iter()
        .chain(
            config
                .registry_type_names
                .iter()
                .filter(|declared| declared.namespace == Namespace::Root)
                .map(|declared| &declared.name),
        )
        .any(|declared| declared.to_upper_camel_case() == name)
}

/// The C# spelling of the builtin type `name`: fully qualified with
/// `global::` when a declaration in the generated namespace shadows it, and
/// bare otherwise.
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
    fn every_qualified_builtin_has_its_formats() {
        let qualified: Vec<&str> = QUALIFIED.iter().map(|(bare, _)| *bare).collect();
        let with_formats: Vec<&str> = QUALIFIED_FORMATS.iter().map(|(bare, _)| *bare).collect();
        assert_eq!(qualified, with_formats);
    }

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
