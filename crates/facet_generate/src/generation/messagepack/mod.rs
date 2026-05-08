//! MessagePack serialization plugin — provides MessagePack-specific imports,
//! annotations, type bodies, and manifest dependencies through the
//! [`EmitterPlugin`](super::plugin::EmitterPlugin) trait.
//!
//! # What the plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports`               | Language-specific MessagePack package imports |
//! | `type_annotations`      | `@Serializable` / `[DerivedTypeShape]` / etc. |
//! | `field_annotations`     | Field-level rename annotations where needed |
//! | `has_type_body`         | `true` for types that need serialize/deserialize helpers |
//! | `type_body`             | `msgPackSerialize` / `msgPackDeserialize` convenience wrappers |
//! | `module_helpers`        | Witness class (C#) or encode/decode helpers (TypeScript) |
//! | `manifest_dependencies` | Package manager dependency entries |
//!
//! # Language-specific variants
//!
//! - **Kotlin** — annotation-driven via `kotlinx-serialization-msgpack`.
//! - **Swift** — `Codable`-driven via `MessagePacker`.
//! - **TypeScript** — structural encoding via `@msgpack/msgpack`.
//! - **C#** — `Nerdbank.MessagePack` with a per-module witness class.

#[cfg(feature = "kotlin")]
pub mod kotlin;

#[cfg(feature = "swift")]
pub mod swift;

#[cfg(feature = "typescript")]
pub mod typescript;

#[cfg(feature = "csharp")]
pub mod csharp;

/// `MessagePack` serialization plugin.
///
/// Add this to a language tag's plugin list to inject `MessagePack`-specific
/// imports, annotations, convenience methods, and manifest dependencies into
/// the generated code.
#[derive(Debug, Clone, Default)]
pub struct MessagePackPlugin;
