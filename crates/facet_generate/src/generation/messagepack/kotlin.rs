//! `EmitterPlugin<Kotlin>` implementation for [`MessagePackPlugin`].
//!
//! Provides MessagePack-specific imports, `@Serializable` / `@SerialName`
//! type annotations, and manifest dependencies for Kotlin code generation,
//! using `kotlinx-serialization-msgpack`.

use crate::generation::{kotlin::Kotlin, plugin::EmitterPlugin};

use super::MessagePackPlugin;

impl EmitterPlugin<Kotlin> for MessagePackPlugin {}
