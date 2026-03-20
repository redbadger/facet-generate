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
//! | `imports` | `kotlinx.serialization.*` imports, BigInt JSON imports |
//! | `module_helpers` | BigInt JSON helper snippet (`BigInt.kt`) |
//! | `type_annotations` | `@Serializable`, `@SerialName("…")` above each type |
//! | `type_body` | `val serialName` accessor in all-unit enum classes |
//!
//! # What still lives in the per-language emitters
//!
//! Variant-level inline `@SerialName("…")` annotations in enum classes are
//! emitted by the Kotlin emitter directly because they appear *inline* with
//! the variant name rather than on a separate line — a pattern that doesn't
//! map cleanly to the current `type_annotations` hook.

#[cfg(feature = "kotlin")]
pub(crate) mod kotlin;

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
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonPlugin {
    fn default() -> Self {
        Self::new()
    }
}
