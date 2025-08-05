#![allow(clippy::missing_panics_doc)]

pub mod error;
pub mod generation;
pub mod reflection;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::{
    error::Error,
    reflection::format::{ContainerFormat, QualifiedTypeName},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub type Registry = BTreeMap<QualifiedTypeName, ContainerFormat>;
