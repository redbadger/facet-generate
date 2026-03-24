//! Internal bincode plugin — provides bincode-specific imports and module
//! helpers through the [`EmitterPlugin`] trait.
//!
//! This module is part of the ongoing effort to extract all encoding-specific
//! code-generation into separate plugin crates. For now it lives inside the
//! core crate so it can share types and feature constants.
//!
//! # What the plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | Serde / bincode package imports |
//! | `module_helpers` | Feature helper snippets (`ListOfT`, `SetOfT`, …) |
//! | `has_type_body` | Always `true` |
//! | `type_body` | `serialize` / `deserialize` methods + `bincodeSerialize` / `bincodeDeserialize` wrappers |

#[cfg(feature = "kotlin")]
pub mod kotlin;

#[cfg(feature = "swift")]
pub mod swift;

#[cfg(feature = "typescript")]
pub mod typescript;

use super::CodeGeneratorConfig;

/// Bincode serialization plugin.
///
/// When added to a language tag's plugin list, it provides the bincode-
/// and serde-related imports, feature helper snippets, runtime files, and
/// manifest dependencies for that language.
///
/// Each target language has its own `impl EmitterPlugin<Lang>` in a
/// submodule (e.g. [`kotlin`], [`swift`]).
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
