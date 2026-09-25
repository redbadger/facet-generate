//! Internal JSON plugin — provides JSON-specific imports, type annotations,
//! and module helpers through the `EmitterPlugin` trait.
//!
//! This module is the counterpart to `super::bincode`. It lives inside the
//! core crate so it can share types and feature constants.
//!
//! # What the plugin handles
//!
//! | Extension point | What it provides |
//! |---|---|
//! | `imports` | `kotlinx.serialization.*` and runtime serializer imports (Kotlin), `import Serde` (Swift) |
//! | `module_helpers` | `Bytes`, `UUID` and `BigInteger` JSON serializers (Kotlin) |
//! | `type_annotations` | `@Serializable`, naming the type's own serializer where it has one (Kotlin) |
//! | `field_annotations` | `@SerialName("…")` with the wire name (Kotlin) |
//! | `type_conformances` | `Codable` (Swift) |
//! | `type_body` | `val serialName` accessor for enum classes, and a nested `JsonSerializer` where needed (Kotlin); `CodingKeys`, `init(from:)` / `encode(to:)` where needed, and `jsonSerialize` / `jsonDeserialize` wrappers (Swift) |
//! | `has_type_body` | Where the type has a `JsonSerializer` (Kotlin); always `true` (Swift) |
//!

#[cfg(feature = "kotlin")]
pub mod kotlin;

#[cfg(feature = "swift")]
pub mod swift;

#[cfg(feature = "typescript")]
pub mod typescript;

#[cfg(feature = "csharp")]
pub mod csharp;

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
