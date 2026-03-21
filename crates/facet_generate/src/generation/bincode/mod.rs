//! Internal bincode plugin — provides bincode-specific imports and module
//! helpers through the [`EmitterPlugin`] trait.
//!
//! This module is the first step toward extracting all bincode code-generation
//! into a separate crate (`facet-generate-bincode`). For now it lives inside
//! the core crate so it can share types and feature constants.
//!
//! # What the plugin handles today
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | Serde / bincode package imports |
//! | `module_helpers` | Feature helper snippets (`ListOfT.kt`, `SetOfT.kt`, …) |
//!
//! # What still lives in the per-language emitters
//!
//! Type-body generation (serialize / deserialize methods, companion objects)
//! remains in the language emitters because it relies heavily on
//! [`IndentWrite::block`] which requires `Self: Sized` and therefore cannot
//! be called through the `&mut dyn IndentWrite` that plugin methods receive.
//! Migrating this code is planned for a future phase.

#[cfg(feature = "kotlin")]
pub mod kotlin;

use super::CodeGeneratorConfig;

/// Bincode serialization plugin.
///
/// When added to a language tag's plugin list, it provides the bincode-
/// and serde-related imports, feature helper snippets, runtime files, and
/// manifest dependencies for that language.
///
/// Each target language has its own `impl EmitterPlugin<Lang>` in a
/// submodule (e.g. [`kotlin`]).
#[derive(Debug, Clone)]
pub struct BincodePlugin {
    /// Resolved serde package name (e.g. `"com.novi.serde"`).
    pub(crate) serde_package: String,
    /// Resolved bincode package name (e.g. `"com.novi.bincode"`).
    pub(crate) bincode_package: String,
}

impl BincodePlugin {
    /// Create a new `BincodePlugin` with package names resolved from the
    /// given config (specifically `config.external_packages`).
    #[must_use]
    pub fn from_config(config: &CodeGeneratorConfig) -> Self {
        let serde_package = resolve_package(config, super::SERDE_NAMESPACE, "com.novi.serde");
        let bincode_package = resolve_package(config, super::BINCODE_NAMESPACE, "com.novi.bincode");
        Self {
            serde_package,
            bincode_package,
        }
    }
}

/// Look up the package path for `namespace` in the config's external
/// packages. Falls back to `default` when no override is configured.
fn resolve_package(config: &CodeGeneratorConfig, namespace: &str, default: &str) -> String {
    config
        .external_packages
        .get(namespace)
        .and_then(|pkg| {
            if let super::PackageLocation::Path(path) = &pkg.location {
                Some(path.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| default.to_string())
}
