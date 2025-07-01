// Copyright (c) Facebook, Inc. and its affiliates
// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Utility function to generate indented text
pub mod indent;

/// Support for code-generation in Java
#[cfg(feature = "java")]
pub mod java;
/// Support for code-generation in Swift
#[cfg(feature = "swift")]
pub mod swift;
/// Support for code-generation in TypeScript
#[cfg(feature = "typescript")]
pub mod typescript;

/// Common logic for codegen.
mod common;
/// Common configuration objects and traits used in public APIs.
mod config;

pub use config::*;
