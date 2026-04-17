//! Internal bincode plugin — provides bincode-specific imports and module
//! helpers through the `EmitterPlugin` trait.
//!
//! This module lives inside the core crate so it can share types and feature
//! constants with the language emitters.
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
//! - **Kotlin**, **Swift**, **TypeScript**, **C#** — use `BincodePlugin`
//!   directly (no language-specific fields required).

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
/// A lightweight, language-agnostic plugin token. All languages currently
/// use this struct directly — no language-specific fields are required.
#[derive(Debug, Clone, Default)]
pub struct BincodePlugin;
