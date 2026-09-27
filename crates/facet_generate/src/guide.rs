//! Guides to using `facet_generate`. These modules hold documentation only.
#![doc = include_str!("../docs/guide/index.md")]
// These pages are prose that names other projects (UniFFI, typeshare, …), which
// `doc_markdown` would have us put in backticks as if they were code.
#![allow(clippy::doc_markdown)]

#[doc = include_str!("../docs/guide/motivation.md")]
pub mod motivation {}

#[doc = include_str!("../docs/guide/supported_types.md")]
pub mod supported_types {}

#[doc = include_str!("../docs/guide/serialization.md")]
pub mod serialization {}

#[doc = include_str!("../docs/guide/swift.md")]
pub mod swift {}

#[doc = include_str!("../docs/guide/kotlin.md")]
pub mod kotlin {}

#[doc = include_str!("../docs/guide/csharp.md")]
pub mod csharp {}

#[doc = include_str!("../docs/guide/typescript.md")]
pub mod typescript {}

#[doc = include_str!("../docs/guide/plugins.md")]
pub mod plugins {}

#[doc = include_str!("../docs/guide/contributing.md")]
pub mod contributing {}
