//! Project scaffolding — writes a ready-to-build TypeScript project to disk.
//!
//! The [`Installer`] is the final stage of the TypeScript generation pipeline.
//! While [`TypeScriptCodeGenerator`] produces the *contents* of a single source file,
//! the installer is responsible for the surrounding project structure:
//!
//! 1. **Runtime files** — copies the serde and/or bincode runtime `.ts`
//!    sources into the output directory, adapting file names and import paths
//!    using extensionless imports (`index.ts` entry points, `.ts` stripped
//!    from import paths).
//!
//! 2. **Per-module source files** — splits the registry by namespace (via
//!    [`module::split`]) and calls [`TypeScriptCodeGenerator`] once per namespace,
//!    writing each to its own `.ts` file. TypeScript has no `namespace`
//!    keyword here — the crate's namespace concept maps to **ES modules**
//!    (separate `.ts` files), and cross-module type references use
//!    `import * as Namespace` wildcard imports with `Namespace.Type` syntax.
//!
//! 3. **`package.json`** — generates an NPM manifest with dependencies
//!    (external packages as `file:` paths or versioned registry references)
//!    and devDependencies (`typescript`).

use std::{
    collections::BTreeMap,
    fs::{File, create_dir_all},
    io::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde_json::{Value, json};

use std::collections::BTreeSet;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, Error, ExternalPackage, ExternalPackages, PackageLocation,
        SERDE_NAMESPACE, SourceInstaller, module,
        plugin::EmitterPlugin,
        typescript::{TypeScript, TypeScriptCodeGenerator},
    },
};

/// Installer for generated source files in TypeScript.
///
/// # Examples
///
/// ```rust
/// use facet_generate::generation::typescript;
///
/// let output_dir = std::path::PathBuf::from("output");
/// let installer = typescript::Installer::new("my-package", &output_dir);
/// ```
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    external_packages: ExternalPackages,
    encoding: Encoding,
    plugins: Vec<Arc<dyn EmitterPlugin<TypeScript>>>,
}

impl Installer {
    /// Create a new installer for the given package name and output directory.
    ///
    /// Use the builder methods [`encoding`](Self::encoding) and
    /// [`external_packages`](Self::external_packages) to configure, then call
    /// [`generate`](Self::generate) to produce the output.
    #[must_use]
    pub fn new(package_name: &str, install_dir: impl AsRef<Path>) -> Self {
        Self {
            package_name: package_name.to_string(),
            install_dir: install_dir.as_ref().to_path_buf(),
            external_packages: ExternalPackages::new(),
            encoding: Encoding::default(),
            plugins: vec![],
        }
    }

    /// Set the encoding for serialization/deserialization.
    ///
    /// When set to anything other than [`Encoding::None`], the appropriate
    /// runtimes (serde + encoding-specific) are installed automatically by
    /// [`generate`](Self::generate).
    #[must_use]
    pub const fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Add a plugin to be used during code generation.
    ///
    /// When plugins are added explicitly, they take priority over the
    /// [`encoding`](Self::encoding) setting.
    #[must_use]
    pub fn plugin(mut self, plugin: impl Into<Arc<dyn EmitterPlugin<TypeScript>>>) -> Self {
        self.plugins.push(plugin.into());
        self
    }

    /// Set external packages to reference.
    #[must_use]
    pub fn external_packages(mut self, packages: &[ExternalPackage]) -> Self {
        self.external_packages = packages
            .iter()
            .map(|d| (d.for_namespace.clone(), d.clone()))
            .collect();
        self
    }

    /// Generate all code for the given registry.
    ///
    /// This method:
    /// 1. Installs the appropriate runtimes based on the configured encoding
    /// 2. Splits the registry by namespace and installs each module
    /// 3. Writes the package manifest
    ///
    /// # Errors
    ///
    /// Returns an error if any file operation or code generation step fails.
    pub fn generate(mut self, registry: &Registry) -> Result<(), Error> {
        // Build a lang tag to get the active plugins, then use them to install
        // runtime files (replacing the old encoding-based install_serde/bincode calls).
        let mut config = CodeGeneratorConfig::new(self.package_name.clone());
        config.update_from(registry);
        // Resolve plugins: explicit plugins take priority over encoding
        if self.plugins.is_empty() {
            self.plugins = match self.encoding {
                Encoding::Bincode => vec![Arc::new(crate::generation::bincode::BincodePlugin)
                    as Arc<dyn EmitterPlugin<TypeScript>>],
                Encoding::Json => vec![Arc::new(crate::generation::json::JsonPlugin)
                    as Arc<dyn EmitterPlugin<TypeScript>>],
                Encoding::None => vec![],
            };
        }

        let lang = {
            let mut base = TypeScript::new(&config, registry);
            for p in &self.plugins {
                base = base.with_plugin(p.clone());
            }
            base
        };

        if !self.external_packages.contains_key(SERDE_NAMESPACE) {
            let mut written: BTreeSet<String> = BTreeSet::new();
            for plugin in lang.plugins() {
                for file in plugin.runtime_files() {
                    if written.insert(file.relative_path.clone()) {
                        let dest = self.install_dir.join(&file.relative_path);
                        if let Some(parent) = dest.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&dest, &file.contents)?;
                    }
                }
            }
        }

        // Split by namespace and install each module
        for (m, module_registry) in module::split(&self.package_name, registry) {
            let config = m.config().clone();
            self.install_module(&config, &module_registry)?;
        }

        // Write the package manifest
        let package_name = self.package_name.clone();
        self.install_manifest(&package_name)?;

        Ok(())
    }

    /// Installs the serde TypeScript runtime sources into the output directory.
    ///
    /// Delegates to the JSON plugin's [`runtime_files`](crate::generation::plugin::EmitterPlugin::runtime_files)
    /// which embeds the serde sources via `include_dir!`.  Most callers should
    /// prefer [`generate`](Self::generate).
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O fails.
    pub fn install_serde_runtime(&mut self) -> Result<(), Error> {
        let config = CodeGeneratorConfig::new(self.package_name.clone());
        let lang = TypeScript::new(&config, &BTreeMap::default())
            .with_plugin(std::sync::Arc::new(crate::generation::json::JsonPlugin));
        for plugin in lang.plugins() {
            for file in plugin.runtime_files() {
                let dest = self.install_dir.join(&file.relative_path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&dest, &file.contents)?;
            }
        }
        Ok(())
    }

    /// Installs the bincode TypeScript runtime sources into the output directory.
    ///
    /// Delegates to the bincode plugin's `runtime_files`, writing only the
    /// `bincode/` files.  Most callers should prefer [`generate`](Self::generate).
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O fails.
    pub fn install_bincode_runtime(&self) -> Result<(), Error> {
        let config = CodeGeneratorConfig::new(self.package_name.clone());
        let lang = TypeScript::new(&config, &BTreeMap::default()).with_plugin(std::sync::Arc::new(
            crate::generation::bincode::BincodePlugin,
        ));
        for plugin in lang.plugins() {
            for file in plugin
                .runtime_files()
                .into_iter()
                .filter(|f| f.relative_path.starts_with("bincode/"))
            {
                let dest = self.install_dir.join(&file.relative_path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&dest, &file.contents)?;
            }
        }
        Ok(())
    }

    /// Produce the contents of a `package.json` manifest.
    ///
    /// Dependencies are derived from external packages: `Path` locations
    /// become `file:` references, `Url` locations use the extracted package
    /// name with an optional version string. `typescript` is always added as
    /// a devDependency.
    #[must_use]
    pub fn make_manifest(&self, package_name: &str) -> Value {
        let mut manifest = json!({
            "name": package_name,
            "version": "0.1.0"
        });

        // Add dependencies if we have external packages
        if !self.external_packages.is_empty() {
            let mut dependencies = BTreeMap::new();

            for external_package in self.external_packages.values() {
                let (name, version) = match &external_package.location {
                    PackageLocation::Path(path) => (
                        external_package.for_namespace.clone(),
                        format!("file:{path}"),
                    ),
                    PackageLocation::Url(url) => (
                        {
                            // Extract package name from URL
                            let parts: Vec<&str> = url.split('/').collect();
                            if parts.len() >= 2 && parts[parts.len() - 2].starts_with('@') {
                                // Scoped package: @scope/package-name
                                format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
                            } else if let Some(last_segment) = parts.last() {
                                // Regular package: package-name
                                (*last_segment).to_string()
                            } else {
                                url.clone()
                            }
                        },
                        external_package
                            .version
                            .clone()
                            .unwrap_or_else(|| "*".to_string()),
                    ),
                };
                dependencies.insert(name, version);
            }

            manifest["dependencies"] = json!(dependencies);
        }

        // Always add devDependencies
        manifest["devDependencies"] = json!({
            "typescript": "^5.8.3"
        });

        manifest
    }
}

impl SourceInstaller for Installer {
    /// Generate a single `.ts` source file for one namespace.
    ///
    /// The file is written as `<namespace>.ts` directly in the install
    /// directory. Namespaces that correspond to external packages are skipped
    /// — their types are imported rather than generated.
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> Result<(), Error> {
        let skip_module = self.external_packages.contains_key(config.module_name());
        if skip_module {
            return Ok(());
        }
        create_dir_all(&self.install_dir)?;
        let module_name = config.module_name();
        let file_name = self.install_dir.join(format!("{module_name}.ts"));
        let mut file = File::create(file_name)?;

        // Update config with external packages from installer
        let mut updated_config = config.clone();
        updated_config.external_packages = self.external_packages.clone();

        let generator = TypeScriptCodeGenerator::new(&updated_config)
            .with_encoding(self.encoding)
            .with_plugins(self.plugins.clone());
        generator.output(&mut file, registry)?;

        Ok(())
    }

    /// Write `package.json` to the output directory.
    fn install_manifest(&self, package_name: &str) -> std::result::Result<(), Error> {
        let manifest = self.make_manifest(package_name);
        let manifest = serde_json::to_string_pretty(&manifest)?;

        let manifest_path = self.install_dir.join("package.json");
        let mut file = File::create(manifest_path)?;
        file.write_all(manifest.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
