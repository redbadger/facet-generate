pub(crate) mod emitter;
mod generator;
mod installer;
mod package;

pub use generator::CodeGenerator;
pub use installer::Installer;

/// Normalize a path string for use in Swift string literals.
/// On Windows, replaces backslashes with forward slashes to avoid
/// Swift interpreting them as escape sequences.
#[must_use]
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}
