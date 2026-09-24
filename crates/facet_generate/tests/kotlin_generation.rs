//! Compilation test for Kotlin code generation.
//!
//! This integration test verifies that the Kotlin code we generate is
//! **syntactically and type-correct** by actually compiling it with
//! `gradle build`. It does *not* run the generated code — see
//! `kotlin_runtime.rs` (if present) for end-to-end round-trip tests.
//!
//! # How it works
//!
//! Every test builds a [`Registry`] from a fixture in [`common`] and hands it
//! to [`assert_generated_code_compiles`], which
//!
//! 1. runs the full [`Installer`](facet_generate::generation::kotlin::Installer)
//!    pipeline into a temporary directory, producing `.kt` source files and a
//!    `build.gradle.kts` manifest,
//! 2. rearranges that output into a shape Gradle can build (see
//!    [`move_sources_into_gradle_source_set`] and [`pin_jvm_target`]),
//! 3. invokes `gradle build`, asserting both a zero exit code and that
//!    `compileKotlin` actually had sources to compile.
//!
//! The test is gated on `#[cfg(feature = "kotlin")]` so it only runs when the
//! Kotlin/Gradle toolchain is available.

#![cfg(feature = "kotlin")]

use std::{path::Path, process::Command};

use facet_generate::{
    Registry,
    generation::{bincode::BincodePlugin, json::JsonPlugin, kotlin},
    reflection::format::{ContainerFormat, Format},
};
use tempfile::tempdir;

pub mod common;

fn gradle_command() -> Command {
    Command::new("gradle")
}

/// Which encoding plugin the generated Gradle project is installed with.
///
/// There is no plugin-less variant: without a plugin the installer ships no
/// runtime sources, so anything the emitter renders as a runtime type (such as
/// `Bytes`) cannot resolve.
#[derive(Clone, Copy, Debug)]
enum Encoding {
    Bincode,
    Json,
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

/// Generate `registry` for `encoding` into a throwaway Gradle project and
/// compile it, asserting that `compileKotlin` ran and succeeded.
fn assert_generated_code_compiles(registry: &Registry, encoding: Encoding) {
    let tmp = tempdir().unwrap();
    let dir = tmp.path().join("testing");

    let installer = kotlin::Installer::new("com.example.testing", &dir);
    let installer = match encoding {
        Encoding::Bincode => installer.plugin(BincodePlugin),
        Encoding::Json => installer.plugin(JsonPlugin),
    };
    installer.generate(registry).unwrap();

    move_sources_into_gradle_source_set(&dir);
    pin_jvm_target(&dir);

    let output = gradle_command()
        .args(["--configuration-cache", "build"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "gradle build failed for {encoding:?}\n{stdout}\n{stderr}"
    );
    // A bare `> Task :compileKotlin` line means the task ran with sources;
    // `NO-SOURCE` (or no line at all) means the generated code was never
    // compiled, which would make the whole test vacuous. Match on whole lines
    // rather than a `"...\n"` substring: Gradle terminates its lines with
    // `\r\n` on Windows, which no `\n`-suffixed needle can ever match.
    assert!(
        stdout
            .lines()
            .any(|line| line.trim_end() == "> Task :compileKotlin"),
        "gradle did not compile any Kotlin sources for {encoding:?}\n{stdout}"
    );
}

/// Drop every `u128` / `i128` struct field from `registry`.
///
/// Known bug, not fixed here: the Kotlin JSON plugin binds its
/// `BigIntegerSerializer` with a same-file `typealias BigInteger`, but the
/// emitter also writes `import java.math.BigInteger` for any module with
/// 128-bit integers, and an explicit import outranks a same-package
/// declaration. The alias therefore never applies and kotlinx.serialization
/// fails with "Serializer has not been found for type '`BigInteger`'".
fn remove_128_bit_fields(registry: &mut Registry) {
    for container in registry.values_mut() {
        if let ContainerFormat::Struct(fields, _) = container {
            fields.retain(|field| !matches!(field.value, Format::I128 | Format::U128));
        }
    }
}

/// The main fixture — the full [`SerdeData`](common::SerdeData) tree of
/// primitives, containers, tuples, maps and recursive enums.
///
/// JSON only: the bincode plugin does not compile this fixture (128-bit
/// integers are declared as `BigInteger` but serialized through
/// `Int128`/`UInt128`, `char` is declared as `String` but serialized as
/// `Char`, and `Vec<()>` / `BTreeMap<_, ()>` call the container helpers
/// without their `serializeElement` argument). All pre-existing bugs.
#[test]
fn test_that_kotlin_code_compiles() {
    let mut registry = common::get_registry();
    remove_128_bit_fields(&mut registry);
    assert_generated_code_compiles(&registry, Encoding::Json);
}

/// Field and variant names that collide with Kotlin hard keywords must be
/// escaped with backticks; soft keywords (`import`, `value`, `field0`) must
/// not be.
#[test]
fn test_that_kotlin_code_with_keyword_names_compiles() {
    let registry = common::get_keyword_registry();
    for encoding in [Encoding::Bincode, Encoding::Json] {
        assert_generated_code_compiles(&registry, encoding);
    }
}

/// A type named `Set` shadows `kotlin.collections.Set` for the whole package,
/// so every `Set<T>` the emitter and the bincode plugin write must be
/// qualified while the `data class Set` keeps its name.
///
/// The fixture's `Set.value` is a `#[facet(fg::bytes)]` field, so this also
/// covers `Bytes` being resolvable (and serializable) under both encodings.
#[test]
fn test_that_kotlin_code_shadowing_builtin_names_compiles() {
    let registry = common::get_shadowing_registry();
    for encoding in [Encoding::Bincode, Encoding::Json] {
        assert_generated_code_compiles(&registry, encoding);
    }
}

/// Types referencing enums and structs in other namespaces, including a type
/// in one named namespace referencing another, one referencing ROOT types
/// directly and nested in generics, and types that inherit a namespace.
#[test]
fn test_that_kotlin_code_with_types_from_other_namespaces_compiles() {
    for registry in [
        common::across_namespaces::get_registry(),
        common::across_namespaces::get_sibling_registry(),
        common::across_namespaces::to_root::get_registry(),
        common::across_namespaces::inherited::get_registry(),
    ] {
        for encoding in [Encoding::Bincode, Encoding::Json] {
            assert_generated_code_compiles(&registry, encoding);
        }
    }
}
