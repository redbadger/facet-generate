//! Project scaffolding — writes a ready-to-build TypeScript project to disk.
//!
//! The [`Installer`] is the final stage of the TypeScript generation pipeline.
//! While [`TypeScriptCodeGenerator`] produces the *contents* of a single source file,
//! the installer is responsible for the surrounding project structure:
//!
//! 1. **Runtime files** — copies the serde and/or bincode runtime `.ts`
//!    sources into the output directory, adapting file names and import paths
//!    for the active [`InstallTarget`] (Node: `index.ts` + extensionless
//!    imports; Deno: `mod.ts` + `.ts` extensions).
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
};

use serde_json::{Value, json};

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, Error, ExternalPackage, ExternalPackages, PackageLocation,
        SERDE_NAMESPACE, SourceInstaller, module,
        typescript::{InstallTarget, TypeScriptCodeGenerator},
    },
};

/// Installer for generated source files in TypeScript.
///
/// # Examples
///
/// ```rust
/// use facet_generate::generation::typescript::{self, InstallTarget};
///
/// let output_dir = std::path::PathBuf::from("output");
///
/// // For Deno (with .ts extensions)
/// let installer = typescript::Installer::new("my-package", &output_dir, InstallTarget::Deno);
///
/// // For React/Node.js (extensionless imports)
/// let installer = typescript::Installer::new("my-package", &output_dir, InstallTarget::Node);
/// ```
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    external_packages: ExternalPackages,
    target: InstallTarget,
    encoding: Encoding,
}

impl Installer {
    /// Create a new installer for the given package name, output directory, and
    /// target platform.
    ///
    /// Use the builder methods [`encoding`](Self::encoding) and
    /// [`external_packages`](Self::external_packages) to configure, then call
    /// [`generate`](Self::generate) to produce the output.
    #[must_use]
    pub fn new(package_name: &str, install_dir: impl AsRef<Path>, target: InstallTarget) -> Self {
        Installer {
            package_name: package_name.to_string(),
            install_dir: install_dir.as_ref().to_path_buf(),
            external_packages: ExternalPackages::new(),
            target,
            encoding: Encoding::default(),
        }
    }

    /// Set the encoding for serialization/deserialization.
    ///
    /// When set to anything other than [`Encoding::None`], the appropriate
    /// runtimes (serde + encoding-specific) are installed automatically by
    /// [`generate`](Self::generate).
    #[must_use]
    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
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
        // Install runtimes if an encoding is configured,
        // unless an external package provides them
        if !self.encoding.is_none() && !self.external_packages.contains_key(SERDE_NAMESPACE) {
            self.install_serde_runtime()?;
            if let Encoding::Bincode = self.encoding {
                self.install_bincode_runtime()?;
            }
        }

        // Split by namespace and install each module
        for (m, module_registry) in module::split(&self.package_name, registry) {
            let config = m.config().clone().with_encoding(self.encoding);
            self.install_module(&config, &module_registry)?;
        }

        // Write the package manifest
        let package_name = self.package_name.clone();
        self.install_manifest(&package_name)?;

        Ok(())
    }

    fn install_runtime(&self, source_dir: &include_dir::Dir, path: &str) -> Result<(), Error> {
        let dir_path = self.install_dir.join(path);
        create_dir_all(&dir_path)?;
        for entry in source_dir.files() {
            let original_name = entry.path().to_string_lossy();
            let file_name = self.target.transform_runtime_filename(&original_name);
            let content_str = std::str::from_utf8(entry.contents())?;
            let content = self.target.transform_import_path(content_str);
            let mut file = File::create(dir_path.join(file_name))?;
            file.write_all(content.as_bytes())?;
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
                            .as_ref()
                            .unwrap_or(&"*".to_string())
                            .clone(),
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
    /// For **Node** targets the file is written as `<namespace>.ts` directly
    /// in the install directory. For **Deno** targets it is written as
    /// `<namespace>/mod.ts`. Namespaces that correspond to external packages
    /// are skipped — their types are imported rather than generated.
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> Result<(), Error> {
        let skip_module = self.external_packages.contains_key(config.module_name());
        if skip_module {
            return Ok(());
        }
        let file_name = match self.target {
            InstallTarget::Node => {
                create_dir_all(&self.install_dir)?;

                let module_name = config.module_name();
                self.install_dir.join(format!("{module_name}.ts"))
            }
            InstallTarget::Deno => {
                let dir_path = self.install_dir.join(&config.module_name);
                create_dir_all(&dir_path)?;

                dir_path.join("mod.ts")
            }
        };
        let mut file = File::create(file_name)?;

        // Update config with external packages from installer
        let mut updated_config = config.clone();
        updated_config.external_packages = self.external_packages.clone();

        let generator = TypeScriptCodeGenerator::new(&updated_config, self.target);
        generator.output(&mut file, registry)?;

        Ok(())
    }

    fn install_serde_runtime(&mut self) -> Result<(), Error> {
        self.install_runtime(self.target.serde_runtime(), "serde")
    }

    fn install_bincode_runtime(&self) -> Result<(), Error> {
        self.install_runtime(self.target.bincode_runtime(), "bincode")
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
