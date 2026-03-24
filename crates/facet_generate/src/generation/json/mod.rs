//! Internal JSON plugin — provides JSON-specific imports, type annotations,
//! and module helpers through the [`EmitterPlugin`] trait.
//!
//! This module is the counterpart to [`super::bincode`] and is the second
//! step toward extracting all encoding-specific code-generation into separate
//! plugin crates. For now it lives inside the core crate so it can share
//! types and feature constants.
//!
//! # What the plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `kotlinx.serialization.*` imports (Kotlin), `import Serde` (Swift) |
//! | `module_helpers` | BigInt JSON helper (Kotlin); feature snippets (Swift) |
//! | `type_annotations` | `@Serializable`, `@SerialName("…")` above each type (Kotlin) |
//! | `type_body` | `val serialName` accessor for enum classes (Kotlin); `serialize` / `deserialize` + `jsonSerialize` / `jsonDeserialize` wrappers (Swift) |
//! | `has_type_body` | Always `true` (Swift) |
//!

#[cfg(feature = "kotlin")]
pub mod kotlin;

#[cfg(feature = "swift")]
pub mod swift;

#[cfg(feature = "typescript")]
pub mod typescript;

/// JSON serialization plugin.
///
/// When added to a language tag's plugin list, it provides the
/// JSON-related imports, annotations, feature helpers, and manifest
/// dependencies for that language.
///
/// Each target language has its own `impl EmitterPlugin<Lang>` in a
/// submodule (e.g. [`kotlin`]).
#[derive(Debug, Clone)]
pub struct JsonPlugin;

impl JsonPlugin {
    /// Create a new `JsonPlugin`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for JsonPlugin {
    fn default() -> Self {
        Self::new()
    }
}
