#![allow(clippy::missing_errors_doc)]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use derive_builder::Builder;
use serde::Serialize;

use crate::{
    Registry,
    reflection::format::{Format, FormatHolder},
};

/// Code generation options meant to be supported by all languages.
#[derive(Clone, Debug, Serialize)]
pub struct CodeGeneratorConfig {
    pub module_name: String,
    pub encoding: Encoding,
    pub external_definitions: ExternalDefinitions,
    pub external_packages: ExternalPackages,
    pub comments: DocComments,
    pub custom_code: CustomCode,
    pub c_style_enums: bool,
    pub package_manifest: bool,
    pub has_bigint: bool,
}

#[derive(Clone, Copy, Default, Debug, PartialOrd, Ord, PartialEq, Eq, Serialize)]
pub enum Encoding {
    #[default]
    None,
    Json,
    Bincode,
    Bcs,
}

/// Track type definitions provided by other modules (key = <module>, value = <type names>).
pub type ExternalDefinitions =
    std::collections::BTreeMap</* module */ String, /* type names */ Vec<String>>;

/// Track locations for imports of external packages (key = <module>, value = <import from>).
pub type ExternalPackages =
    std::collections::BTreeMap</* module */ String, /* import from */ ExternalPackage>;

/// Track documentation to be attached to particular definitions.
pub type DocComments =
    std::collections::BTreeMap</* qualified name */ Vec<String>, /* comment */ String>;

/// Track custom code to be added to particular definitions (use with care!).
pub type CustomCode = std::collections::BTreeMap<
    /* qualified name */ Vec<String>,
    /* custom code */ String,
>;

/// How to copy generated source code and available runtimes for a given language.
pub trait SourceInstaller {
    type Error;

    /// Create a module exposing the container types contained in the registry.
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error>;

    /// Install the serde runtime.
    fn install_serde_runtime(&mut self) -> std::result::Result<(), Self::Error>;

    /// Install the bincode runtime.
    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error>;

    /// Install the Libra Canonical Serialization (BCS) runtime.
    fn install_bcs_runtime(&self) -> std::result::Result<(), Self::Error>;

    /// Install a package manifest.
    fn install_manifest(
        &self,
        _module_name: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

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
            c_style_enums: false,
            package_manifest: true,
            has_bigint: false,
        }
    }

    #[must_use]
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    /// for Java: updates the module name to be a child of the specified parent
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
    pub fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    #[must_use]
    pub fn has_encoding(&self) -> bool {
        self.encoding != Encoding::None
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
    pub fn with_custom_code(mut self, code: CustomCode) -> Self {
        self.custom_code = code;
        self
    }

    /// Generate C-style enums (without variant data) as the target language
    /// native enum type in supported languages.
    #[must_use]
    pub fn with_c_style_enums(mut self, c_style_enums: bool) -> Self {
        self.c_style_enums = c_style_enums;
        self
    }

    /// Generate a package manifest file for the target language.
    #[must_use]
    pub fn with_package_manifest(mut self, package_manifest: bool) -> Self {
        self.package_manifest = package_manifest;
        self
    }

    pub fn update_from(&mut self, registry: &Registry) {
        for format in registry.values() {
            format
                .visit(&mut |f| {
                    match f {
                        Format::I128 | Format::U128 => self.has_bigint = true,
                        _ => (),
                    }
                    Ok(())
                })
                .unwrap();
        }
    }
}

impl Encoding {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Encoding::None => "none",
            Encoding::Json => "json",
            Encoding::Bincode => "bincode",
            Encoding::Bcs => "bcs",
        }
    }
}

/// Configuration for foreign type generation.
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
    /// Whether to add runtimes to the generated types.
    #[builder(default = false, setter(custom))]
    pub add_runtimes: bool,
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
    pub fn add_runtimes(&mut self) -> &mut Self {
        self.add_runtimes = Some(true);
        self
    }

    #[must_use]
    pub fn add_extensions(&mut self) -> &mut Self {
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

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageLocation {
    /// Either a local file path or, for Java, a dot-separated package name.
    Path(String),
    // The URL of a remote package.
    Url(String),
}

#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExternalPackage {
    /// The namespace as specified in `#[facet(namespace = "namespace")]`.
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
}
