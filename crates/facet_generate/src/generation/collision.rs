//! The error the installers report when a namespace's generated name collides
//! with a type, the root package or another namespace.
//!
//! Each installer turns a namespace into an identifier or a path: a Swift
//! module, a TypeScript `import * as` binding and file, a Kotlin package, a C#
//! namespace. The rules for when two of those collide differ per language, so
//! each installer checks its own before it writes anything; this module only
//! words the error, so that every language reports a collision the same way.

use std::{collections::BTreeMap, fmt, io};

use crate::{
    generation::ExternalPackages,
    reflection::format::{Namespace, QualifiedTypeName},
};

/// What the reader can do about a collision.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Fix {
    /// A type collides with a namespace.
    RenameType,
    /// A type collides with the root package.
    RenameTypeOrPackage,
    /// Two namespaces, or a namespace and a builtin, collide.
    ChooseNamespace,
    /// A namespace collides with the root package.
    ChooseNamespaceOrPackage,
}

impl fmt::Display for Fix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RENAME: &str = "Rename the type with `#[facet(rename = \"...\")]`";
        match self {
            Self::RenameType => write!(f, "{RENAME} or choose a different namespace"),
            Self::RenameTypeOrPackage => write!(f, "{RENAME} or choose a different package name"),
            Self::ChooseNamespace => write!(f, "Choose a different namespace"),
            Self::ChooseNamespaceOrPackage => {
                write!(f, "Choose a different namespace or package name")
            }
        }
    }
}

/// The namespace or package a generated module comes from, as the error names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Origin<'a> {
    /// The root package, whose module holds the ROOT types.
    RootPackage(&'a str),
    /// A namespace declared with `#[facet(fg::namespace = "...")]`.
    Namespace(&'a str),
}

impl<'a> Origin<'a> {
    /// The origin of the module named `module_name`, for a language whose
    /// module names are the namespace itself or, for the root module, the
    /// root package (Swift and TypeScript).
    #[cfg_attr(not(any(feature = "swift", feature = "typescript")), allow(dead_code))]
    pub(crate) fn of_module(module_name: &'a str, root_package: &str) -> Self {
        if module_name == root_package {
            Self::RootPackage(module_name)
        } else {
            Self::Namespace(module_name)
        }
    }

    /// The origin of the module that generates `namespace`, for a language
    /// whose module names are nested under the root package (Kotlin and C#).
    #[cfg_attr(not(any(feature = "kotlin", feature = "csharp")), allow(dead_code))]
    pub(crate) const fn of_namespace(namespace: &'a Namespace, root_package: &'a str) -> Self {
        match namespace {
            Namespace::Root => Self::RootPackage(root_package),
            Namespace::Named(namespace) => Self::Namespace(namespace.as_str()),
        }
    }

    /// The fix for a collision between this origin and a type.
    pub(crate) const fn rename_type(self) -> Fix {
        match self {
            Self::RootPackage(_) => Fix::RenameTypeOrPackage,
            Self::Namespace(_) => Fix::RenameType,
        }
    }

    /// The fix for a collision between this origin and a namespace.
    pub(crate) const fn choose_namespace(self) -> Fix {
        match self {
            Self::RootPackage(_) => Fix::ChooseNamespaceOrPackage,
            Self::Namespace(_) => Fix::ChooseNamespace,
        }
    }
}

impl fmt::Display for Origin<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootPackage(package) => write!(f, "the root package \"{package}\""),
            Self::Namespace(namespace) => write!(f, "namespace \"{namespace}\""),
        }
    }
}

/// A type as the error names it: its registry name and the namespace it is in.
pub(crate) struct TypeName<'a>(pub &'a QualifiedTypeName);

impl fmt::Display for TypeName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0.namespace {
            Namespace::Root => write!(f, "type `{}` in the root namespace", self.0.name),
            Namespace::Named(namespace) => {
                write!(f, "type `{}` in namespace \"{namespace}\"", self.0.name)
            }
        }
    }
}

/// The error for a collision: `subject` says what a namespace or package
/// becomes, `collider` names what it collides with, and `fix` what to do.
///
/// Reads as `"{language}: {subject}, the same as {collider}. {fix}"`.
pub(crate) fn error(
    language: &str,
    subject: impl fmt::Display,
    collider: impl fmt::Display,
    fix: Fix,
) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("{language}: {subject}, the same as {collider}. {fix}"),
    )
}

/// Fails when the root package is named exactly like a namespace that an
/// external package provides.
///
/// [`module::split`](super::module::split) merges a namespace spelled exactly
/// like the root package into the root module, so that module would be both
/// generated, for the ROOT types, and provided by the external package.
pub(crate) fn check_root_package(
    language: &str,
    root_package: &str,
    external_packages: &ExternalPackages,
) -> io::Result<()> {
    if external_packages.contains_key(root_package) {
        return Err(error(
            language,
            format_args!("the root package is \"{root_package}\""),
            format_args!(
                "{}, which an external package provides, so it would be merged into the root \
                 module",
                Origin::Namespace(root_package)
            ),
            Fix::ChooseNamespaceOrPackage,
        ));
    }
    Ok(())
}

/// Fails when two of the generated `files` would be the same file, naming the
/// namespace or package each comes from.
///
/// Paths that differ only in case count as the same file, whatever the file
/// system the check runs on: they are one file on a case-insensitive file
/// system (macOS, Windows), where one module would overwrite the other, and
/// the output should not depend on where it is generated.
pub(crate) fn check_files<'a>(
    language: &str,
    files: impl IntoIterator<Item = (Origin<'a>, String)>,
) -> io::Result<()> {
    let mut written = BTreeMap::<String, (Origin, String)>::new();
    for (origin, file) in files {
        let Some(existing) = written.insert(file.to_lowercase(), (origin, file.clone())) else {
            continue;
        };
        // Name the namespace first: the root package is the one to keep.
        let ((subject, subject_file), (collider, collider_file)) =
            if matches!(origin, Origin::RootPackage(_)) {
                (existing, (origin, file))
            } else {
                ((origin, file), existing)
            };
        let fix = collider.choose_namespace();
        let collider = if subject_file == collider_file {
            collider.to_string()
        } else {
            format!(
                "{collider}, whose `{collider_file}` is the same file on a case-insensitive file \
                 system"
            )
        };
        return Err(error(
            language,
            format_args!("{subject} is written to `{subject_file}`"),
            collider,
            fix,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_subject_the_collider_and_the_fix() {
        let kv = QualifiedTypeName::root("Kv".to_string());
        let error = error(
            "Swift",
            format_args!("{} is the module `Kv`", Origin::Namespace("kv")),
            TypeName(&kv),
            Fix::RenameType,
        );

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(
            error.to_string(),
            "Swift: namespace \"kv\" is the module `Kv`, the same as type `Kv` in the root \
             namespace. Rename the type with `#[facet(rename = \"...\")]` or choose a different \
             namespace"
        );
    }
}
