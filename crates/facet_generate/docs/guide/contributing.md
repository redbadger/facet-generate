# Contributing: how the tests are organised

Tests are organized in four layers, from fast and narrow to slow and broad:

## Unit tests (snapshot)

Each language has snapshot-based tests that assert on generated **text** without touching the
filesystem.

| Layer | Location | What it covers |
|-------|----------|----------------|
| Emitter | `generation/<lang>/emitter/tests.rs` (+ `tests_bincode.rs`, `tests_json.rs`) | Output for individual types — no file headers, no imports. Uses the `emit` macro. |
| Generator | `generation/<lang>/generator/tests.rs` | Full file output including package declarations, imports, and namespace-qualified names. |
| Installer | `generation/<lang>/installer/tests.rs` | Generated manifest strings (`.csproj`, `build.gradle.kts`, `package.json`, `Package.swift`). Still pure string assertions — no files written. |

All three use the [`insta`](https://docs.rs/insta) crate for snapshot assertions.

## Cross-language expect-file tests (`tests` module, `src/tests/`)

Each sub-module defines one or more Rust types and invokes the `test!` macro, which reflects
the types and runs the full [`CodeGenerator`](crate::generation::CodeGenerator) pipeline for every listed language
(e.g. `for kotlin, swift`). The output is compared against checked-in expect files
(`output.kt`, `output.swift`, …) sitting alongside each `mod.rs`, using the
[`expect_test`](https://docs.rs/expect_test) crate. These tests are fast (no compiler
invocation) but exercise the complete generator path — including package declarations, imports,
and multi-type ordering — across multiple languages in a single test case. Every test should
support all languages, except for a few that exercise language-specific features like
`#[facet(swift = "Equatable")]` or `#[facet(kotlin = "Parcelable")]`.

Gated on `#[cfg(all(test, feature = "generate"))]`.

## Compilation tests (`tests/<lang>_generation.rs`)

Integration tests that generate code **and** a project scaffold into a temporary directory,
then invoke the real compiler (`dotnet build`, `gradle build`, `swift build`, `deno check`).
They verify that the generated code is syntactically and type-correct in the target language.
Each file is feature-gated (e.g. `#![cfg(feature = "kotlin")]`) so tests only run when the
corresponding toolchain is available.

## Runtime tests (`tests/<lang>_runtime.rs`)

End-to-end tests that go one step further: they serialize sample data in Rust (typically with
bincode), generate target-language code that deserializes the same bytes, compile and **run**
the resulting program, and assert that the round-trip is correct. These catch subtle encoding
bugs that snapshot and compilation tests cannot.

## Doctests

The Rust examples in the guide and the API docs are doctests, and must compile and pass.
nextest does not run doctests, so `just test` runs `cargo test --doc --all-features` after it.
