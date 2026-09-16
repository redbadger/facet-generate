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

use std::{path::Path, process::Command};

use facet_generate::{
    generation::{bincode::BincodePlugin, json::JsonPlugin, kotlin},
    reflect,
};
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

/// The installer writes the package tree at the project root, but Gradle's
/// Kotlin source set is `src/main/kotlin`, so `gradle build` would find no
/// sources. Move everything except the manifest there.
fn move_sources_into_gradle_source_set(dir: &Path) {
    let source_set = dir.join("src").join("main").join("kotlin");
    std::fs::create_dir_all(&source_set).unwrap();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name == "src" || name == "build.gradle.kts" {
            continue;
        }
        std::fs::rename(entry.path(), source_set.join(&name)).unwrap();
    }
}

/// Pin both the Java and the Kotlin JVM target so the build does not fail with
/// "Inconsistent JVM-target compatibility" on a JDK newer than the one the
/// Kotlin compiler supports.
fn pin_jvm_target(dir: &Path) {
    let manifest = dir.join("build.gradle.kts");
    let mut contents = std::fs::read_to_string(&manifest).unwrap();
    contents.push_str(
        r"
java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

kotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }
}
",
    );
    std::fs::write(&manifest, contents).unwrap();
}

/// Field and variant names that collide with Kotlin hard keywords must be
/// escaped with backticks; soft keywords (`import`, `value`, `field0`) must
/// not be.
#[test]
fn test_that_kotlin_code_with_keyword_names_compiles() {
    for encoding in ["bincode", "json"] {
        let registry = common::get_keyword_registry();
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("testing");

        let installer = kotlin::Installer::new("com.example.testing", &dir);
        let installer = if encoding == "bincode" {
            installer.plugin(BincodePlugin)
        } else {
            installer.plugin(JsonPlugin)
        };
        installer.generate(&registry).unwrap();
        move_sources_into_gradle_source_set(&dir);
        pin_jvm_target(&dir);

        let status = gradle_command()
            .args(["--configuration-cache", "build"])
            .current_dir(&dir)
            .status()
            .unwrap();
        assert!(status.success(), "gradle build failed for {encoding}");
    }
}
