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
}
