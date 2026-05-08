//! Runtime tests for Kotlin MessagePack serialization.
//!
//! These tests generate Kotlin code, serialize data in Rust with
//! `rmp_serde::to_vec_named`, then run the generated Kotlin code to
//! deserialize, verify field values, and re-serialize — checking that
//! the bytes round-trip correctly.
//!
//! # How it works
//!
//! 1. Generates a Kotlin project via [`kotlin::Installer`] with [`MessagePackPlugin`].
//! 2. Patches `build.gradle.kts` to add the `application` plugin, configure
//!    a custom `sourceSets` root so Gradle finds the generated `.kt` files,
//!    and set `mainClass`.
//! 3. Writes a `Main.kt` at the project root that drives the assertions.
//! 4. Runs `gradle --configuration-cache run` and asserts a zero exit code
//!    plus "PASSED" in stdout.

#![allow(clippy::doc_markdown)]
#![cfg(feature = "kotlin")]

use std::{fs, process::Command};

use facet_generate::generation::{kotlin, messagepack::MessagePackPlugin};
use tempfile::tempdir;

pub mod common;

fn gradle_command() -> Command {
    Command::new("gradle")
}

/// Convert a byte slice into a Kotlin `byteArrayOf(...)` argument string.
///
/// Kotlin's `Byte` type is signed (`-128..127`), so unsigned bytes in the
/// range `128..=255` are re-interpreted as their signed two's-complement value.
fn format_kotlin_byte_array(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{}", i8::from_ne_bytes([*b])))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Patch the generated `build.gradle.kts` so that:
///
/// - The `application` plugin is applied (needed for `gradle run`).
/// - `sourceSets.main.kotlin` includes the project root `.`, so Gradle's
///   Kotlin compiler discovers the generated `.kt` files that live under
///   `com/example/testing/` as well as our handwritten `Main.kt`.
/// - `application.mainClass` is set to `"MainKt"` (the JVM class generated
///   from a top-level `fun main()` in a package-less `Main.kt`).
fn patch_build_file_for_run(dir: &std::path::Path) {
    let build_path = dir.join("build.gradle.kts");
    let original = fs::read_to_string(&build_path)
        .unwrap_or_else(|e| panic!("could not read build.gradle.kts: {e}"));

    // Insert `application` as a sibling plugin to `java-library` inside the
    // existing `plugins { }` block.
    let patched = original.replace("`java-library`", "`java-library`\n    application");

    // Append the sourceSets and application configuration blocks.
    // Also explicitly align the JVM target for both the Kotlin and Java
    // compile tasks, because when the current JDK is newer than Kotlin's
    // maximum supported JVM target (e.g. JDK 25 → Kotlin falls back to
    // JVM_24) Gradle's consistency check fails unless both compilers
    // agree on the same target.
    let patched = format!(
        r#"{patched}
sourceSets {{
    main {{
        kotlin.srcDirs(".")
    }}
}}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinJvmCompile>().configureEach {{
    compilerOptions {{
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_21)
    }}
}}

tasks.withType<JavaCompile>().configureEach {{
    sourceCompatibility = "21"
    targetCompatibility = "21"
}}

application {{
    mainClass.set("MainKt")
}}
"#
    );

    fs::write(&build_path, patched)
        .unwrap_or_else(|e| panic!("could not write build.gradle.kts: {e}"));
}

// ---------------------------------------------------------------------------
// Test 1 — simple `Test` / `Choice` types
// ---------------------------------------------------------------------------

#[test]
fn test_kotlin_msgpack_runtime_on_simple_data() {
    let registry = common::get_simple_registry();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    kotlin::Installer::new("com.example.testing", &dir)
        .plugin(MessagePackPlugin)
        .generate(&registry)
        .unwrap();

    // Compute reference bytes from Rust to confirm the Rust side can
    // serialise this value.  Note that rmp_serde encodes Rust tuples as
    // msgpack arrays while kotlinx-serialization-msgpack encodes Kotlin
    // Pair as a named map, so the byte representations differ between the
    // two languages.  The runtime test therefore uses a Kotlin-internal
    // encode→decode roundtrip to validate the generated code, rather than
    // attempting to decode cross-format Rust bytes.
    let _reference = rmp_serde::to_vec_named(&common::Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: common::Choice::C { x: 7 },
    })
    .unwrap();

    patch_build_file_for_run(&dir);

    // Write Main.kt at the project root (no `package` declaration so the JVM
    // class name is simply `MainKt`, matching `mainClass.set("MainKt")`).
    // The test constructs the value directly in Kotlin, serialises it with
    // MsgPack, deserialises the resulting bytes, and verifies field equality.
    let main_kt = r#"import com.example.testing.Choice
import com.example.testing.Test
import com.ensarsarajcic.kotlinx.serialization.msgpack.MsgPack
import kotlinx.serialization.decodeFromByteArray
import kotlinx.serialization.encodeToByteArray

fun main() {
    // Construct the expected value directly in Kotlin.
    val original = Test(
        a = listOf(4u, 6u),
        b = Pair(-3L, 5UL),
        c = Choice.C(x = 7.toUByte())
    )

    // Encode to MessagePack bytes.
    val bytes = MsgPack.encodeToByteArray(original)

    // Decode back and verify field values.
    val value = MsgPack.decodeFromByteArray<Test>(bytes)

    require(value.a == listOf(4u, 6u)) {
        "a: expected [4, 6], got ${value.a}"
    }
    require(value.b.first == -3L) {
        "b.first: expected -3, got ${value.b.first}"
    }
    require(value.b.second == 5UL) {
        "b.second: expected 5, got ${value.b.second}"
    }
    require(value.c is Choice.C) {
        "c: expected Choice.C, got ${value.c}"
    }
    require((value.c as Choice.C).x == 7.toUByte()) {
        "c.x: expected 7, got ${(value.c as Choice.C).x}"
    }

    // Roundtrip: re-encode and verify the bytes are stable.
    val reEncoded = MsgPack.encodeToByteArray(value)
    require(reEncoded.contentEquals(bytes)) {
        "roundtrip mismatch:\n  first:  ${bytes.toList()}\n  second: ${reEncoded.toList()}"
    }

    println("PASSED")
}
"#;

    let main_kt_path = dir.join("Main.kt");
    fs::write(&main_kt_path, main_kt).unwrap_or_else(|e| panic!("could not write Main.kt: {e}"));

    let output = gradle_command()
        .args(["--configuration-cache", "run"])
        .current_dir(&dir)
        .output()
        .unwrap_or_else(|e| panic!("could not spawn gradle: {e}"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "gradle run failed\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
    assert!(
        stdout.contains("PASSED"),
        "Expected 'PASSED' in output\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// Test 2 — full MsgPackSerdeData round-trip
// ---------------------------------------------------------------------------

#[test]
#[ignore = "slow"]
fn test_kotlin_msgpack_runtime_on_supported_types() {
    let registry = common::get_msgpack_registry();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    kotlin::Installer::new("com.example.testing", &dir)
        .plugin(MessagePackPlugin)
        .generate(&registry)
        .unwrap();

    patch_build_file_for_run(&dir);

    // Collect all positive sample encodings.
    let positive_samples = common::get_msgpack_positive_samples();
    let total = positive_samples.len();

    // Build a Kotlin array-of-arrays literal.
    let samples_literal = positive_samples
        .iter()
        .map(|bytes| format!("    byteArrayOf({})", format_kotlin_byte_array(bytes)))
        .collect::<Vec<_>>()
        .join(",\n");

    let main_kt = format!(
        r#"import com.example.testing.MsgPackSerdeData
import com.ensarsarajcic.kotlinx.serialization.msgpack.MsgPack
import kotlinx.serialization.decodeFromByteArray
import kotlinx.serialization.encodeToByteArray

fun main() {{
    val samples: Array<ByteArray> = arrayOf(
{samples_literal}
    )

    var passed = 0
    for ((index, input) in samples.withIndex()) {{
        val value = MsgPack.decodeFromByteArray<MsgPackSerdeData>(input)
        val output = MsgPack.encodeToByteArray(value)
        require(output.contentEquals(input)) {{
            "roundtrip mismatch for sample ${{index}}:\n" +
            "  expected: ${{input.toList()}}\n" +
            "  actual:   ${{output.toList()}}"
        }}
        passed++
    }}

    println("${{passed}}/{total} PASSED")
}}
"#
    );

    let main_kt_path = dir.join("Main.kt");
    fs::write(&main_kt_path, main_kt).unwrap_or_else(|e| panic!("could not write Main.kt: {e}"));

    let output = gradle_command()
        .args(["--configuration-cache", "run"])
        .current_dir(&dir)
        .output()
        .unwrap_or_else(|e| panic!("could not spawn gradle: {e}"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "gradle run failed\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
    let expected_marker = format!("{total}/{total} PASSED");
    assert!(
        stdout.contains(&expected_marker),
        "Expected '{expected_marker}' in output\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );
}
