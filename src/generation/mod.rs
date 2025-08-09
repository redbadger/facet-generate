// Copyright (c) Facebook, Inc. and its affiliates
// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: MIT OR Apache-2.0

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
mod common;
/// Common configuration objects and traits used in public APIs.
mod config;

use std::io::{Result, Write};

pub use config::*;
use indent::IndentWrite;

use crate::Registry;

pub trait Language<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self;

    /// Generate code for the given [`Registry`] and write it to the provided `writer`.
    ///
    /// # Errors
    /// This function may fail if the writer encounters an error while writing the generated code.
    fn write_output<W: Write>(&mut self, writer: &mut W, registry: &Registry) -> Result<()>;
}

#[cfg(all(test, feature = "generate"))]
mod tests;

pub trait Emitter<Language> {
    /// Write the code to the provided `writer`.
    ///
    /// # Errors
    /// This function may fail if the writer encounters an error while writing the generated code.
    fn write<W: IndentWrite>(&self, writer: &mut W) -> Result<()>;
}
