#![allow(clippy::missing_errors_doc)]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Configuration that controls code-generation behaviour independent of the
//! target language.
//!
//! The most important knob is [`Encoding`]: it decides whether generated types
//! are plain data classes (`None`) or also get `serialize` / `deserialize`
//! methods (`Json` or `Bincode`). When serialization is enabled, the installer
//! copies the corresponding runtime support files alongside the generated code.
//!
//! Cross-module relationships are handled by [`ExternalDefinitions`] (which
//! types live in other modules) and [`ExternalPackages`] (where to import them
//! from), so generators can emit `import` / `using` statements instead of
//! redeclaring types.
//!
//! [`Feature`] flags are discovered automatically by
//! [`CodeGeneratorConfig::update_from`] scanning the registry — they tell the
//! installer which container-type helpers (`ListOfT`, `MapOfT`, …) to include.
//!
//! There are two configuration levels:
//!
//! - [`Config`] / [`ConfigBuilder`] — the public API entry point (package
//!   name, output directory, encoding, external packages).
//! - [`CodeGeneratorConfig`] — the internal, per-module config that generators
//!   and [`Emitter`](super::Emitter) implementations receive. Generators copy
//!   the [`Encoding`] from here into the language tag so that emitters can
//!   access it via the `lang` parameter without needing the full config.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use derive_builder::Builder;
use serde::Serialize;
use thiserror::Error;

use crate::{
    Registry,
    generation::indent::IndentConfig,
    reflection::format::{Format, FormatHolder, Namespace},
};

/// Code generation options meant to be supported by all languages.
#[derive(Clone, Debug)]
#[expect(
    deprecated,
    reason = "CodeGeneratorConfig still contains the deprecated CustomCode field for backwards compatibility"
)]
pub struct CodeGeneratorConfig {
    pub module_name: String,
    pub encoding: Encoding,
    pub external_definitions: ExternalDefinitions,
    pub external_packages: ExternalPackages,
    pub comments: DocComments,
    /// **Deprecated since 0.16.0:** `custom_code` was only used by the Java generator, which is deprecated. Use Kotlin instead.
    pub custom_code: CustomCode,
    pub package_manifest: bool,
    pub features: BTreeSet<Feature>,
    /// The indentation style used when writing generated source code.
    ///
    /// Defaults to `IndentConfig::Space(4)`. Pass a custom value via
    /// [`with_indent`](Self::with_indent) if a different style is required
    /// (e.g. tabs, or a different space width).
    pub indent: IndentConfig,
    /// Which primitive/leaf format types are used in the registry.
    /// Populated by `update_from`. Used by TypeScript to emit type aliases.
    pub used_format_types: BTreeSet<String>,
    /// External namespaces actually referenced via `Format::TypeName` in the registry.
    /// Populated by `update_from`. Used to generate namespace import statements.
    pub referenced_namespaces: BTreeSet<String>,
}

/// The wire format to target when generating serialization code.
///
/// - `None` — generate type declarations only, with no serialization support.
/// - `Json` — generate JSON `serialize` / `deserialize` methods and install
///   the JSON serde runtime.
/// - `Bincode` — generate binary `serialize` / `bincodeDeserialize` methods
///   and install the Bincode runtime.
#[derive(Clone, Copy, Default, Debug, PartialOrd, Ord, PartialEq, Eq, Serialize)]
pub enum Encoding {
    #[default]
    None,
    Json,
    Bincode,
}

/// Container or leaf types in the registry that need a runtime support file
/// installed alongside the generated code.
///
/// Discovered automatically by [`CodeGeneratorConfig::update_from`] and
/// consumed by [`SourceInstaller`] implementations.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize)]
pub enum Feature {
    BigInt,
    Bytes,
    ListOfT,
    MapOfT,
    OptionOfT,
    SetOfT,
    TupleArray,
}

impl Encoding {
    #[must_use]
    pub fn is_none(self) -> bool {
        self == Self::None
    }

    #[must_use]
    pub fn is_json(self) -> bool {
        self == Self::Json
    }

    #[must_use]
    pub fn is_bincode(self) -> bool {
        self == Self::Bincode
    }
}

/// Track type definitions provided by other modules (key = `module`, value = `type names`).
pub type ExternalDefinitions =
    BTreeMap</* module */ String, /* type names */ Vec<String>>;

/// Track locations for imports of external packages (key = `module`, value = `import from`).
pub type ExternalPackages =
    BTreeMap</* module */ String, /* import from */ ExternalPackage>;

/// Track documentation to be attached to particular definitions.
pub type DocComments = BTreeMap</* qualified name */ Vec<String>, /* comment */ String>;

/// Track custom code to be added to particular definitions (use with care!).
#[deprecated(
    since = "0.16.0",
    note = "custom_code was only used by the Java generator, which is deprecated. Use Kotlin instead."
)]
pub type CustomCode =
    BTreeMap</* qualified name */ Vec<String>, /* custom code */ String>;

/// Errors that can occur during code generation and installation.
#[derive(Debug, Error)]
pub enum Error {
    /// An I/O error occurred while reading or writing files.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// A runtime file contained invalid UTF-8.
    #[error("invalid UTF-8 in runtime file: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// JSON serialization failed (e.g. when writing a TypeScript `package.json`).
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Writes generated source code and runtime support files to disk.
///
/// Each target language provides its own implementation. The installer is
/// the third layer of the pipeline — after [`CodeGenerator`](super::CodeGenerator)
/// produces the source text and [`Emitter`](super::Emitter) renders each
/// AST node, the installer places everything into the output directory and
/// copies any runtime files required by the chosen [`Encoding`].
pub trait SourceInstaller {
    /// Create a module exposing the container types contained in the registry.
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Error>;

    /// Install the serde runtime.
    fn install_serde_runtime(&mut self) -> std::result::Result<(), Error>;

    /// Install the bincode runtime.
    fn install_bincode_runtime(&self) -> std::result::Result<(), Error>;

    /// Install a package manifest.
    fn install_manifest(&self, _module_name: &str) -> std::result::Result<(), Error> {
        Ok(())
    }
}

#[expect(
    deprecated,
    reason = "impl references the deprecated CustomCode type and with_custom_code method"
)]
impl CodeGeneratorConfig {
    /// Default config for the given module name.
    #[must_use]
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            encoding: Encoding::default(),
            external_definitions: BTreeMap::new(),
            external_packages: BTreeMap::new(),
            comments: BTreeMap::new(),
            custom_code: BTreeMap::new(),
            package_manifest: true,
            features: BTreeSet::new(),
            used_format_types: BTreeSet::new(),
            referenced_namespaces: BTreeSet::new(),
            indent: IndentConfig::Space(4),
        }
    }

    #[must_use]
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    /// for Kotlin: updates the module name to be a child of the specified parent
    #[must_use]
    pub fn with_parent(mut self, parent: &str) -> Self {
        if parent == self.module_name() {
            return self;
        }

        self.module_name = format!("{}.{}", parent, self.module_name());
        self
    }

    /// Which encoding to use.
    #[must_use]
    pub const fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Which indentation style to use when writing generated source code.
    #[must_use]
    pub const fn with_indent(mut self, indent: IndentConfig) -> Self {
        self.indent = indent;
        self
    }

    #[must_use]
    pub fn has_encoding(&self) -> bool {
        !self.encoding.is_none()
    }

    /// Container names provided by other modules.
    #[must_use]
    pub fn with_external_definitions(mut self, external_definitions: ExternalDefinitions) -> Self {
        self.external_definitions = external_definitions;
        self
    }

    /// Comments attached to particular entity.
    #[must_use]
    pub fn with_comments(mut self, mut comments: DocComments) -> Self {
        // Make sure comments end with a (single) newline.
        for comment in comments.values_mut() {
            *comment = format!("{}\n", comment.trim());
        }
        self.comments = comments;
        self
    }

    /// Custom code attached to particular entity.
    #[must_use]
    #[deprecated(
        since = "0.16.0",
        note = "custom_code was only used by the Java generator, which is deprecated. Use Kotlin instead."
    )]
    pub fn with_custom_code(mut self, code: CustomCode) -> Self {
        self.custom_code = code;
        self
    }

    /// Generate a package manifest file for the target language.
    #[must_use]
    pub const fn with_package_manifest(mut self, package_manifest: bool) -> Self {
        self.package_manifest = package_manifest;
        self
    }

    /// Updates a config with features present in the specified registry.
    ///
    /// # Panics
    ///
    /// Panics if the registry is not properly formatted.
    pub fn update_from(&mut self, registry: &Registry) {
        for format in registry.values() {
            format
                .visit(&mut |f| {
                    match f {
                        Format::I128 | Format::U128 => {
                            self.features.insert(Feature::BigInt);
                        }
                        Format::Bytes => {
                            self.features.insert(Feature::Bytes);
                        }
                        Format::Seq(..) => {
                            self.features.insert(Feature::ListOfT);
                        }
                        Format::Option(..) => {
                            self.features.insert(Feature::OptionOfT);
                        }
                        Format::Set(..) => {
                            self.features.insert(Feature::SetOfT);
                        }
                        Format::Map { .. } => {
                            self.features.insert(Feature::MapOfT);
                        }
                        Format::TupleArray { .. } => {
                            self.features.insert(Feature::TupleArray);
                        }
                        _ => (),
                    }

                    // Track external namespaces actually referenced in format types.
                    if let Format::TypeName(qualified_name) = f
                        && let Namespace::Named(ns) = &qualified_name.namespace
                        && ns != &self.module_name
                    {
                        self.referenced_namespaces.insert(ns.clone());
                    }

                    // Also record the leaf format type key (used by TypeScript for type aliases).
                    let format_key = match f {
                        Format::Unit => "unit",
                        Format::Bool => "bool",
                        Format::I8 => "int8",
                        Format::I16 => "int16",
                        Format::I32 => "int32",
                        Format::I64 => "int64",
                        Format::I128 => "int128",
                        Format::U8 => "uint8",
                        Format::U16 => "uint16",
                        Format::U32 => "uint32",
                        Format::U64 => "uint64",
                        Format::U128 => "uint128",
                        Format::F32 => "float32",
                        Format::F64 => "float64",
                        Format::Char => "char",
                        Format::Str => "str",
                        Format::Bytes => "bytes",
                        Format::Option(_) => "option",
                        Format::Seq(_) | Format::Set(_) => "seq",
                        Format::Map { .. } => "map",
                        Format::Tuple(_) => "tuple",
                        Format::TupleArray { .. } => "list_tuple",
                        Format::TypeName(_) | Format::Variable(_) => "",
                    };
                    if !format_key.is_empty() {
                        self.used_format_types.insert(format_key.to_string());
                    }

                    Ok(())
                })
                .expect("failed to parse registry");
        }

        for name in registry.keys() {
            if let Namespace::Named(ns) = &name.namespace
                && ns != &self.module_name
            {
                let entry = self.external_definitions.entry(ns.to_owned()).or_default();
                entry.push(name.name.clone());
            }
        }
    }
}

impl Encoding {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Json => "json",
            Self::Bincode => "bincode",
        }
    }
}

/// Public API entry point for configuring a generation run.
///
/// Use [`Config::builder`] to create one, then pass it to a language-specific
/// `generate` function.
#[derive(Default, Builder)]
#[builder(
    custom_constructor,
    create_empty = "empty",
    build_fn(private, name = "fallible_build")
)]
pub struct Config {
    /// The name of the package to generate.
    #[builder(setter(into))]
    pub package_name: String,
    /// The directory to generate the types in.
    #[builder(setter(into))]
    pub out_dir: PathBuf,
    /// External packages to reference.
    #[builder(default = vec![], setter(each(name = "reference")))]
    pub external_packages: Vec<ExternalPackage>,
    /// The encoding to use for serialization/deserialization.
    /// When set to anything other than `Encoding::None`, the appropriate
    /// runtimes will be installed automatically.
    #[builder(default, setter(custom))]
    pub encoding: Encoding,
    /// Whether to add extensions to the generated types.
    #[builder(default = false, setter(custom))]
    pub add_extensions: bool,
}

impl Config {
    pub fn builder(name: &str, out_dir: impl AsRef<Path>) -> ConfigBuilder {
        ConfigBuilder {
            package_name: Some(name.to_string()),
            out_dir: Some(out_dir.as_ref().to_path_buf()),
            ..ConfigBuilder::empty()
        }
    }
}

impl ConfigBuilder {
    #[must_use]
    pub const fn encoding(&mut self, encoding: Encoding) -> &mut Self {
        self.encoding = Some(encoding);
        self
    }

    #[must_use]
    pub const fn add_extensions(&mut self) -> &mut Self {
        self.add_extensions = Some(true);
        self
    }

    /// # Panics
    /// If any required fields are not initialized.
    #[must_use]
    pub fn build(&self) -> Config {
        self.fallible_build()
            .expect("All required fields were initialized")
    }
}

/// Where an external package can be found.
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageLocation {
    /// Either a local file path or, for Kotlin, a dot-separated package name.
    Path(String),
    // The URL of a remote package.
    Url(String),
}

/// A reference to a package that provides types from another namespace,
/// so the generator can emit the correct import statements.
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExternalPackage {
    /// The namespace as specified in `#[facet(fg::namespace = "namespace")]`.
    pub for_namespace: String,
    /// The location of the package.
    pub location: PackageLocation,
    /// The name of the module, if you are importing one from a package.
    /// e.g. in TypeScript: `import { Foo } from 'package_name/module_name';`
    pub module_name: Option<String>,
    /// An optional string to specify the version of a published package.
    pub version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_parent() {
        let root_package = "root";
        let child_package = "child";

        let root_config = CodeGeneratorConfig::new(root_package.to_string());

        let actual = root_config.with_parent(root_package).module_name;
        let expected = root_package;
        assert_eq!(&actual, expected);

        let actual = CodeGeneratorConfig::new(child_package.to_string())
            .with_parent(root_package)
            .module_name;
        let expected = format!("{root_package}.{child_package}");
        assert_eq!(&actual, &expected);
    }

    #[test]
    fn config_builder_populates_external_packages() {
        let config = Config::builder("MyPackage", "/tmp/out")
            .reference(ExternalPackage {
                for_namespace: "serde".to_string(),
                location: PackageLocation::Path("../Serde".to_string()),
                module_name: None,
                version: None,
            })
            .reference(ExternalPackage {
                for_namespace: "other".to_string(),
                location: PackageLocation::Path("../Other".to_string()),
                module_name: None,
                version: None,
            })
            .build();

        assert_eq!(config.external_packages.len(), 2);
        assert_eq!(config.external_packages[0].for_namespace, "serde");
        assert_eq!(config.external_packages[1].for_namespace, "other");
    }

    #[test]
    fn config_builder_defaults_external_packages_to_empty() {
        let config = Config::builder("MyPackage", "/tmp/out").build();
        assert!(config.external_packages.is_empty());
    }
}
