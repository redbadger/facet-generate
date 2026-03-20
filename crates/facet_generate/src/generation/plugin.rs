//! Plugin infrastructure for extending code generation.
//!
//! The `EmitterPlugin<L>` trait defines well-known extension points that
//! plugins can hook into to inject additional code at each stage of the
//! generation pipeline — imports, type annotations, protocol conformances,
//! method bodies, runtime files, and manifest dependencies.
//!
//! Each plugin is parameterized by a **language tag** `L` (e.g. `Kotlin`,
//! `Swift`), so the same plugin crate can provide different implementations
//! for each target language. For instance, a bincode plugin would implement
//! `EmitterPlugin<Kotlin>` and `EmitterPlugin<Swift>` separately, since the
//! generated serialization code differs between languages.
//!
//! Plugins are stored in the language tag as `Vec<Arc<dyn EmitterPlugin<L>>>`
//! and invoked by the emitter functions at each extension point. The plugin
//! list is built once when the language tag is constructed (typically in the
//! installer or generator) and shared immutably throughout the generation run.
//!
//! # Extension points
//!
//! | Method | When it's called | Example use |
//! |---|---|---|
//! | `imports` | Module header | `import kotlinx.serialization.*` |
//! | `type_annotations` | Before a type declaration | `@Serializable` |
//! | `type_conformances` | After the type name | `: KeyPathMutable<Foo>` |
//! | `type_body` | Inside the type body, after fields | `fun patching(...)` |
//! | `after_type` | After the closing brace of a type | extension methods |
//! | `module_helpers` | After imports, before types | feature helper snippets |
//! | `field_annotations` | Before a field declaration | `@SerialName("foo")` |
//! | `runtime_files` | During installation | serde/bincode runtime `.kt` files |
//! | `manifest_dependencies` | When writing the build manifest | `kotlinx-serialization-json` |

use std::io;

use super::{CodeGeneratorConfig, Container, indent::IndentWrite};
use crate::reflection::format::{Format, Named};

/// A plugin that injects additional code at well-defined extension points
/// in the code-generation pipeline.
///
/// `L` is the **language tag** — e.g.
/// [`Kotlin`](super::kotlin::Kotlin),
/// [`Swift`](super::swift::Swift),
/// [`TypeScript`](super::typescript::TypeScript),
/// [`CSharp`](super::csharp::CSharp).
/// A plugin crate provides a separate `impl EmitterPlugin<L>` for each
/// language it supports.
///
/// All methods have default (no-op) implementations, so plugins only need to
/// override the extension points they care about.
///
/// # Object safety
///
/// This trait is object-safe. Language tags store plugins as
/// `Arc<dyn EmitterPlugin<L>>` so that heterogeneous plugins can coexist in
/// the same list and the list is cheaply cloneable.
///
/// The [`type_body`](Self::type_body) and [`after_type`](Self::after_type)
/// methods receive `&mut dyn IndentWrite` (a trait object) rather than a
/// generic `W: IndentWrite`, which preserves object safety. Plugins can call
/// [`indent()`](IndentWrite::indent) / [`unindent()`](IndentWrite::unindent)
/// and all [`Write`](io::Write) methods on this object. The RAII
/// [`block()`](IndentWrite::block) helper requires `Self: Sized` and is
/// therefore unavailable on `dyn IndentWrite` — use manual `write!("{{")`
/// + `indent()` / `unindent()` + `write!("}}") ` instead.
pub trait EmitterPlugin<L>: std::fmt::Debug {
    /// Extra import statements to include in the module header.
    ///
    /// Called once per module, before any types are emitted. The returned
    /// strings are merged with the language's built-in imports and
    /// deduplicated.
    ///
    /// # Examples
    ///
    /// ```text
    /// vec!["import kotlinx.serialization.Serializable".into()]
    /// ```
    fn imports(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec![]
    }

    /// Annotations to emit immediately before a type declaration.
    ///
    /// Each string is written on its own line above the `data class` /
    /// `struct` / `enum` / etc.
    ///
    /// # Examples
    ///
    /// ```text
    /// vec!["@Serializable".into(), r#"@SerialName("Foo")"#.into()]
    /// ```
    fn type_annotations(&self, _container: &Container) -> Vec<String> {
        vec![]
    }

    /// Protocol or interface conformances to append to the type declaration.
    ///
    /// The emitter joins these with `, ` and appends them after the type
    /// name (and any existing conformances). The separator and syntax
    /// (`: ` vs ` : ` vs nothing) are handled by the language emitter.
    ///
    /// # Examples
    ///
    /// ```text
    /// vec!["KeyPathMutable<Foo>".into(), "Serializable".into()]
    /// ```
    fn type_conformances(&self, _container: &Container) -> Vec<String> {
        vec![]
    }

    /// Extra code to emit inside the type body, after all fields have been
    /// written but before the closing brace.
    ///
    /// Use this for methods, companion objects, nested types, etc.
    ///
    /// The writer is already indented to the correct level inside the type
    /// body.
    fn type_body(&self, _w: &mut dyn IndentWrite, _container: &Container) -> io::Result<()> {
        Ok(())
    }

    /// Extra code to emit after the type's closing brace.
    ///
    /// Use this for extension methods, free functions, or companion
    /// declarations that must appear outside the type.
    fn after_type(&self, _w: &mut dyn IndentWrite, _container: &Container) -> io::Result<()> {
        Ok(())
    }

    /// Module-level helper code to emit after imports but before any type
    /// declarations.
    ///
    /// Use this for feature helper snippets (e.g. `ListOfT.kt`),
    /// type aliases, or other module-scoped declarations that types depend on.
    fn module_helpers(
        &self,
        _w: &mut dyn IndentWrite,
        _config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        Ok(())
    }

    /// Annotations to emit before an individual field declaration.
    ///
    /// Called once per field in a struct or data class. The `container`
    /// parameter provides context about the enclosing type.
    ///
    /// # Examples
    ///
    /// ```text
    /// vec![r#"@SerialName("myField")"#.into()]
    /// ```
    fn field_annotations(&self, _field: &Named<Format>, _container: &Container) -> Vec<String> {
        vec![]
    }

    /// Runtime support files to install alongside the generated code.
    ///
    /// Called by the installer after code generation. Each [`RuntimeFile`]
    /// describes a file to write into the output directory.
    fn runtime_files(&self) -> Vec<RuntimeFile> {
        vec![]
    }

    /// Extra dependency entries for the package manifest.
    ///
    /// The format of each string is language-specific — for example, a
    /// Gradle dependency line for Kotlin or an SPM `.package(...)` entry
    /// for Swift.
    ///
    /// # Examples
    ///
    /// ```text
    /// vec![r#"implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")"#.into()]
    /// ```
    fn manifest_dependencies(&self) -> Vec<String> {
        vec![]
    }
}

/// A file to be written into the output directory during installation.
///
/// Returned by [`EmitterPlugin::runtime_files`].
#[derive(Debug, Clone)]
pub struct RuntimeFile {
    /// Path relative to the installation root directory.
    ///
    /// For example, `"com/novi/serde/Serializer.kt"` or
    /// `"Sources/Serde/Serializer.swift"`.
    pub relative_path: String,

    /// The raw file contents.
    pub contents: Vec<u8>,
}

/// Helper to invoke a plugin method that returns `Vec<String>` across all
/// plugins in a list and collect the results into a single `Vec`.
///
/// This avoids repetitive `iter().flat_map().collect()` at every call site.
pub fn collect_from_plugins<L, F>(
    plugins: &[std::sync::Arc<dyn EmitterPlugin<L>>],
    f: F,
) -> Vec<String>
where
    F: Fn(&dyn EmitterPlugin<L>) -> Vec<String>,
{
    plugins.iter().flat_map(|p| f(p.as_ref())).collect()
}

/// Helper to invoke a plugin method that writes to a writer across all
/// plugins in a list.
///
/// Calls each plugin's method in order. Returns the first error encountered,
/// if any.
pub fn write_from_plugins<L, F>(
    plugins: &[std::sync::Arc<dyn EmitterPlugin<L>>],
    w: &mut dyn IndentWrite,
    f: F,
) -> io::Result<()>
where
    F: Fn(&dyn EmitterPlugin<L>, &mut dyn IndentWrite) -> io::Result<()>,
{
    for plugin in plugins {
        f(plugin.as_ref(), w)?;
    }
    Ok(())
}
