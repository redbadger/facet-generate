//! Internal bincode plugin — provides bincode-specific imports and module
//! helpers through the `EmitterPlugin` trait.
//!
//! This module is part of the ongoing effort to extract all encoding-specific
//! code-generation into separate plugin crates. For now it lives inside the
//! core crate so it can share types and feature constants.
//!
//! # What the plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | Language-specific bincode package imports |
//! | `module_helpers` | Feature helper snippets (`ListOfT`, `SetOfT`, …) |
//! | `has_type_body` | Always `true` |
//! | `type_body` | `serialize` / `deserialize` methods + wrappers |
//!
//! # Language-specific variants
//!
//! - **Kotlin** — `kotlin::KotlinBincodePlugin` carries the resolved JVM
//!   package names (`serde_package`, `bincode_package`) needed for import
//!   generation.
//! - **Swift**, **TypeScript** — use `BincodePlugin` directly (no
//!   language-specific fields required).
//! - **C#** — `csharp::CSharpBincodePlugin` carries the precomputed set of
//!   C-style enum names needed for correct serialization dispatch.

#[cfg(feature = "kotlin")]
pub mod kotlin;

#[cfg(feature = "swift")]
pub mod swift;

#[cfg(feature = "typescript")]
pub mod typescript;

#[cfg(feature = "csharp")]
pub mod csharp;

/// Bincode serialization plugin.
///
/// A lightweight, language-agnostic plugin token. Languages that need
/// extra configuration (e.g. JVM package names for Kotlin, C-style enum
/// names for C#) use their own language-specific plugin structs defined in
/// the corresponding submodule.
///
/// Swift and TypeScript use this struct directly — neither needs any
/// additional configuration beyond the encoding flag carried by their
/// language tag.
#[derive(Debug, Clone, Default)]
pub struct BincodePlugin;
