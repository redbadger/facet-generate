//! Generates idiomatic source code in Swift, Kotlin, TypeScript, and C# from Rust types
//! annotated with `#[derive(Facet)]`.
//!
//! When Rust is the source of truth for a data model shared across platforms (e.g. a mobile
//! app talking to a Rust core over FFI), every target language needs matching type definitions
//! — and, optionally, serialization code to move data across the boundary. Writing these by
//! hand is tedious and error-prone; this crate automates it.
//!
//! Optionally, when an [`Encoding`](generation::Encoding) such as Bincode or JSON is
//! configured, the generated types include `serialize` / `deserialize` methods and the
//! appropriate runtime library is installed alongside the generated code.
//!
//! # Modules
//!
//! - [`reflection`] — walks Rust type metadata (via the [`facet`] crate) and builds a
//!   language-neutral [`Registry`]: a flat map from qualified type names to their
//!   [`ContainerFormat`] descriptions.
//! - [`generation`] — transforms a registry into source code for a target language.
//!   Each language (`kotlin`, `csharp`, `swift`, `typescript`) lives behind a feature flag and
//!   follows a three-layer pipeline: **Installer** (project scaffolding and manifests) →
//!   **Generator** (file-level output with imports and namespaces) → **Emitter** (per-type
//!   code emission).
//!
//! # Getting Started
//!
//! Add the crates to your project:
//!
//! ```sh
//! cargo add facet facet_generate
//! ```
//!
//! ## 1. Annotate your types
//!
//! Derive [`facet::Facet`] on every type you want to share across language boundaries.
//! Aliasing this crate as `fg` keeps attribute paths short:
//!
//! ```rust,ignore
//! use facet::Facet;
//! use facet_generate as fg;
//!
//! #[derive(Facet)]
//! #[repr(C)]
//! enum HttpResult {
//!     Ok(HttpResponse),
//!     Err(HttpError),
//! }
//!
//! #[derive(Facet)]
//! struct HttpResponse {
//!     status: u16,
//!     headers: Vec<HttpHeader>,
//!     #[facet(fg::bytes)]          // Vec<u8> → native byte-array type
//!     body: Vec<u8>,
//! }
//!
//! #[derive(Facet)]
//! struct HttpHeader {
//!     name: String,
//!     value: String,
//! }
//! ```
//!
//! You only need to register **root types** — all referenced types are collected transitively.
//!
//! ## 2. Build a [`Registry`]
//!
//! ```rust,ignore
//! use facet_generate::reflection::RegistryBuilder;
//!
//! let registry = RegistryBuilder::new()
//!     .add_type::<HttpResult>()?
//!     .build()?;
//! ```
//!
//! ## 3. Generate code
//!
//! Pass the [`Registry`] to a language-specific installer, optionally configure an
//! [`Encoding`](generation::Encoding), and call `generate()`:
//!
//! ```rust,ignore
//! use facet_generate::generation::{self, Encoding};
//! use generation::{kotlin, swift, typescript};
//! use generation::typescript::InstallTarget;
//!
//! // Swift package with Bincode serialization
//! swift::Installer::new("MyPackage", &out_dir)
//!     .encoding(Encoding::Bincode)
//!     .generate(&registry)?;
//!
//! // Kotlin package with Bincode serialization
//! kotlin::Installer::new("com.example", &out_dir)
//!     .encoding(Encoding::Bincode)
//!     .generate(&registry)?;
//!
//! // TypeScript (Node) with Bincode serialization
//! typescript::Installer::new("my-package", &out_dir, InstallTarget::Node)
//!     .encoding(Encoding::Bincode)
//!     .generate(&registry)?;
//! ```
//!
//! Each installer writes a ready-to-build project to `out_dir` — type definitions plus,
//! when encoding is configured, the appropriate serialization runtime. Omit
//! `.encoding(...)` to generate plain type definitions without any serialization code.
//!
//! ## Key attributes
//!
//! | Attribute | Effect |
//! |---|---|
//! | `#[facet(fg::bytes)]` | Emit `Vec<u8>` / `&[u8]` as a native byte-array type (`[UInt8]`, `ByteArray`, `Uint8Array`) |
//! | `#[facet(fg::namespace = "ns")]` | Group a type, transitively, into a named namespace, emitted as a separate module |
//! | `#[facet(fg::namespace)]` | Group a type, transitively, into the ROOT namespace |
//! | `#[facet(rename = "Name")]` | Override the generated name of a type, field, or variant |
//! | `#[facet(rename_all = "camelCase")]` | Apply a naming convention across all fields or variants. Options are `PascalCase`, `camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE` |
//! | `#[facet(skip)]` | Exclude a field or variant from the generated output |
//! | `#[facet(opaque)]` | Do not descend into the field's type |
//! | `#[facet(transparent)]` | Unwrap a newtype wrapper in the generated output |
//!
//! # Testing
//!
//! Tests are organized in four layers, from fast and narrow to slow and broad:
//!
//! ## Unit tests (snapshot)
//!
//! Each language has snapshot-based tests that assert on generated **text** without touching the
//! filesystem.
//!
//! | Layer | Location | What it covers |
//! |-------|----------|----------------|
//! | Emitter | `generation/<lang>/emitter/tests.rs` (+ `tests_bincode.rs`, `tests_json.rs`) | Output for individual types — no file headers, no imports. Uses the `emit` macro. |
//! | Generator | `generation/<lang>/generator/tests.rs` | Full file output including package declarations, imports, and namespace-qualified names. |
//! | Installer | `generation/<lang>/installer/tests.rs` | Generated manifest strings (`.csproj`, `build.gradle.kts`, `package.json`, `Package.swift`). Still pure string assertions — no files written. |
//!
//! All three use the [`insta`](https://docs.rs/insta) crate for snapshot assertions.
//!
//! ## Cross-language expect-file tests (`tests` module, `src/tests/`)
//!
//! Each sub-module defines one or more Rust types and invokes the `test!` macro, which reflects
//! the types and runs the full [`CodeGenerator`](generation::CodeGenerator) pipeline for every listed language
//! (e.g. `for kotlin, swift`). The output is compared against checked-in expect files
//! (`output.kt`, `output.swift`, …) sitting alongside each `mod.rs`, using the
//! [`expect_test`](https://docs.rs/expect_test) crate. These tests are fast (no compiler
//! invocation) but exercise the complete generator path — including package declarations, imports,
//! and multi-type ordering — across multiple languages in a single test case. Every test should
//! support all languages, except for a few that exercise language-specific features like
//! `#[facet(swift = "Equatable")]` or `#[facet(kotlin = "Parcelable")]`.
//!
//! Gated on `#[cfg(all(test, feature = "generate"))]`.
//!
//! ## Compilation tests (`tests/<lang>_generation.rs`)
//!
//! Integration tests that generate code **and** a project scaffold into a temporary directory,
//! then invoke the real compiler (`dotnet build`, `gradle build`, `swift build`, `tsc`).
//! They verify that the generated code is syntactically and type-correct in the target language.
//! Each file is feature-gated (e.g. `#![cfg(feature = "kotlin")]`) so tests only run when the
//! corresponding toolchain is available.
//!
//! ## Runtime tests (`tests/<lang>_runtime.rs`)
//!
//! End-to-end tests that go one step further: they serialize sample data in Rust (typically with
//! bincode), generate target-language code that deserializes the same bytes, compile and **run**
//! the resulting program, and assert that the round-trip is correct. These catch subtle encoding
//! bugs that snapshot and compilation tests cannot.

// Re-export attribute macros from facet-generate-attrs.
// This allows users to write e.g. `#[facet(facet_generate::bytes)]`
// or `use facet_generate as fg; #[facet(fg::bytes)]`
pub use facet_generate_attrs::*;

pub mod error;
pub mod generation;
pub mod reflection;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::{
    error::Error,
    reflection::format::{ContainerFormat, QualifiedTypeName},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The registry of reflected types — a flat map from qualified type names to their container formats.
///
/// Built by [`reflection::RegistryBuilder`] (typically via the [`reflect!`] macro) and consumed by
/// language-specific code generators in [`generation`].
///
/// Only named container types (structs and enums) get top-level entries. Primitives, `Option`,
/// `Vec`, `Map`, etc. are represented inline as [`Format`](reflection::format::Format) variants
/// within the containers that use them. Cross-type references are expressed as
/// `Format::TypeName(QualifiedTypeName)` — symbolic look-ups back into this same map.
///
/// Keys are namespace-qualified, so a type `Foo` in the root namespace and a type `Foo` in
/// namespace `Bar` are separate entries. For example, in Kotlin these would generate as `Foo`
/// and `Bar.Foo` respectively.
pub type Registry = BTreeMap<QualifiedTypeName, ContainerFormat>;

/// Test/convenience macro: reflects the given types and emits code for each
/// container using the specified language tag and encoding.
///
/// Returns `anyhow::Result<String>` containing the generated source.
///
/// ```ignore
/// let code = emit!(MyStruct, MyEnum as Kotlin with Encoding::Json)?;
/// ```
///
/// This skips the [`Module`](generation::module::Module) header (no `package`
/// or `import` statements) — it only emits the type declarations. Useful in
/// tests to assert on individual type output without the file-level boilerplate.
#[cfg(test)]
#[macro_export]
macro_rules! emit {
    ($($ty:ident),* as $language:ident with $encoding:path) => {
        || -> anyhow::Result<String> {
            use $crate::generation::{
                Container,
                config::CodeGeneratorConfig,
                indent::{IndentConfig, IndentedWriter},
            };
            use std::io::Write as _;
            let mut out = Vec::new();
            let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let registry = $crate::reflect!($($ty),*)?;
            let config = CodeGeneratorConfig::new("testing".to_string());
            let lang = $language::for_encoding($encoding, &registry, &config);
            for container in registry.iter().map(Container::from) {
                writeln!(&mut w)?;
                container.write(&mut w, &lang)?;
            }
            let out = String::from_utf8(out)?;

            Ok(out)
        }()
    };
}

/// **Deprecated since 0.16.0:** The Java generator is deprecated. Use the Kotlin generator instead.
#[cfg(test)]
#[macro_export]
#[deprecated(
    since = "0.16.0",
    note = "The Java generator is deprecated. Use the Kotlin generator instead."
)]
macro_rules! emit_java {
    ($($ty:ident),* as $encoding:path) => {
        #[allow(deprecated)]
        || -> anyhow::Result<String> {
            use $crate::generation::{Encoding, indent::{IndentConfig, IndentedWriter}};
            let mut out = Vec::new();
            let w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let config = $crate::generation::CodeGeneratorConfig::new("com.example".to_string())
                .with_encoding($encoding);
            let generator = $crate::generation::java::JavaCodeGenerator::new(&config);
            let mut emitter = $crate::generation::java::emitter::JavaEmitter {
                out: w,
                generator: &generator,
                current_namespace: Vec::new(),
                current_reserved_names: HashMap::new(),
            };
            let registry = $crate::reflect!($($ty),*)?;
            for (name, format) in &registry {
                emitter.output_container(&name.name, format).unwrap();
            }
            let out = String::from_utf8(out)?;

            Ok(out)
        }()
    };
}

/// Reflects one or more types into a [`Registry`], recursively capturing all reachable types.
///
/// This is a convenience wrapper around [`RegistryBuilder`](reflection::RegistryBuilder) —
/// used directly by the `emit!` macro and available for cases where you need the registry
/// without code generation.
///
/// ```ignore
/// let registry = reflect!(MyStruct, MyEnum)?;
/// ```
#[macro_export]
macro_rules! reflect {
    ($($ty:ident),*) => {
        || -> anyhow::Result<std::collections::BTreeMap<$crate::reflection::format::QualifiedTypeName, $crate::reflection::format::ContainerFormat>> {
            let registry = $crate::reflection::RegistryBuilder::new()
                $(.add_type::<$ty>().map_err(|e| anyhow::anyhow!("failed to add type {}: {}", stringify!($ty), e))?)*
                .build()
                .map_err(|e| anyhow::anyhow!("failed to build registry: {e}"))?;
            Ok(registry)
        }()
    };
}

/// Test-only macro for multi-namespace generation tests.
///
/// Reflects `$facet`, splits the resulting registry by namespace (expecting
/// exactly two namespaces), and runs the full [`CodeGenerator`](generation::CodeGenerator)
/// pipeline for each. Returns `(String, String)` — the generated source for
/// the non-root module and the root module, sorted alphabetically by module
/// name.
///
/// This exercises the complete generator path *including* the
/// [`Module`](generation::module::Module) header (package declaration,
/// imports), unlike [`emit!`] which skips it.
#[cfg(test)]
#[macro_export]
macro_rules! emit_two_modules {
    ($generator:ty, $facet:ident, $root:expr) => {{
        use $crate::generation::CodeGenerator;
        use $crate::generation::module::{self, Module};
        use $crate::{Registry, reflect};

        fn emit_module<'a, G: CodeGenerator<'a>>(
            module: &'a Module,
            registry: &Registry,
        ) -> String {
            let mut out = Vec::new();
            let mut generator = G::new(module.config());
            generator.write_output(&mut out, registry).unwrap();
            String::from_utf8(out).unwrap()
        }

        let registry = reflect!($facet).unwrap();
        let mut modules: Vec<_> = module::split($root, &registry).into_iter().collect();
        modules.sort_by(|a, b| a.0.config().module_name.cmp(&b.0.config().module_name));

        let modules: [(Module, Registry); 2] = modules.try_into().expect("Two modules expected");
        let [(other_module, other_registry), (root_module, root_registry)] = modules;

        let module_1 = emit_module::<$generator>(&other_module, &other_registry);
        let module_2 = emit_module::<$generator>(&root_module, &root_registry);
        (module_1, module_2)
    }};
}
