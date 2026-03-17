//! TypeScript code generation.
//!
//! This module translates a [`Registry`](crate::Registry) of reflected type
//! definitions into idiomatic TypeScript source code.
//!
//! # Submodules (in pipeline order)
//!
//! 1. **generator** — Top-level orchestrator. [`CodeGenerator`](crate::generation::typescript::CodeGenerator) implements
//!    [`CodeGen`](crate::generation::CodeGen) to produce a complete TypeScript source file
//!    from a registry. It resolves qualified type names against the
//!    configuration (external packages, namespaces), carries the active
//!    [`InstallTarget`](crate::generation::typescript::InstallTarget), and delegates writing to the emitter layer.
//!
//! 2. **emitter** — AST-to-source rendering. Implements
//!    [`Emitter<TypeScript>`](crate::generation::Emitter) for each AST node type
//!    ([`Module`](crate::generation::module::Module), [`Container`](crate::generation::Container),
//!    `Named<Format>`, `Format`, `Doc`). This is where the TypeScript language
//!    mapping lives: type aliases, `export class` / `export abstract class` +
//!    variant subclass selection, and `Serializer`/`Deserializer`
//!    interface-based serialize/deserialize method generation. Feature helpers
//!    (`ArrayOfT`, `SetOfT`, etc.) are embedded as `include_bytes!` snippets
//!    and emitted as needed.
//!
//! 3. **installer** — Project scaffolding. [`Installer`](crate::generation::typescript::Installer) implements
//!    [`SourceInstaller`](crate::generation::SourceInstaller) to write a ready-to-build
//!    TypeScript project: it copies serde/bincode runtime sources (with
//!    target-specific variants for Node vs Deno), splits the registry by
//!    namespace into per-module files, and generates a `package.json` manifest.

pub use emitter::TypeScript;
pub use generator::CodeGenerator;
pub use installer::Installer;

mod emitter;
mod generator;
mod installer;

use include_dir::{Dir, include_dir};

/// Installation target — Node.js or Deno.
///
/// TypeScript supports two distinct module layouts:
///
/// - **Node** — flat `.ts` files with extensionless imports (e.g.
///   `import … from "./serde"`). Runtime entry points use `index.ts`.
///   Each namespace becomes a single `<namespace>.ts` file in the output
///   directory.
///
/// - **Deno** — directory structure with `.ts` extensions kept in import
///   paths (e.g. `import … from "./serde/mod.ts"`). Runtime entry points
///   use `mod.ts`. Each namespace becomes a `<namespace>/mod.ts` file.
#[derive(Debug, Clone, Copy)]
pub enum InstallTarget {
    Node,
    Deno,
}

impl InstallTarget {
    pub(crate) fn serde_import_path(&self) -> &str {
        match self {
            InstallTarget::Node => "serde",
            InstallTarget::Deno => "serde/mod.ts",
        }
    }

    pub(crate) fn serde_runtime(self) -> &'static Dir<'static> {
        match self {
            Self::Node => {
                static DIR: Dir<'_> =
                    include_dir!("$CARGO_MANIFEST_DIR/runtime/typescript-node/serde");
                &DIR
            }
            Self::Deno => {
                static DIR: Dir<'_> =
                    include_dir!("$CARGO_MANIFEST_DIR/runtime/typescript-deno/serde");
                &DIR
            }
        }
    }

    pub(crate) fn bincode_runtime(self) -> &'static Dir<'static> {
        match self {
            Self::Node => {
                static DIR: Dir<'_> =
                    include_dir!("$CARGO_MANIFEST_DIR/runtime/typescript-node/bincode");
                &DIR
            }
            Self::Deno => {
                static DIR: Dir<'_> =
                    include_dir!("$CARGO_MANIFEST_DIR/runtime/typescript-deno/bincode");
                &DIR
            }
        }
    }

    pub(crate) fn transform_import_path(self, content: &str) -> String {
        match self {
            Self::Node => content
                .lines()
                .map(|line| {
                    let trimmed = line.trim_start();
                    if (trimmed.starts_with("import") || trimmed.starts_with("export"))
                        && line.contains(".ts")
                    {
                        line.replace(".ts\"", "\"").replace(".ts'", "'")
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Deno => content.to_string(),
        }
    }

    pub(crate) fn transform_runtime_filename(self, filename: &str) -> String {
        match self {
            Self::Node => {
                if filename == "mod.ts" {
                    "index.ts".to_string()
                } else {
                    filename.to_string()
                }
            }
            Self::Deno => filename.to_string(),
        }
    }
}
