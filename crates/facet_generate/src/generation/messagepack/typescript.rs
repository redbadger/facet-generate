//! `EmitterPlugin<TypeScript>` implementation for [`MessagePackPlugin`].
//!
//! Provides module-level encode/decode helpers and manifest dependencies for
//! TypeScript code generation, using `@msgpack/msgpack`.

use crate::generation::{plugin::EmitterPlugin, typescript::TypeScript};

use super::MessagePackPlugin;

impl EmitterPlugin<TypeScript> for MessagePackPlugin {}
