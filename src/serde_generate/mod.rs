// Copyright (c) Facebook, Inc. and its affiliates
// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! This crate aims to compile the data formats extracted from Rust by [`serde-reflection`](https://crates.io/crates/serde-reflection)
//! into type definitions and (de)serialization methods for other programming languages.
//!
//! It can be used as a library or as a command-line tool (see `serdegen` below).
//!
//! ## Supported Languages
//!
//! The following programming languages are fully supported as target languages:
//!
//! * Java 8
//! * Swift 5.3
//!
//! The following languages are partially supported and/or still considered under development:
//!
//! * TypeScript 4 (packaged and tested with Deno) [(follow-up issue)](https://github.com/zefchain/serde-reflection/issues/58)
//!
//! ## Supported Encodings
//!
//! Type definitions in a target language are meant to be used together with a runtime library that
//! provides (de)serialization in a particular [Serde encoding format](https://serde.rs/#data-formats).
//!
//! This crate provides easy-to-deploy runtime libraries for the following binary formats, in all supported languages:
//!
//! * [Bincode](https://docs.rs/bincode/1.3.1/bincode/) (default configuration only),
//! * [BCS](https://github.com/diem/bcs) (short for Binary Canonical Serialization, the main format used
//!   in the [Diem blockchain](https://github.com/diem/diem)).
//!
//! ## Binary Tool
//!
//! In addition to a Rust library, this crate provides a binary tool `serdegen` to process Serde formats
//! saved on disk.
//!
//! The tool `serdegen` assumes that a Rust value of type `serde_reflection::Registry` has
//! been serialized into a YAML file. The recommended way to generate such a value is to
//! use the library `serde-reflection` to introspect Rust definitions (see also the
//! example above).
//!
//! For a quick test, one may create a test file like this:
//! ```bash
//! cat >test.yaml <<EOF
//! ---
//! Foo:
//!   ENUM:
//!     0:
//!       A:
//!         NEWTYPE:
//!           U64
//!     1:
//!       B: UNIT
//! EOF
//! ```
//!
//! Then, the following command will generate Python class definitions and write them into `test.py`:
//! ```bash
//! cargo run -p serde-generate-bin -- --language python3 test.yaml > test.py
//! ```
//!
//! To create a python module `test` and install the bincode runtime in a directory `$DEST`, you may run:
//! ```bash
//! cargo run -p serde-generate-bin -- --language python3 --with-runtimes serde bincode --module-name test --target-source-dir "$DEST" test.yaml
//! ```
//!
//! See the help message of the tool with `--help` for more options.
//!
//! Note: Outside of this repository, you may install the tool with `cargo install serde-generate-bin` then use `$HOME/.cargo/bin/serdegen`.

/// Dependency analysis and topological sort for Serde formats.
pub mod analyzer;
/// Utility function to generate indented text
pub mod indent;

/// Support for code-generation in Java
#[cfg(feature = "java")]
pub mod java;
/// Support for code-generation in Swift
#[cfg(feature = "swift")]
pub mod swift;
/// Support for code-generation in TypeScript/JavaScript
#[cfg(feature = "typescript")]
pub mod typescript;

/// Common logic for codegen.
mod common;
/// Common configuration objects and traits used in public APIs.
mod config;

pub use config::*;
