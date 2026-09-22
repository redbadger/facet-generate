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
//! | `after_type` | After the closing brace of every top-level type | extension methods |
//! | `module_helpers` | After imports, before types | feature helper snippets |
//! | `field_annotations` | Before a field declaration | `@SerialName("foo")` |
//! | `runtime_files` | During installation | serde/bincode runtime `.kt` files |
//! | `companion_files` | During installation | an extra source file beside the module |
//! | `manifest_dependencies` | When writing the build manifest | `kotlinx-serialization-json` |
//! | `target_dependencies` | When writing the build manifest | `.product(name: "Shared", package: "Shared")` |
//!
//! ## Where `after_type` fires
//!
//! `after_type` is called once for **every top-level container**, in all four
//! languages, with a top-level [`EmitContext`](crate::generation::plugin::EmitContext) (never a variant one):
//!
//! | Language | Call sites |
//! |---|---|
//! | Swift | after `public struct` / `indirect public enum` |
//! | Kotlin | after `data class`, `data object`, `enum class`, `sealed interface` |
//! | TypeScript | after `export class` and after the union type + helpers of an enum |
//! | C# | after `partial class`, `sealed record`, `public enum`, and the `abstract record` variant hierarchy |
//!
//! It is **not** called for the individual variants of an enum, so a plugin
//! that only wants to act on one container shape must say so — see
//! [`EmitterPlugin::after_type`](crate::generation::plugin::EmitterPlugin::after_type).
//!
//! ## Where `companion_files` land
//!
//! A [`CompanionFile`](crate::generation::plugin::CompanionFile) is a whole source file that belongs to the module but
//! does not fit in the module's own file. The generator renders the module
//! header in front of it (imports merged with the file's own, no module
//! helpers) and the installer writes it into the module's directory:
//!
//! | Language | Destination |
//! |---|---|
//! | Swift | `Sources/<Module>/<file_name>` |
//! | Kotlin | the module's package directory |
//! | C# | the module's namespace directory |
//! | TypeScript | — a module is a single file, so the hook is ignored |
//!
//! Unlike [`RuntimeFile`](crate::generation::plugin::RuntimeFile)s, companion files are written even when the serde
//! runtime comes from an external package.

use std::io;
use std::sync::Arc;

use std::collections::BTreeMap;

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

    /// The variants of the container being emitted, keyed by discriminant,
    /// or `None` when the container is not an enum.
    ///
    /// This is the container's variants even inside a variant context — a
    /// plugin that needs the *current* variant should read
    /// [`variant`](Self::variant) instead.
    #[must_use]
    pub const fn variants(&self) -> Option<&BTreeMap<u32, Named<VariantFormat>>> {
        match self.container.format {
            crate::reflection::format::ContainerFormat::Enum(variants, _, _) => Some(variants),
            _ => None,
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
            ContainerFormat::UnitStruct(_) | ContainerFormat::Enum(_, _, _) => vec![],
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
    /// Called once for **every top-level container** in every language, with
    /// a top-level context (`ctx.is_variant()` is always `false`) — see the
    /// [module docs](self#where-after_type-fires) for the exact call sites.
    /// An implementation that is only meaningful for one container shape must
    /// therefore inspect `ctx.container.format` (or
    /// [`ctx.variants()`](EmitContext::variants)) and return `Ok(())` for the
    /// rest.
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

    /// Extra source files to write **beside the generated module file**.
    ///
    /// Unlike [`runtime_files`](Self::runtime_files) — which installers skip
    /// when the serde runtime is provided by an external package — a companion
    /// file is written whenever the module itself is, because it is part of the
    /// module rather than of the shared runtime.
    ///
    /// The generator renders the module's own header (package / namespace
    /// declaration and imports, merged with
    /// [`CompanionFile::imports`]) in front of
    /// [`CompanionFile::contents`]; module helpers are *not* repeated, since
    /// they are already declared in the module file and would collide.
    ///
    /// Only Swift, Kotlin and C# write companion files — TypeScript emits a
    /// single file per module and ignores this hook.
    fn companion_files(&self, _config: &CodeGeneratorConfig) -> Vec<CompanionFile> {
        vec![]
    }

    /// Extra dependency entries for the package manifest.
    ///
    /// The format of each string is language-specific — a Gradle dependency
    /// line for Kotlin, an SPM `.package(...)` entry (unindented) for Swift, or
    /// a `package.json` dependency pair for TypeScript.
    ///
    /// # Examples
    ///
    /// ```text
    /// // Kotlin
    /// vec![r#"implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")"#.into()]
    /// // Swift
    /// vec![".package(\n    path: \"../Shared\"\n)".into()]
    /// // TypeScript
    /// vec![r#""shared": "file:../pkg""#.into()]
    /// ```
    fn manifest_dependencies(&self) -> Vec<String> {
        vec![]
    }

    /// Extra dependency edges for the generated module's build target.
    ///
    /// Swift only: each string is written verbatim into the SPM target's
    /// `dependencies:` array, so it must be a valid `Target.Dependency`
    /// expression.
    ///
    /// Called once per module, with that module's config — a type in a
    /// namespace is generated into a module of its own, and each gets its own
    /// SPM target. A plugin whose edge belongs to one module in particular
    /// (the app's, say) compares [`CodeGeneratorConfig::module_name`] and
    /// returns nothing for the rest; a plugin whose edge every target needs
    /// ignores the argument. Contrast
    /// [`manifest_dependencies`](Self::manifest_dependencies), which takes no
    /// config because the manifest's `dependencies:` are the *package*'s and
    /// are asked for once.
    ///
    /// # Examples
    ///
    /// ```text
    /// vec![r#".product(name: "Shared", package: "Shared")"#.into()]
    /// ```
    fn target_dependencies(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
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
// CompanionFile
// ---------------------------------------------------------------------------

/// A source file written next to the generated module file, sharing the
/// module's package / namespace.
///
/// Returned by [`EmitterPlugin::companion_files`] and rendered by the language
/// generator, which prepends the module header (see
/// [`companion_files`](EmitterPlugin::companion_files)).
#[derive(Debug, Clone)]
pub struct CompanionFile {
    /// File name, including the extension — e.g. `"FfiBridge.swift"`.
    ///
    /// The installer writes it into the same directory as the module file, so
    /// this is a bare name, not a path. When two plugins ask for the same file
    /// name, the first one wins.
    pub file_name: String,

    /// Imports needed by [`contents`](Self::contents) on top of the module's
    /// own imports.
    ///
    /// Each string takes the same shape as [`EmitterPlugin::imports`] for the
    /// language: a bare module name in Swift (`"Foundation"`), a whole
    /// `import` line in Kotlin (`"import com.example.CoreFfi"`), and a whole
    /// `using` directive in C# (`"using Example.Shared;"`).
    pub imports: Vec<String>,

    /// The body of the file, written after the header.
    pub contents: String,
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

/// Collect the companion files of every plugin, rendering each one's header
/// with `write_header` (which receives the file's extra imports and returns the
/// rendered module header).
///
/// The first plugin to claim a file name wins; later plugins asking for the
/// same name are ignored.
///
/// # Errors
///
/// Returns an error if rendering a header fails.
pub(crate) fn render_companion_files<L, F>(
    plugins: &[Arc<dyn EmitterPlugin<L>>],
    config: &CodeGeneratorConfig,
    mut write_header: F,
) -> io::Result<Vec<CompanionFile>>
where
    F: FnMut(&[String]) -> io::Result<String>,
{
    let mut seen = std::collections::BTreeSet::new();
    let mut files = Vec::new();

    for plugin in plugins {
        for file in plugin.companion_files(config) {
            if !seen.insert(file.file_name.clone()) {
                continue;
            }
            let header = write_header(&file.imports)?;
            let contents = join_header(&header, &file.contents);
            files.push(CompanionFile {
                file_name: file.file_name,
                imports: file.imports,
                contents,
            });
        }
    }

    Ok(files)
}

/// Join a rendered module header and a companion file's body, separated by a
/// single blank line and terminated by a newline.
fn join_header(header: &str, body: &str) -> String {
    let header = header.trim_end();
    let body = body.trim_start_matches('\n');

    let mut contents = if header.is_empty() {
        body.to_string()
    } else {
        format!("{header}\n\n{body}")
    };

    if !contents.ends_with('\n') {
        contents.push('\n');
    }

    contents
}

/// Check whether *any* plugin in the list returns `true` for a predicate.
pub fn any_plugin<L, F>(plugins: &[Arc<dyn EmitterPlugin<L>>], f: F) -> bool
where
    F: Fn(&dyn EmitterPlugin<L>) -> bool,
{
    plugins.iter().any(|p| f(p.as_ref()))
}
