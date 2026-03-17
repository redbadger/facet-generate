//! C# code generation — MVVM-friendly types with file-scoped namespaces.
//!
//! This module translates a [`Registry`](crate::Registry) of reflected type
//! definitions into idiomatic C# source code targeting .NET with the
//! `CommunityToolkit.Mvvm` MVVM pattern.
//!
//! # Submodules (in pipeline order)
//!
//! 1. **generator** — Top-level orchestrator. [`CSharpCodeGenerator`](crate::generation::csharp::CSharpCodeGenerator) implements
//!    [`CodeGenerator`](crate::generation::CodeGenerator) to produce a complete C# source file from a
//!    registry. It resolves qualified type names using the dotted namespace
//!    convention (e.g. `Company.Models.Shared.Child`) and then delegates
//!    writing to the emitter layer.
//!
//! 2. **emitter** — AST-to-source rendering. Implements
//!    [`Emitter<CSharp>`](crate::generation::Emitter) for each AST node type
//!    ([`Module`](crate::generation::module::Module), [`Container`](crate::generation::Container),
//!    `Named<Format>`, `Format`, `Doc`). Structs become
//!    `partial class : ObservableObject` with `[ObservableProperty]` private
//!    fields; enums become either native `public enum` (all-unit) or
//!    `abstract record` + `sealed record` variant hierarchies (mixed/data).
//!    Serialization uses `System.Text.Json` annotations for JSON and
//!    `IFacetSerializable`/`IFacetDeserializable<T>` interfaces for Bincode.
//!    Like Kotlin/Swift/TypeScript, C# uses reusable helper functions for
//!    serializing container types. C# places these in a shared runtime file
//!    (`Facet/Runtime/Bincode/FacetHelpers.cs`) rather than per-module snippets,
//!    since C# `using` directives make a shared namespace globally accessible.
//!
//! 3. **installer** — Project scaffolding. [`Installer`](crate::generation::csharp::Installer) implements
//!    [`SourceInstaller`](crate::generation::SourceInstaller) to write a ready-to-build
//!    C# project: it copies runtime files to `Facet/Runtime/` subdirectories,
//!    splits the registry by namespace into per-module `.cs` files, and
//!    generates a `.csproj` manifest with NuGet `PackageReference` and
//!    `ProjectReference` entries.

mod emitter;
mod generator;
mod installer;

pub use emitter::CSharp;
pub use generator::CSharpCodeGenerator;
pub use installer::Installer;
