// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(clippy::missing_errors_doc)]
#![allow(dead_code)]

mod emitter;
mod generator;
mod installer;
mod package;

pub use generator::CodeGenerator;
pub use installer::Installer;
