//! C# code generation — MVVM-friendly types with file-scoped namespaces.
//!
//! This module translates a [`Registry`](crate::Registry) of reflected type
//! definitions into idiomatic C# source code targeting .NET with the
//! `CommunityToolkit.Mvvm` MVVM pattern.
//!
//! # Submodules (in pipeline order)
//!
//! 1. **[`generator`]** — Top-level orchestrator. [`CodeGenerator`] implements
//!    [`CodeGen`](super::CodeGen) to produce a complete C# source file from a
//!    registry. It resolves qualified type names using the dotted namespace
//!    convention (e.g. `Company.Models.Shared.Child`) and then delegates
//!    writing to the emitter layer.
//!
//! 2. **[`emitter`]** — AST-to-source rendering. Implements
//!    [`Emitter<CSharp>`](super::Emitter) for each AST node type
//!    ([`Module`](super::module::Module), [`Container`](super::Container),
//!    `Named<Format>`, `Format`, `Doc`). Structs become
//!    `partial class : ObservableObject` with `[ObservableProperty]` private
//!    fields; enums become either native `public enum` (all-unit) or
//!    `abstract record` + `sealed record` variant hierarchies (mixed/data).
//!    Serialization uses `System.Text.Json` annotations for JSON and
//!    `IFacetSerializable`/`IFacetDeserializable<T>` interfaces for Bincode.
//!    Unlike Kotlin/Swift/TypeScript, C# serializes collections inline — there
//!    are no feature helper snippet files.
//!
//! 3. **[`installer`]** — Project scaffolding. [`Installer`] implements
//!    [`SourceInstaller`](super::SourceInstaller) to write a ready-to-build
//!    C# project: it copies runtime files to `Facet/Runtime/` subdirectories,
//!    splits the registry by namespace into per-module `.cs` files, and
//!    generates a `.csproj` manifest with NuGet `PackageReference` and
//!    `ProjectReference` entries.

mod emitter;
mod generator;
mod installer;

pub use emitter::CSharp;
pub use generator::CodeGenerator;
pub use installer::Installer;
