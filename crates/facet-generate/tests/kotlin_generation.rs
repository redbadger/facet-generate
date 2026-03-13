//! Compilation test for Kotlin code generation.
//!
//! This integration test verifies that the Kotlin code we generate is
//! **syntactically and type-correct** by actually compiling it with
//! `gradle build`. It does *not* run the generated code — see
//! `kotlin_runtime.rs` (if present) for end-to-end round-trip tests.
//!
//! # How it works
//!
//! 1. Reflects [`PrimitiveTypes`](common::PrimitiveTypes) into a [`Registry`].
//! 2. Runs the full [`Installer`](facet_generate::generation::kotlin::Installer)
//!    pipeline into a temporary directory, producing `.kt` source files and a
//!    `build.gradle.kts` manifest.
//! 3. Invokes `gradle --version` as a smoke check for the toolchain.
//! 4. Invokes `gradle build` and asserts a zero exit code.
//!
//! The test is gated on `#[cfg(feature = "kotlin")]` so it only runs when the
//! Kotlin/Gradle toolchain is available.

#![cfg(feature = "kotlin")]

use std::process::Command;

use facet_generate::{generation::kotlin, reflect};
use tempfile::tempdir;

pub mod common;

fn gradle_command() -> Command {
    Command::new("gradle")
}

#[test]
fn test_that_kotlin_code_compiles() {
    type Test = common::PrimitiveTypes;

    let registry = reflect!(Test).unwrap();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    let package_name = "com.example.testing";

    kotlin::Installer::new(package_name, &dir)
        .generate(&registry)
        .unwrap();

    let args = ["--configuration-cache"];

    let status = gradle_command()
        .args(args)
        .arg("--version")
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success());

    let status = gradle_command()
        .args(args)
        .arg("build")
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success());
}
