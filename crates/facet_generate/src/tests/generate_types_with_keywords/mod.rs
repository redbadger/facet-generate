#![expect(unused)]

use facet::Facet;

/// A struct whose every field is a keyword in at least one target language.
/// Each language escapes only its own reserved words: `import` is escaped in
/// Swift and TypeScript but is a soft keyword in Kotlin, and `type` is
/// contextual everywhere, so both come through bare where they are legal.
#[derive(Facet)]
#[allow(clippy::struct_excessive_bools)]
pub struct KeywordFields {
    pub r#default: String,
    pub r#in: i32,
    pub class: bool,
    pub object: String,
    pub r#static: bool,
    pub r#let: String,
    pub when: i32,
    pub is: bool,
    pub fun: String,
    pub operator: String,
    pub import: String,
    pub r#type: String,
    pub function: Option<String>,
    /// A tuple field: the Swift plugin derives `whereField0` / `whereField1`
    /// locals from this name, which must stay unescaped.
    pub r#where: (i32, String),
}

/// Tuple struct — its members are named `field0`, `field1`, which are never
/// keywords.
#[derive(Facet)]
pub struct KeywordTuple(pub String, pub i32);

/// Newtype struct — its member is named `value`, a Kotlin soft keyword that
/// must not be escaped.
#[derive(Facet)]
pub struct KeywordNewType(pub String);

#[derive(Facet)]
#[repr(C)]
pub enum KeywordEnum {
    Default,
    Case,
    Switch(String),
    Where { r#in: i32, r#default: String },
}

crate::test! {
    KeywordFields, KeywordTuple, KeywordNewType, KeywordEnum for kotlin, swift, typescript, csharp
}
