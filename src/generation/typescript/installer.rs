use std::{
    collections::BTreeMap,
    fs::{File, create_dir_all},
    io::Write as _,
    path::{Path, PathBuf},
};

use include_dir::include_dir;
use serde_json::{Value, json};

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, ExternalPackage, ExternalPackages, PackageLocation, SourceInstaller,
        typescript::{CodeGenerator, InstallTarget},
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
/// let installer = typescript::Installer::new(&output_dir, &[], InstallTarget::Deno);
///
/// // For React/Node.js (extensionless imports)
/// let installer = typescript::Installer::new(&output_dir, &[], InstallTarget::Node);
/// ```
pub struct Installer {
    install_dir: PathBuf,
    external_packages: ExternalPackages,
    target: InstallTarget,
}

impl Installer {
    #[must_use]
    pub fn new(
        install_dir: impl AsRef<Path>,
        external_packages: &[ExternalPackage],
        target: InstallTarget,
    ) -> Self {
        let external_packages = external_packages
            .iter()
            .map(|d| (d.for_namespace.clone(), d.clone()))
            .collect();

        Installer {
            install_dir: install_dir.as_ref().to_path_buf(),
            external_packages,
            target,
        }
    }

    fn install_runtime(
        &self,
        source_dir: &include_dir::Dir,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir_path = self.install_dir.join(path);
        create_dir_all(&dir_path)?;
        for entry in source_dir.files() {
            let (file_name, content) = self.transform_runtime_file(entry)?;
            let mut file = File::create(dir_path.join(file_name))?;
            file.write_all(&content)?;
        }
        Ok(())
    }

    fn transform_imports(content: &str) -> String {
        // Transform imports and exports to remove .ts extensions
        content
            .lines()
            .map(|line| {
                let trimmed = line.trim_start();
                if (trimmed.starts_with("import") || trimmed.starts_with("export"))
                    && line.contains(".ts")
                {
                    // Remove .ts extensions from import and export statements
                    line.replace(".ts\"", "\"").replace(".ts'", "'")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn transform_runtime_file(
        &self,
        entry: &include_dir::File,
    ) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
        let file_name = match self.target {
            InstallTarget::Node => {
                if entry.path().file_name() == Some(std::ffi::OsStr::new("mod.ts")) {
                    "index.ts".to_string()
                } else {
                    entry.path().to_string_lossy().to_string()
                }
            }
            InstallTarget::Deno => entry.path().to_string_lossy().to_string(),
        };

        let content = match self.target {
            InstallTarget::Node => {
                // Strip .ts extensions from imports and exports
                let content_str = std::str::from_utf8(entry.contents())?;
                let transformed = Self::transform_imports(content_str);
                transformed.into_bytes()
            }
            InstallTarget::Deno => {
                // Keep original content with .ts extensions
                entry.contents().to_vec()
            }
        };

        Ok((file_name, content))
    }

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
                            // For npm packages, the URL might be like "https://registry.npmjs.org/package-name"
                            // or "https://registry.npmjs.org/@scope/package-name"
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
    type Error = Box<dyn std::error::Error>;

    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> Result<(), Self::Error> {
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

        let generator = CodeGenerator::new(&updated_config, self.target);
        generator.output(&mut file, registry)?;

        Ok(())
    }

    fn install_serde_runtime(&mut self) -> Result<(), Self::Error> {
        let dir = match self.target {
            InstallTarget::Node => include_dir!("runtime/typescript-node/serde"),
            InstallTarget::Deno => include_dir!("runtime/typescript-deno/serde"),
        };
        self.install_runtime(&dir, "serde")
    }

    fn install_bincode_runtime(&self) -> Result<(), Self::Error> {
        let dir = match self.target {
            InstallTarget::Node => include_dir!("runtime/typescript-node/bincode"),
            InstallTarget::Deno => include_dir!("runtime/typescript-deno/bincode"),
        };
        self.install_runtime(&dir, "bincode")
    }

    fn install_bcs_runtime(&self) -> Result<(), Self::Error> {
        let dir = match self.target {
            InstallTarget::Node => include_dir!("runtime/typescript-node/bcs"),
            InstallTarget::Deno => include_dir!("runtime/typescript-deno/bcs"),
        };
        self.install_runtime(&dir, "bcs")
    }

    fn install_manifest(&self, package_name: &str) -> std::result::Result<(), Self::Error> {
        let manifest = self.make_manifest(package_name);
        let manifest = serde_json::to_string_pretty(&manifest)?;

        let manifest_path = self.install_dir.join("package.json");
        let mut file = File::create(manifest_path)?;
        file.write_all(manifest.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "installer_tests.rs"]
mod installer_tests;
