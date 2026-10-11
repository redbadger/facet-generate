//! Kotlin code generation — the canonical target-language implementation.
//!
//! This module translates a [`Registry`] of reflected type
//! definitions into idiomatic Kotlin source code. It serves as the reference
//! implementation for adding new target languages.
//!
//! # Submodules (in pipeline order)
//!
//! 1. **generator** — Top-level orchestrator. [`KotlinCodeGenerator`](crate::generation::kotlin::KotlinCodeGenerator) implements
//!    [`CodeGenerator`] to produce a complete Kotlin source file from a
//!    registry. It resolves qualified type names against the configuration
//!    (external packages, namespaces) and then delegates writing to the emitter
//!    layer.
//!
//! 2. **emitter** — AST-to-source rendering. Implements
//!    [`Emitter<Kotlin>`] for each AST node type
//!    ([`Module`](crate::generation::module::Module), [`Container`],
//!    `Named<Format>`, `Format`, `Doc`). This is where the Kotlin language
//!    mapping lives: type names, serialization annotations, `data class` /
//!    `sealed interface` / `enum class` selection, and bincode
//!    serialize/deserialize method generation. Feature helpers (BigInt, ListOfT,
//!    etc.) are embedded as `include_bytes!` snippets and emitted as needed.
//!
//! 3. **installer** — Project scaffolding. [`Installer`](crate::generation::kotlin::Installer) implements
//!    [`SourceInstaller`] to write a
//!    Kotlin project: it copies serde/bincode runtime sources, splits the
//!    registry by namespace into per-module files, and generates a
//!    `build.gradle.kts` manifest.

mod emitter;
mod generator;
mod installer;
pub(crate) mod naming;

pub use emitter::{
    Kotlin, enum_constant_name, escape_identifier, field_name, render_type, requalify,
    requalify_format, variant_class_name,
};
pub use generator::KotlinCodeGenerator;
pub use installer::Installer;
