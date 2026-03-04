pub use generator::CodeGenerator;
pub use installer::Installer;

mod emitter;
mod generator;
mod installer;

use include_dir::{Dir, include_dir};

/// Installation target (node.js or deno)
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
                static DIR: Dir<'_> = include_dir!("runtime/typescript-node/serde");
                &DIR
            }
            Self::Deno => {
                static DIR: Dir<'_> = include_dir!("runtime/typescript-deno/serde");
                &DIR
            }
        }
    }

    pub(crate) fn bincode_runtime(self) -> &'static Dir<'static> {
        match self {
            Self::Node => {
                static DIR: Dir<'_> = include_dir!("runtime/typescript-node/bincode");
                &DIR
            }
            Self::Deno => {
                static DIR: Dir<'_> = include_dir!("runtime/typescript-deno/bincode");
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
