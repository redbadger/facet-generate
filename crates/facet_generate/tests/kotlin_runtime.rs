#![cfg(feature = "kotlin")]
//! Runtime tests for Kotlin bincode serialization.
//!
//! These tests generate Kotlin code, serialize data in Rust with bincode, then
//! compile and run the generated Kotlin code to deserialize, verify field
//! values, and re-serialize — checking that the bytes round-trip correctly.
//!
//! # Toolchain requirement
//!
//! `kotlinc` and `java` must be on `PATH`. The test compiles all generated
//! `.kt` sources (including the serde runtime) into a single JAR with
//! `kotlinc -include-runtime`, then runs the JVM entry-point with
//! `java -classpath`.
//!
//! Unlike the compilation-only test in `kotlin_generation.rs`, this test
//! actually *executes* the generated serialization logic and verifies the
//! bytes produced by the Kotlin code match what Rust's `bincode` produces
//! for the same value.

use std::{
    fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
    process::Command,
};

use facet_generate::generation::{bincode::BincodePlugin, kotlin};
use tempfile::tempdir;

pub mod common;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect all `.kt` files under `dir`, recursively.
fn collect_kt_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(collect_kt_files(&path));
            } else if path.extension().is_some_and(|e| e == "kt") {
                out.push(path);
            }
        }
    }
    out
}

/// Format a `&[u8]` as a Kotlin `byteArrayOf(…)` literal.
///
/// Kotlin `Byte` is signed, so values > 127 must be cast to their signed
/// equivalents (e.g. 255u8 → -1i8).
fn quote_bytes_kotlin(bytes: &[u8]) -> String {
    let elems: Vec<String> = bytes.iter().map(|&b| b.cast_signed().to_string()).collect();
    format!("byteArrayOf({})", elems.join(", "))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_kotlin_bincode_runtime_on_uuid_data() {
    // Skip gracefully when kotlinc is not on PATH (e.g. Windows CI runners
    // that have Gradle but not the standalone kotlinc compiler).
    match Command::new("kotlinc").arg("-version").output() {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("kotlinc not found on PATH — skipping runtime test");
            return;
        }
        Err(e) => panic!("failed to probe kotlinc: {e}"),
        Ok(_) => {}
    }

    let registry = common::get_uuid_registry();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    // Generate the Kotlin source + serde/bincode runtime files.
    kotlin::Installer::new("com.example.testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let reference = common::get_uuid_reference_bytes();
    let id_str = common::UUID_ID.to_string();
    let parent_id_str = common::UUID_PARENT_ID.to_string();

    // Write a top-level Main.kt (default package → JVM class `MainKt`).
    let main_path = dir.join("Main.kt");
    let mut main_file = fs::File::create(&main_path).unwrap();
    writeln!(
        main_file,
        r#"import com.example.testing.UuidData
import java.util.UUID

fun main() {{
    val input = {bytes}
    val value = UuidData.bincodeDeserialize(input)

    check(value.id == UUID.fromString("{id}")) {{
        "id mismatch: ${{value.id}}"
    }}
    check(value.parentId == UUID.fromString("{parent_id}")) {{
        "parentId mismatch: ${{value.parentId}}"
    }}

    val output = value.bincodeSerialize()
    check(input.contentEquals(output)) {{
        "roundtrip failed:\n  input  = ${{input.toList()}}\n  output = ${{output.toList()}}"
    }}

    println("UUID roundtrip: PASSED")
}}
"#,
        bytes = quote_bytes_kotlin(&reference),
        id = id_str,
        parent_id = parent_id_str,
    )
    .unwrap();

    // Compile all .kt files (generated types + serde runtime + Main.kt) into
    // a single self-contained JAR.
    let jar_path = dir.join("test.jar");
    let kt_files = collect_kt_files(&dir);

    let status = Command::new("kotlinc")
        .args(&kt_files)
        .arg("-include-runtime")
        .arg("-d")
        .arg(&jar_path)
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success(), "kotlinc compilation failed");

    // Run the JVM entry-point. `Main.kt` in the default package compiles to
    // the JVM class `MainKt`.
    let status = Command::new("java")
        .arg("-classpath")
        .arg(&jar_path)
        .arg("MainKt")
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success(), "UUID round-trip test failed");
}
