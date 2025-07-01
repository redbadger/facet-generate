#![allow(unused)]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]

mod error;
mod format;

use std::collections::BTreeMap;

pub use error::{Error, Result};
pub use format::{
    ContainerFormat, Format, FormatHolder, Named, Namespace, QualifiedTypeName, Variable,
    VariantFormat,
};

/// A map of container formats.
pub type Registry = BTreeMap<String, ContainerFormat>;

#[cfg(test)]
mod tests;
