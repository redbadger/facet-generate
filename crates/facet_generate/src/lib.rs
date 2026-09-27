//! Generates idiomatic source code in Swift, Kotlin, TypeScript, and C# from Rust types
//! annotated with `#[derive(Facet)]`.
//!
//! When Rust is the source of truth for a data model shared across platforms (e.g. a mobile
//! app talking to a Rust core over FFI), every target language needs matching type definitions
//! — and, optionally, serialization code to move data across the boundary. Writing these by
//! hand is tedious and error-prone; this crate automates it.
//!
//! Optionally, when a plugin such as [`BincodePlugin`](generation::bincode::BincodePlugin) or
//! [`JsonPlugin`](generation::json::JsonPlugin) is configured, the generated code includes
//! serialization for that format and the runtime library it needs is installed alongside the
//! generated code.
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
//! cargo add facet facet_generate facet-generate-attrs
//! ```
//!
//! The `#[facet(fg::…)]` attributes expand to paths in `facet_generate_attrs`, so a crate that
//! uses them must depend on `facet-generate-attrs` directly.
//!
//! ## 1. Annotate your types
//!
//! Derive [`facet::Facet`] on every type you want to share across language boundaries.
//! Aliasing this crate as `fg` keeps attribute paths short:
//!
//! ```rust
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
//! # #[derive(Facet)]
//! # struct HttpError {
//! #     message: String,
//! # }
//! ```
//!
//! You only need to register **root types** — all referenced types are collected transitively.
//!
//! ## 2. Build a [`Registry`]
//!
//! ```rust
//! # use facet::Facet;
//! # #[derive(Facet)]
//! # #[repr(C)]
//! # enum HttpResult {
//! #     Ok(String),
//! #     Err(String),
//! # }
//! use facet_generate::reflection::RegistryBuilder;
//!
//! let registry = RegistryBuilder::new()
//!     .add_type::<HttpResult>()?
//!     .build()?;
//! # Ok::<(), facet_generate::error::Error>(())
//! ```
//!
//! ## 3. Generate code
//!
//! Pass the [`Registry`] to a language-specific installer, optionally add plugins for
//! serialization support, and call `generate()`:
//!
//! ```rust
//! # use facet::Facet;
//! # #[derive(Facet)]
//! # struct HttpHeader {
//! #     name: String,
//! #     value: String,
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let registry = facet_generate::reflect!(HttpHeader)?;
//! # let out_dir = std::env::temp_dir().join("facet_generate_getting_started");
//! use facet_generate::generation::{bincode::BincodePlugin, csharp, kotlin, swift, typescript};
//!
//! // Swift package with Bincode serialization
//! swift::Installer::new("MyPackage", out_dir.join("swift"))
//!     .plugin(BincodePlugin)
//!     .generate(&registry)?;
//!
//! // Kotlin package with Bincode serialization
//! kotlin::Installer::new("com.example", out_dir.join("kotlin"))
//!     .plugin(BincodePlugin)
//!     .generate(&registry)?;
//!
//! // C# project with Bincode serialization
//! csharp::Installer::new("Example", out_dir.join("csharp"))
//!     .plugin(BincodePlugin)
//!     .generate(&registry)?;
//!
//! // TypeScript with Bincode serialization
//! typescript::Installer::new("my-package", out_dir.join("typescript"))
//!     .plugin(BincodePlugin)
//!     .generate(&registry)?;
//! # Ok(())
//! # }
//! ```
//!
//! Each installer writes a project to its directory — type definitions plus, when a
//! serialization plugin is configured, the appropriate runtime. The Swift, C# and TypeScript
//! projects build as they are written; the Kotlin sources need moving into `src/main/kotlin`
//! first (see the [Kotlin guide](guide::kotlin)). Omit `.plugin(...)` to generate plain type
//! definitions without any serialization code.
//!
//! ## Key attributes
//!
//! | Attribute | Effect |
//! |---|---|
//! | `#[facet(fg::bytes)]` | Emit `Vec<u8>` / `&[u8]` / `[u8; N]` / `Bytes` as a native byte-array type (`[UInt8]`, `Bytes`, `bytes`, `byte[]`) |
//! | `#[facet(fg::namespace = "ns")]` | Group a type, transitively, into a named namespace, emitted as a separate module |
//! | `#[facet(fg::namespace)]` | Group a type, transitively, into the ROOT namespace |
//! | `#[facet(rename = "Name")]` | Override the generated name of a type, field, or variant |
//! | `#[facet(rename_all = "camelCase")]` | Apply a naming convention across all fields or variants. Options are `PascalCase`, `camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, `kebab-case`, `SCREAMING-KEBAB-CASE` |
//! | `#[facet(skip)]` | Exclude a field or variant from the generated output |
//! | `#[facet(opaque)]` | Exclude a field from the generated output without reflecting its type, as for a type that can't be generated |
//! | `#[facet(transparent)]` | Unwrap a newtype wrapper in the generated output |
//!
//! # Guide
//!
//! The [`guide`] module goes further than this page: why this crate exists and how it compares
//! with other tools, the Rust types it supports in each language, a page per target language,
//! the serialization formats, and writing your own plugin.

// Re-export attribute macros from facet-generate-attrs.
// This allows users to write e.g. `#[facet(facet_generate::bytes)]`
// or `use facet_generate as fg; #[facet(fg::bytes)]`
pub use facet_generate_attrs::*;

pub mod error;
pub mod generation;
pub mod guide;
pub mod reflection;

/// Re-exported for the `reflect!` macro, so that its callers don't need `anyhow` themselves.
#[doc(hidden)]
pub use anyhow as __anyhow;

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
/// container using the specified language tag, optionally with plugins.
///
/// Returns `anyhow::Result<String>` containing the generated source.
///
/// ```ignore
/// // Plain type declarations (no serialization)
/// let code = emit!(MyStruct, MyEnum as Kotlin)?;
///
/// // With a plugin (e.g. bincode serialization)
/// let code = emit!(MyStruct, MyEnum as Kotlin with BincodePlugin)?;
///
/// // With multiple plugins
/// let code = emit!(MyStruct as Swift with BincodePlugin, MyCustomPlugin)?;
/// ```
///
/// This skips the [`Module`](generation::module::Module) header (no `package`
/// or `import` statements) — it only emits the type declarations. Useful in
/// tests to assert on individual type output without the file-level boilerplate.
#[cfg(test)]
#[macro_export]
macro_rules! emit {
    ($($ty:ident),* as $language:ident) => {
        emit!($($ty),* as $language with)
    };
    ($($ty:ident),* as $language:ident with $($plugin:expr),* $(,)?) => {
        || -> $crate::__anyhow::Result<String> {
            use $crate::generation::{Container, Emitter as _, CodeGeneratorConfig, indent::IndentedWriter};
            use std::io::Write as _;
            let mut out = Vec::new();
            let mut cfg = CodeGeneratorConfig::new("test".to_string());
            let registry = $crate::reflect!($($ty),*)?;
            cfg.update_from(&registry);
            let mut w = IndentedWriter::new(&mut out, cfg.indent);
            let lang = $language::new(&cfg, &registry)
                $(.with_plugin(Arc::new($plugin)))*;
            for container in registry.iter().map(Container::from) {
                writeln!(&mut w)?;
                container.write(&mut w, &lang)?;
            }
            Ok(String::from_utf8(out)?)
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
        || -> $crate::__anyhow::Result<::std::collections::BTreeMap<$crate::reflection::format::QualifiedTypeName, $crate::reflection::format::ContainerFormat>> {
            let registry = $crate::reflection::RegistryBuilder::new()
                $(.add_type::<$ty>().map_err(|e| $crate::__anyhow::anyhow!("failed to add type {}: {}", stringify!($ty), e))?)*
                .build()
                .map_err(|e| $crate::__anyhow::anyhow!("failed to build registry: {e}"))?;
            ::core::result::Result::Ok(registry)
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
