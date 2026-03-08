//! Code generation — transforms a [`Registry`](crate::Registry) into source code for target
//! languages.
//!
//! Each language has its own submodule (`kotlin`, `csharp`, `swift`, `typescript`) behind a
//! feature flag. The generation pipeline has three layers:
//!
//! - **Generator** ([`CodeGen`]) — top-level entry point that writes a complete output file
//!   from a registry.
//! - **Emitter** ([`Emitter`]) — walks the [`Format`](crate::reflection::format::Format) AST
//!   inside each [`Container`] and emits the target language equivalent for each node
//!   (type declarations, fields, serialization logic, etc.).
//! - **Installer** — (language-specific) scaffolds project files, package manifests, and
//!   generated source into an output directory.
//!
//! Shared infrastructure:
//! - [`CodeGeneratorConfig`] — configuration (package name, encoding, custom types, etc.)
//! - [`indent`] — indentation-aware writer
//! - [`module`] — splits a registry by namespace into separate output modules

/// Utility function to generate indented text
pub mod indent;

/// Modules for code generation that map to Namespaces declared as `#[facet(namespace = "my_namespace")]`
pub mod module;

/// Support for code-generation in C#
#[cfg(feature = "csharp")]
pub mod csharp;
/// Support for code-generation in Java.
///
/// **Deprecated since 0.16.0:** The Java generator is deprecated. Use the Kotlin generator instead.
#[cfg(feature = "java")]
pub mod java;
/// Support for code-generation in Kotlin
#[cfg(feature = "kotlin")]
pub mod kotlin;
/// Support for code-generation in Swift
#[cfg(feature = "swift")]
pub mod swift;
/// Support for code-generation in TypeScript
#[cfg(feature = "typescript")]
pub mod typescript;

/// Common configuration objects and traits used in public APIs.
mod config;

use std::io::{Result, Write};

pub use config::*;
use indent::IndentWrite;

use crate::{
    Registry,
    reflection::format::{ContainerFormat, QualifiedTypeName},
};

pub(crate) const SERDE_NAMESPACE: &str = "serde";
pub(crate) const BINCODE_NAMESPACE: &str = "bincode";

/// Transforms a [`Registry`] into a complete source file. Each target language provides
/// its own implementation.
pub trait CodeGen<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self;

    /// Generate code for the given [`Registry`] and write it to the provided `writer`.
    ///
    /// # Errors
    /// This function may fail if the writer encounters an error while writing the generated code.
    fn write_output<W: Write>(&mut self, writer: &mut W, registry: &Registry) -> Result<()>;
}

/// A borrowed view of a single type definition (struct or enum) from the
/// [`Registry`], ready to be passed to [`Emitter::write`].
pub struct Container<'a> {
    pub name: &'a QualifiedTypeName,
    pub format: &'a ContainerFormat,
}

impl<'a> From<(&'a QualifiedTypeName, &'a ContainerFormat)> for Container<'a> {
    fn from(value: (&'a QualifiedTypeName, &'a ContainerFormat)) -> Self {
        Container {
            name: value.0,
            format: value.1,
        }
    }
}

/// Emits target-language source code for a single AST node.
///
/// `Emitter` is the core abstraction of the code-generation pipeline. Each target
/// language defines a zero-sized (or near-zero-sized) **language tag** type — e.g.
/// [`csharp::CSharp`], [`kotlin::Kotlin`], [`swift::Swift`], [`typescript::TypeScript`]
/// — and then provides `Emitter<L>` implementations for the AST node types that
/// need to be rendered in that language.
///
/// # Type parameter
///
/// `L` is the **language tag**. It serves two purposes:
///
/// 1. **Dispatch** — A single AST type (e.g. [`Container`], [`Module`](module::Module),
///    `Named<Format>`) can implement `Emitter<L>` once per language, and the compiler
///    resolves the correct implementation from the tag alone.
/// 2. **Configuration** — Language tags carry per-invocation settings such as the
///    target [`Encoding`] (None / Json / Bincode), so implementations
///    can conditionally emit serialization methods.
///
/// # Typical implementors
///
/// | AST node | What it emits |
/// |---|---|
/// | [`Module`](module::Module) | File header: imports, package/namespace declarations |
/// | [`Container`] | A complete type: struct, enum, sealed class, etc. |
/// | `Named<Format>` | A single field / property declaration |
/// | `Format` | An inline type expression (e.g. `List<Int>`, `Optional<String>`) |
/// | `Doc` | Documentation comment (`///`, `/** */`, etc.) |
///
/// # Usage
///
/// Generators ([`CodeGen`] implementations) create an [`IndentedWriter`](indent::IndentedWriter),
/// construct the language tag, and then call `write` on each node in sequence:
///
/// ```rust,ignore
/// let w = &mut IndentedWriter::new(out, IndentConfig::Space(4));
/// let lang = CSharp::new(Encoding::Json);
///
/// Module::new(&config).write(w, lang)?;       // header
/// for container in containers {
///     container.write(w, lang)?;               // each type
/// }
/// ```
pub trait Emitter<L> {
    /// Write the code for this node to the provided [`IndentWrite`].
    ///
    /// `lang` selects the target language **and** carries configuration (e.g.
    /// encoding). When a type implements `Emitter` for multiple languages the
    /// compiler uses `lang` to disambiguate.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying writer fails.
    fn write<W: IndentWrite>(&self, writer: &mut W, lang: L) -> Result<()>;
}

#[cfg(all(test, feature = "generate"))]
mod tests;
