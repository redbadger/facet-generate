#![allow(clippy::missing_errors_doc)]

pub use generator::CodeGenerator;
pub use installer::Installer;

mod emitter;
mod generator;
mod installer;

/// Installation target (node.js or deno)
#[derive(Clone, Copy)]
pub enum InstallTarget {
    Node,
    Deno,
}
