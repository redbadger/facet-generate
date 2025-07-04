#![allow(clippy::missing_panics_doc)]

pub mod error;
pub mod generation;
pub mod reflection;

use crate::error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
