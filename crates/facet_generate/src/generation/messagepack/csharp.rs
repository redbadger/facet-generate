//! `EmitterPlugin<CSharp>` implementation for [`MessagePackPlugin`].
//!
//! Provides MessagePack-specific imports, `[DerivedTypeShape]` type
//! annotations, per-module witness class generation, and convenience
//! serialize/deserialize methods for C# code generation, using
//! `Nerdbank.MessagePack`.

use crate::generation::{csharp::CSharp, plugin::EmitterPlugin};

use super::MessagePackPlugin;

impl EmitterPlugin<CSharp> for MessagePackPlugin {}
