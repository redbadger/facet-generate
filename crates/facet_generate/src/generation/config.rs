#![allow(clippy::missing_errors_doc)]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Configuration that controls code-generation behaviour independent of the
//! target language.
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
//!   name, output directory, external packages).
//! - [`CodeGeneratorConfig`] — the internal, per-module config that generators
//!   and [`Emitter`](super::Emitter) implementations receive.

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
    reflection::format::{
        ContainerFormat, Format, FormatHolder, Namespace, QualifiedTypeName, VariantFormat,
    },
};

/// Code generation options meant to be supported by all languages.
#[derive(Clone, Debug)]
pub struct CodeGeneratorConfig {
    pub module_name: String,
    /// The package this module was nested under by [`with_parent`](Self::with_parent),
    /// if any.
    ///
    /// Kept because `module_name` alone cannot be decomposed: once a namespaced
    /// module becomes `com.example.auth`, nothing in the string says whether
    /// `auth` is a namespace or the last segment of the root package. Kotlin
    /// and C# need the answer to qualify a type living in a *sibling*
    /// namespace — that path is rooted at the parent, not at this module — and
    /// C# to qualify a ROOT type too.
    ///
    /// The TypeScript installer sets it for a namespaced module without
    /// renaming the module, whose name is also its file's, so that the module
    /// can import the root types it references from the root package.
    pub parent: Option<String>,
    pub external_definitions: ExternalDefinitions,
    pub external_packages: ExternalPackages,
    pub comments: DocComments,
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
    /// Every enum whose variants are all unit variants, in whichever module
    /// of the registry it lives.
    ///
    /// Populated by `update_from` for the module's own types and by
    /// [`module::split`](super::module::split) for every other module's, so an
    /// enum from another namespace is known too. Keyed by qualified name, and
    /// the generators respell the keys the way they rewrite type references,
    /// so query it with [`is_unit_enum`](Self::is_unit_enum) and the name as
    /// it appears in the registry the emitter sees. Used by the C# Bincode
    /// plugin to dispatch enum-typed fields through
    /// `{TypeName}Bincode.Serialize(...)` helpers.
    pub unit_variant_enums: BTreeSet<QualifiedTypeName>,
    /// Every enum type, in whichever module of the registry it lives.
    ///
    /// Filled and keyed like [`unit_variant_enums`](Self::unit_variant_enums);
    /// query it with [`is_enum`](Self::is_enum). Used by TypeScript
    /// bincode/json plugins to branch `Format::TypeName` serialization: enums
    /// use standalone `serializeX(value, serializer)` functions while structs
    /// use `.serialize(serializer)`.
    pub enum_type_names: BTreeSet<QualifiedTypeName>,
    /// Every type name the generated module declares, in raw registry spelling:
    /// each container name, plus the variant names of every enum that has at
    /// least one non-unit variant (Kotlin and C# nest those as classes and
    /// records).
    ///
    /// Populated by `update_from`. Used to decide whether a builtin type name
    /// the generated code would otherwise write bare (`Set`, `Map`, `String`,
    /// …) is shadowed by a declaration and must be written fully qualified.
    pub declared_type_names: BTreeSet<String>,
    /// Every container in the registry this module was split from, in raw
    /// registry spelling (unlike the enum sets, these keys are not respelled).
    ///
    /// Populated by `update_from` and [`module::split`](super::module::split).
    /// Used where a declaration in *another* module is in scope in this one —
    /// a C# namespace sees the types of its enclosing namespace, and a Swift
    /// module sees those of every module it imports — to decide whether it
    /// shadows a builtin type name.
    pub registry_type_names: BTreeSet<QualifiedTypeName>,
}

/// Container or leaf types in the registry that need a runtime support file
/// installed alongside the generated code.
///
/// Discovered automatically by [`CodeGeneratorConfig::update_from`] and
/// consumed by [`SourceInstaller`] implementations.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize)]
#[non_exhaustive]
pub enum Feature {
    BigInt,
    Bytes,
    ListOfT,
    MapOfT,
    OptionOfT,
    SetOfT,
    TupleArray,
    Uuid,
}

/// Track type definitions provided by other modules (key = `module`, value = `type names`).
pub type ExternalDefinitions =
    BTreeMap</* module */ String, /* type names */ Vec<String>>;

/// Track locations for imports of external packages (key = `module`, value = `import from`).
pub type ExternalPackages =
    BTreeMap</* module */ String, /* import from */ ExternalPackage>;

/// Track documentation to be attached to particular definitions.
pub type DocComments = BTreeMap</* qualified name */ Vec<String>, /* comment */ String>;

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
/// writes any runtime files declared by the active plugins.
pub trait SourceInstaller {
    /// Create a module exposing the container types contained in the registry.
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Error>;

    /// Install a package manifest.
    fn install_manifest(&self, _module_name: &str) -> std::result::Result<(), Error> {
        Ok(())
    }
}

impl CodeGeneratorConfig {
    /// Default config for the given module name.
    #[must_use]
    pub const fn new(module_name: String) -> Self {
        Self {
            module_name,
            parent: None,
            external_definitions: BTreeMap::new(),
            external_packages: BTreeMap::new(),
            comments: BTreeMap::new(),
            package_manifest: true,
            features: BTreeSet::new(),
            used_format_types: BTreeSet::new(),
            referenced_namespaces: BTreeSet::new(),
            unit_variant_enums: BTreeSet::new(),
            enum_type_names: BTreeSet::new(),
            declared_type_names: BTreeSet::new(),
            registry_type_names: BTreeSet::new(),
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

        self.parent = Some(parent.to_string());
        self.module_name = format!("{}.{}", parent, self.module_name());
        self
    }

    /// The package that sibling namespaces hang off.
    ///
    /// The parent when this module was nested under one, and the module itself
    /// otherwise — so a root module and its namespaced children agree on where
    /// namespace packages live.
    #[must_use]
    pub fn root_package(&self) -> &str {
        self.parent.as_deref().unwrap_or(&self.module_name)
    }

    /// Which indentation style to use when writing generated source code.
    #[must_use]
    pub const fn with_indent(mut self, indent: IndentConfig) -> Self {
        self.indent = indent;
        self
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
                        Format::Uuid => {
                            self.features.insert(Feature::Uuid);
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
                        Format::Uuid => "uuid",
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

        for (name, format) in registry {
            if let Namespace::Named(ns) = &name.namespace
                && ns != &self.module_name
            {
                let entry = self.external_definitions.entry(ns.to_owned()).or_default();
                entry.push(name.name.clone());
            }

            self.declared_type_names.insert(name.name.clone());

            if let ContainerFormat::Enum(variants, _, _) = format
                && !is_unit_only(format)
            {
                // A data-carrying enum is emitted as a nested class or
                // record per variant, so each variant name is a declared
                // type too.
                for variant in variants.values() {
                    self.declared_type_names.insert(variant.name.clone());
                }
            }
        }

        self.index_types(registry);
    }

    /// Records every container of `registry`, and which of them are enums, so
    /// that a module knows about types declared in the other modules too.
    ///
    /// [`module::split`](super::module::split) calls this with the whole
    /// registry for each module it produces.
    pub(crate) fn index_types(&mut self, registry: &Registry) {
        for (name, format) in registry {
            self.registry_type_names.insert(name.clone());
            if let ContainerFormat::Enum(..) = format {
                self.enum_type_names.insert(name.clone());
                if is_unit_only(format) {
                    self.unit_variant_enums.insert(name.clone());
                }
            }
        }
    }

    /// Respells the keys of the enum sets with `requalify`, the function the
    /// generator rewrites each type reference in `local` (this module's
    /// registry) with, so that [`is_enum`](Self::is_enum) and
    /// [`is_unit_enum`](Self::is_unit_enum) can be asked with the name the
    /// emitter sees.
    ///
    /// A rewrite can give a type in another module the spelling of one this
    /// module declares (a TypeScript namespaced module that does not know the
    /// root package writes both its own types and ROOT ones bare), and the
    /// bare name then means the local declaration, so the module's own types
    /// decide their spelling.
    pub(crate) fn requalify_enums(
        &mut self,
        local: &Registry,
        requalify: impl Fn(&Self, &QualifiedTypeName) -> QualifiedTypeName,
    ) {
        let respell = |set: &BTreeSet<QualifiedTypeName>| -> BTreeSet<QualifiedTypeName> {
            set.iter().map(|name| requalify(self, name)).collect()
        };
        let mut enums = respell(&self.enum_type_names);
        let mut unit_enums = respell(&self.unit_variant_enums);

        for (name, format) in local {
            let name = requalify(self, name);
            unit_enums.remove(&name);
            if let ContainerFormat::Enum(..) = format {
                if is_unit_only(format) {
                    unit_enums.insert(name.clone());
                }
                enums.insert(name);
            } else {
                enums.remove(&name);
            }
        }

        self.enum_type_names = enums;
        self.unit_variant_enums = unit_enums;
    }

    /// Whether `name` is an enum, in the spelling the emitter sees (see
    /// [`enum_type_names`](Self::enum_type_names)).
    ///
    /// A name from a format in [`EmitContext`](super::plugin::EmitContext) is
    /// already in that spelling; a registry-spelled one goes through the
    /// language's `requalify` first (for example
    /// [`csharp::requalify`](super::csharp::requalify)).
    #[must_use]
    pub fn is_enum(&self, name: &QualifiedTypeName) -> bool {
        self.enum_type_names.contains(name)
    }

    /// Whether `name` is an enum whose variants are all unit variants, in the
    /// spelling the emitter sees (see
    /// [`unit_variant_enums`](Self::unit_variant_enums)).
    ///
    /// As with [`is_enum`](Self::is_enum), a registry-spelled name goes
    /// through the language's `requalify` first.
    #[must_use]
    pub fn is_unit_enum(&self, name: &QualifiedTypeName) -> bool {
        self.unit_variant_enums.contains(name)
    }
}

/// Returns `true` for an enum whose variants are all unit variants.
fn is_unit_only(format: &ContainerFormat) -> bool {
    matches!(format, ContainerFormat::Enum(variants, _, _)
        if variants.values().all(|v| matches!(v.value, VariantFormat::Unit)))
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
    /// Swift only: the deployment targets the generated package declares.
    ///
    /// Each entry is a raw SPM platform expression — `".iOS(.v16)"` — and they
    /// are rendered in the order given as
    /// `platforms: [.iOS(.v16), .macOS(.v13)],`. Leave empty to omit the
    /// `platforms:` line, which leaves SPM on its own defaults.
    ///
    /// This is configuration rather than a plugin hook because the floor
    /// depends on the app the generated package is linked into, which no
    /// plugin can know.
    #[builder(default = vec![], setter(each(name = "platform", into)))]
    pub platforms: Vec<String>,
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
    use crate::reflection::format::{Doc, EnumTagging, Named, QualifiedTypeName};

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
    fn declared_type_names_includes_variants_of_data_carrying_enums_only() {
        let mut variants = BTreeMap::new();
        variants.insert(
            0u32,
            Named::new(&VariantFormat::Unit, "UnitOnly".to_string()),
        );
        let unit_enum = ContainerFormat::Enum(variants, EnumTagging::External, Doc::default());

        let mut variants = BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "Err".to_string()));
        variants.insert(
            1u32,
            Named::new(
                &VariantFormat::NewType(Box::new(Format::Str)),
                "Ok".to_string(),
            ),
        );
        let data_enum = ContainerFormat::Enum(variants, EnumTagging::External, Doc::default());

        let mut registry = Registry::new();
        registry.insert(
            QualifiedTypeName {
                namespace: Namespace::Root,
                name: "Set".to_string(),
            },
            ContainerFormat::UnitStruct(Doc::default()),
        );
        registry.insert(
            QualifiedTypeName {
                namespace: Namespace::Root,
                name: "UnitEnum".to_string(),
            },
            unit_enum,
        );
        registry.insert(
            QualifiedTypeName {
                namespace: Namespace::Root,
                name: "DataEnum".to_string(),
            },
            data_enum,
        );

        let mut config = CodeGeneratorConfig::new("test".to_string());
        config.update_from(&registry);

        let names: Vec<&str> = config
            .declared_type_names
            .iter()
            .map(String::as_str)
            .collect();
        // Container names, plus the variants of the data-carrying enum — but
        // not `UnitOnly`, which becomes an enum constant rather than a type.
        assert_eq!(names, ["DataEnum", "Err", "Ok", "Set", "UnitEnum"]);
    }

    fn unit_enum() -> ContainerFormat {
        let mut variants = BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "On".to_string()));
        ContainerFormat::Enum(variants, EnumTagging::External, Doc::default())
    }

    #[test]
    fn split_tells_every_module_about_the_enums_of_the_others() {
        let kit_presence = QualifiedTypeName::namespaced("kit".to_string(), "Presence".to_string());
        let root_presence = QualifiedTypeName::root("Presence".to_string());

        let mut registry = Registry::new();
        registry.insert(kit_presence.clone(), unit_enum());
        registry.insert(
            root_presence.clone(),
            ContainerFormat::UnitStruct(Doc::default()),
        );

        let modules = crate::generation::module::split("root", &registry);
        let root = modules
            .keys()
            .find(|module| module.config().module_name() == "root")
            .unwrap()
            .config();

        assert!(root.is_enum(&kit_presence));
        assert!(root.is_unit_enum(&kit_presence));
        // The same name in another namespace is a different type.
        assert!(!root.is_enum(&root_presence));
    }

    #[test]
    fn requalified_enum_names_defer_to_the_module_s_own_types() {
        let kit_presence = QualifiedTypeName::namespaced("kit".to_string(), "Presence".to_string());
        let root_presence = QualifiedTypeName::root("Presence".to_string());

        // A ROOT enum, and a struct of the same name in the module being
        // generated, whose references are respelled bare like ROOT ones.
        let mut local = Registry::new();
        local.insert(kit_presence, ContainerFormat::UnitStruct(Doc::default()));
        let mut config = CodeGeneratorConfig::new("kit".to_string());
        config.index_types(&Registry::from([(root_presence.clone(), unit_enum())]));
        config.update_from(&local);

        config.requalify_enums(&local, |config, name| match &name.namespace {
            Namespace::Named(namespace) if namespace == config.module_name() => {
                QualifiedTypeName::root(name.name.clone())
            }
            _ => name.clone(),
        });

        // A bare `Presence` in this module means the local struct.
        assert!(!config.is_enum(&root_presence));
        assert!(!config.is_unit_enum(&root_presence));
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

    #[test]
    fn config_builder_populates_platforms_in_order() {
        let config = Config::builder("MyPackage", "/tmp/out")
            .platform(".iOS(.v16)")
            .platform(".macOS(.v13)")
            .build();

        assert_eq!(config.platforms, vec![".iOS(.v16)", ".macOS(.v13)"]);
    }

    #[test]
    fn config_builder_defaults_platforms_to_empty() {
        let config = Config::builder("MyPackage", "/tmp/out").build();
        assert!(config.platforms.is_empty());
    }
}
