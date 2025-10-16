/// Utility function to generate indented text
pub mod indent;

/// Modules for code generation that map to Namespaces declared as `#[facet(namespace = "my_namespace")]`
pub mod module;

/// Support for code-generation in Java
#[cfg(feature = "java")]
pub mod java;
/// Support for code-generation in Kotlin
#[cfg(feature = "kotlin")]
pub mod kotlin;
/// Support for code-generation in Swift
#[cfg(feature = "swift")]
pub mod swift;
/// Support for code-generation in Swift
#[cfg(feature = "swift")]
pub mod swift2;
/// Support for code-generation in TypeScript
#[cfg(feature = "typescript")]
pub mod typescript;

/// Common logic for codegen.
#[cfg(any(
    feature = "java",
    feature = "kotlin",
    feature = "swift",
    feature = "typescript"
))]
pub mod common;
/// Common configuration objects and traits used in public APIs.
mod config;

use std::{
    fmt::{Display, Formatter},
    io::{Result, Write},
};

pub use config::*;
use indent::IndentWrite;

use crate::{
    Registry,
    reflection::format::{ContainerFormat, QualifiedTypeName},
};

pub trait CodeGen<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self;

    /// Generate code for the given [`Registry`] and write it to the provided `writer`.
    ///
    /// # Errors
    /// This function may fail if the writer encounters an error while writing the generated code.
    fn write_output<W: Write>(&mut self, writer: &mut W, registry: &Registry) -> Result<()>;
}

pub enum Language {
    Java,
    Kotlin,
    Swift,
    TypeScript,
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Java => write!(f, "Java"),
            Language::Kotlin => write!(f, "Kotlin"),
            Language::Swift => write!(f, "Swift"),
            Language::TypeScript => write!(f, "TypeScript"),
        }
    }
}

pub struct WithEncoding<T> {
    pub encoding: Encoding,
    pub value: T,
}

pub struct Container<'a> {
    pub name: &'a QualifiedTypeName,
    pub format: &'a ContainerFormat,
}

pub trait Emitter<Language> {
    /// Write the code to the provided `IndentWrite`.
    ///
    /// # Errors
    /// This function may fail if the writer encounters an error while writing the generated code.
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()>;
}

#[cfg(all(test, feature = "generate"))]
mod tests;
