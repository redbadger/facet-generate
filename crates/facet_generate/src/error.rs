use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("problem reflecting type '{type_name:?}': {message}")]
    ReflectionError { type_name: String, message: String },
    #[error("incomplete reflection detected")]
    UnknownFormat,
    #[error(
        "unsupported generic type: {0}, the type may have already been used with different parameters"
    )]
    UnsupportedGenericType(String),
    #[error("unsupported layout: {0}")]
    LayoutUnsized(String),
    #[error(
        r#"bad attribute format: use `#[facet(fg::namespace = "my_ns")]` or `#[facet(fg::namespace)]`"#
    )]
    InvalidNamespaceFormat,
    #[error("invalid namespace identifier")]
    InvalidNamespaceIdentifier,
    #[error(r#"ambiguous namespace inheritance: "{type_name}" in both "{existing_namespace}" and "{new_namespace}""#)]
    AmbiguousNamespaceInheritance {
        type_name: String,
        existing_namespace: String,
        new_namespace: String,
    },
    #[error(
        r#"two types generate as "{name}" in namespace "{namespace}": `{existing}` and `{new}`. Rename one with `#[facet(rename = "...")]` or give it its own namespace with `#[facet(fg::namespace = "...")]`"#
    )]
    DuplicateTypeName {
        name: String,
        namespace: String,
        existing: String,
        new: String,
    },
    /// A type reference that names no type in the registry. Reflection registers every type it
    /// refers to, so this is a bug in reflection rather than in the reflected types.
    #[error(
        r#"`{location}` in "{name}" in namespace "{namespace}" refers to "{missing_name}" in namespace "{missing_namespace}", which is not a registered type. This is a bug in facet_generate's reflection; please report it"#
    )]
    DanglingTypeReference {
        name: String,
        namespace: String,
        location: String,
        missing_name: String,
        missing_namespace: String,
    },
}
