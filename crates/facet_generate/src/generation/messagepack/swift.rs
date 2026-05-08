//! `EmitterPlugin<Swift>` implementation for [`MessagePackPlugin`].
//!
//! Provides MessagePack-specific imports and manifest dependencies for Swift
//! code generation, using the `MessagePacker` SPM package.

use crate::generation::{plugin::EmitterPlugin, swift::Swift};

use super::MessagePackPlugin;

impl EmitterPlugin<Swift> for MessagePackPlugin {}
