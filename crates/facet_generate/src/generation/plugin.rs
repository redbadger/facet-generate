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
//! | `has_type_body` | Before deciding to open `{ }` | tell emitter a body is needed |
//! | `type_body_preamble` | Start of type body, before variants | abstract method declarations |
//! | `type_body` | Inside the type body, after fields | `fun patching(...)` |
//! | `after_type` | After the closing brace of a type | extension methods |
//! | `module_helpers` | After imports, before types | feature helper snippets |
//! | `field_annotations` | Before a field declaration | `@SerialName("foo")` |
//! | `runtime_files` | During installation | serde/bincode runtime `.kt` files |
//! | `manifest_dependencies` | When writing the build manifest | `kotlinx-serialization-json` |

use std::io;
use std::sync::Arc;

use super::{CodeGeneratorConfig, Container, indent::IndentWrite};
use crate::reflection::format::{Format, Named, VariantFormat};

// ---------------------------------------------------------------------------
// Context types passed to plugin methods
// ---------------------------------------------------------------------------

/// Full context for a plugin extension-point invocation.
///
/// Wraps the [`Container`] (the top-level type being emitted) together with
/// optional [`VariantInfo`] when the call site is inside a sealed-interface /
/// enum variant rather than a top-level type.
#[derive(Debug, Clone)]
pub struct EmitContext<'a> {
    /// The top-level container (struct or enum) being emitted.
    pub container: &'a Container<'a>,

    /// The fully-populated [`CodeGeneratorConfig`] for the current module.
    ///
    /// Available to plugins so they can read derived data (e.g.
    /// `unit_variant_enums`, `features`, `external_packages`) without
    /// needing to cache it at construction time.
    pub config: &'a CodeGeneratorConfig,

    /// When the current emission site is a variant inside an enum /
    /// sealed interface, this carries the variant-specific details.
    /// `None` for top-level types.
    pub variant: Option<VariantInfo<'a>>,
}

impl<'a> EmitContext<'a> {
    /// Create a context for a top-level type (no variant).
    #[must_use]
    pub const fn top_level(container: &'a Container<'a>, config: &'a CodeGeneratorConfig) -> Self {
        Self {
            container,
            config,
            variant: None,
        }
    }

    /// Create a context for a variant inside a sealed interface / enum.
    #[must_use]
    pub const fn for_variant(
        container: &'a Container<'a>,
        config: &'a CodeGeneratorConfig,
        variant: VariantInfo<'a>,
    ) -> Self {
        Self {
            container,
            config,
            variant: Some(variant),
        }
    }

    /// Whether this context represents a variant (as opposed to a top-level
    /// type).
    #[must_use]
    pub const fn is_variant(&self) -> bool {
        self.variant.is_some()
    }

    /// Convenience: the simple name of the entity being emitted.
    ///
    /// For top-level types this is the container name; for variants it is the
    /// variant name.
    #[must_use]
    pub fn name(&self) -> &str {
        match &self.variant {
            Some(v) => v.name,
            None => &self.container.name.name,
        }
    }

    /// Return the normalized fields for the current entity.
    ///
    /// This handles the different representations in the AST:
    ///
    /// - **Variant**: returns `variant.fields` directly (the caller already
    ///   normalized newtype → `[Named("value")]` etc.).
    /// - **`Struct`**: returns the struct's named fields.
    /// - **`NewTypeStruct`**: returns a single field named `"value"`.
    /// - **`TupleStruct`**: returns fields named `"field0"`, `"field1"`, …
    /// - **`UnitStruct`** / **`Enum`**: returns an empty slice.
    #[must_use]
    pub fn fields(&self) -> Vec<Named<Format>> {
        use crate::reflection::format::ContainerFormat;

        if let Some(v) = &self.variant {
            return v.fields.to_vec();
        }

        match self.container.format {
            ContainerFormat::UnitStruct(_) | ContainerFormat::Enum(_, _) => vec![],
            ContainerFormat::NewTypeStruct(format, _) => {
                vec![Named::new(format, "value".to_string())]
            }
            ContainerFormat::TupleStruct(formats, _) => formats
                .iter()
                .enumerate()
                .map(|(i, f)| Named::new(f, format!("field{i}")))
                .collect(),
            ContainerFormat::Struct(fields, _) => fields.clone(),
        }
    }
}

/// Details about the enum / sealed-interface variant currently being emitted.
#[derive(Debug, Clone)]
pub struct VariantInfo<'a> {
    /// The variant's own name (e.g. `"Ok"`, `"Err"`).
    pub name: &'a str,

    /// Zero-based discriminant index used for binary serialization.
    pub index: usize,

    /// The variant's payload format.
    pub format: &'a VariantFormat,

    /// The fields of the variant (empty for unit / newtype variants encoded
    /// as a single `value` field — the caller normalizes this before
    /// constructing the info).
    pub fields: &'a [Named<Format>],

    /// The name of the parent sealed interface / enum that owns this variant.
    pub parent_name: &'a str,
}

// ---------------------------------------------------------------------------
// The plugin trait
// ---------------------------------------------------------------------------

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
/// The writer-accepting methods receive `&mut dyn IndentWrite` (a trait
/// object) rather than a generic `W: IndentWrite`, which preserves object
/// safety. Plugins can call
/// [`indent()`](IndentWrite::indent) / [`unindent()`](IndentWrite::unindent)
/// and all [`Write`](io::Write) methods on this object. For `{ }` block
/// scoping, use the free function
/// [`with_block()`](super::indent::with_block) (the RAII
/// [`block()`](IndentWrite::block) helper requires `Self: Sized` and is
/// therefore unavailable on `dyn IndentWrite`).
pub trait EmitterPlugin<L>: std::fmt::Debug {
    // ----- module-level hooks -----

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

    /// Module-level helper code to emit after imports but before any type
    /// declarations.
    ///
    /// Use this for feature helper snippets (e.g. `ListOfT.kt`),
    /// type aliases, or other module-scoped declarations that types depend on.
    ///
    /// # Errors
    ///
    /// Returns an error if writing the helper code fails.
    fn module_helpers(
        &self,
        _w: &mut dyn IndentWrite,
        _config: &CodeGeneratorConfig,
    ) -> io::Result<()> {
        Ok(())
    }

    // ----- type-level hooks -----

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
    fn type_annotations(&self, _ctx: &EmitContext) -> Vec<String> {
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
    fn type_conformances(&self, _ctx: &EmitContext) -> Vec<String> {
        vec![]
    }

    /// Whether this plugin needs a type body block `{ … }` for the given
    /// context.
    ///
    /// Emitters call this (across all plugins) to decide whether to open a
    /// `{ }` block after a type declaration. If **any** plugin returns
    /// `true`, the block is opened and [`type_body_preamble`](Self::type_body_preamble) /
    /// [`type_body`](Self::type_body) will be called inside it.
    ///
    /// The default returns `false`.
    fn has_type_body(&self, _ctx: &EmitContext) -> bool {
        false
    }

    /// Code at the very start of the type body, before any fields or
    /// variants are written.
    ///
    /// In Kotlin sealed interfaces this is where abstract method
    /// declarations (e.g. `fun serialize(serializer: Serializer)`) and
    /// convenience wrappers (e.g. `bincodeSerialize()`) go.
    ///
    /// Only called when [`has_type_body`](Self::has_type_body) returned
    /// `true` for at least one plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if writing the preamble code fails.
    fn type_body_preamble(&self, _w: &mut dyn IndentWrite, _ctx: &EmitContext) -> io::Result<()> {
        Ok(())
    }

    /// Extra code to emit inside the type body, after all fields / variants
    /// have been written but before the closing brace.
    ///
    /// Use this for methods, companion objects, nested types, etc.
    ///
    /// The writer is already indented to the correct level inside the type
    /// body.
    ///
    /// # Errors
    ///
    /// Returns an error if writing the body code fails.
    fn type_body(&self, _w: &mut dyn IndentWrite, _ctx: &EmitContext) -> io::Result<()> {
        Ok(())
    }

    /// Extra code to emit after the type's closing brace.
    ///
    /// Use this for extension methods, free functions, or companion
    /// declarations that must appear outside the type.
    ///
    /// # Errors
    ///
    /// Returns an error if writing the after-type code fails.
    fn after_type(&self, _w: &mut dyn IndentWrite, _ctx: &EmitContext) -> io::Result<()> {
        Ok(())
    }

    // ----- field-level hooks -----

    /// Annotations to emit before an individual field declaration.
    ///
    /// Called once per field in a struct or data class. The `ctx`
    /// parameter provides context about the enclosing type.
    ///
    /// # Examples
    ///
    /// ```text
    /// vec![r#"@SerialName("myField")"#.into()]
    /// ```
    fn field_annotations(&self, _field: &Named<Format>, _ctx: &EmitContext) -> Vec<String> {
        vec![]
    }

    /// Inline annotations to prepend to an `enum class` variant declaration.
    ///
    /// Called for each all-unit variant inside an `enum class`. Unlike
    /// [`type_annotations`](Self::type_annotations), these are rendered on
    /// the **same line** as the uppercased variant name, e.g.:
    ///
    /// ```text
    /// @SerialName("Variant1") VARIANT1,
    /// ```
    ///
    /// The returned strings are space-joined and written immediately before
    /// the variant name, with a trailing space separator.
    ///
    /// # Examples
    ///
    /// ```text
    /// vec![r#"@SerialName("Variant1")"#.into()]
    /// ```
    fn enum_variant_annotations(&self, _name: &str) -> Vec<String> {
        vec![]
    }

    // ----- installation hooks -----

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

// ---------------------------------------------------------------------------
// RuntimeFile
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Helpers for calling plugin lists
// ---------------------------------------------------------------------------

/// Invoke a string-returning plugin method across all plugins and collect
/// the results into a single `Vec`.
///
/// Avoids repetitive `iter().flat_map().collect()` at every call site.
pub fn collect_from_plugins<L, F>(plugins: &[Arc<dyn EmitterPlugin<L>>], f: F) -> Vec<String>
where
    F: Fn(&dyn EmitterPlugin<L>) -> Vec<String>,
{
    plugins.iter().flat_map(|p| f(p.as_ref())).collect()
}

/// Invoke a writer-accepting plugin method across all plugins in order.
///
/// Returns the first error encountered, if any.
///
/// # Errors
///
/// Returns an error if any plugin fails to write.
pub fn write_from_plugins<L, F>(
    plugins: &[Arc<dyn EmitterPlugin<L>>],
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

/// Check whether *any* plugin in the list returns `true` for a predicate.
pub fn any_plugin<L, F>(plugins: &[Arc<dyn EmitterPlugin<L>>], f: F) -> bool
where
    F: Fn(&dyn EmitterPlugin<L>) -> bool,
{
    plugins.iter().any(|p| f(p.as_ref()))
}
