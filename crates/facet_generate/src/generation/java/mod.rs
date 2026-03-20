#![allow(clippy::missing_errors_doc)]
#![expect(
    deprecated,
    reason = "internal implementation of the deprecated Java generator"
)]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

mod emitter;
mod generator;
mod installer;

pub use generator::JavaCodeGenerator;
pub use installer::Installer;
