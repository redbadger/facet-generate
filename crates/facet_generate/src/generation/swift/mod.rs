//! Swift code generation.
//!
//! This module translates a [`Registry`](crate::Registry) of reflected type
//! definitions into idiomatic Swift source code.
//!
//! # Submodules (in pipeline order)
//!
//! 1. **`generator`** — Top-level orchestrator. [`CodeGenerator`](crate::generation::swift::CodeGenerator) implements
//!    [`CodeGen`](crate::generation::CodeGen) to produce a complete Swift source file from a
//!    registry. It resolves qualified type names against the configuration
//!    (external packages, namespaces) and then delegates writing to the emitter
//!    layer.
//!
//! 2. **`emitter`** — AST-to-source rendering. Implements
//!    [`Emitter<Swift>`](crate::generation::Emitter) for each AST node type
//!    ([`Module`](crate::generation::module::Module), [`Container`](crate::generation::Container),
//!    `Named<Format>`, `Format`, `Doc`). This is where the Swift language
//!    mapping lives: type names, `Serializer`/`Deserializer` protocol methods,
//!    `public struct` / `indirect public enum` selection, and bincode
//!    serialize/deserialize method generation. Feature helpers (`ListOfT`,
//!    `SetOfT`, etc.) are embedded as `include_bytes!` snippets and emitted as
//!    needed.
//!
//! 3. **`installer`** — Project scaffolding. [`Installer`](crate::generation::swift::Installer) implements
//!    [`SourceInstaller`](crate::generation::SourceInstaller) to write a ready-to-build
//!    Swift package: it copies serde runtime sources, splits the registry by
//!    namespace into per-module files, and generates a `Package.swift` manifest
//!    with SPM targets.

pub(crate) mod emitter;
mod generator;
mod installer;
mod package;

pub use emitter::Swift;
pub use generator::CodeGenerator;
pub use installer::Installer;

/// Normalize a path string for use in Swift string literals.
/// On Windows, replaces backslashes with forward slashes to avoid
/// Swift interpreting them as escape sequences.
#[must_use]
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}
