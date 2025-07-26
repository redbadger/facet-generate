use std::{
    collections::BTreeMap,
    io::Write as _,
    path::{Path, PathBuf},
};

use include_dir::include_dir;
use serde_json::{Value, json};

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, ExternalPackage, ExternalPackages, PackageLocation, SourceInstaller,
        typescript::CodeGenerator,
    },
};

/// Installer for generated source files in TypeScript.
pub struct Installer {
    install_dir: PathBuf,
    external_packages: ExternalPackages,
}

impl Installer {
    #[must_use]
    pub fn new(install_dir: impl AsRef<Path>, external_packages: &[ExternalPackage]) -> Self {
        let external_packages = external_packages
            .iter()
            .map(|d| (d.for_namespace.clone(), d.clone()))
            .collect();

        Installer {
            install_dir: install_dir.as_ref().to_path_buf(),
            external_packages,
        }
    }

    fn install_runtime(
        &self,
        source_dir: &include_dir::Dir,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir_path = self.install_dir.join(path);
        std::fs::create_dir_all(&dir_path)?;
        for entry in source_dir.files() {
            let mut file = std::fs::File::create(dir_path.join(entry.path()))?;
            file.write_all(entry.contents())?;
        }
        Ok(())
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
                let package_name = Self::package_name_from_location(&external_package.location);
                let version = match &external_package.location {
                    PackageLocation::Path(path) => format!("file:{path}"),
                    PackageLocation::Url(_) => external_package
                        .version
                        .as_ref()
                        .unwrap_or(&"*".to_string())
                        .clone(),
                };
                dependencies.insert(package_name, version);
            }

            manifest["dependencies"] = json!(dependencies);
        }

        // Always add devDependencies
        manifest["devDependencies"] = json!({
            "typescript": "^5.8.3"
        });

        manifest
    }

    fn package_name_from_location(location: &PackageLocation) -> String {
        match location {
            PackageLocation::Url(url) => {
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
            }
            PackageLocation::Path(path) => {
                // For local packages, use the directory name
                if let Some(last_segment) = path.split('/').next_back() {
                    last_segment.to_string()
                } else {
                    path.clone()
                }
            }
        }
    }
}

impl SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> Result<(), Self::Error> {
        let should_install_module = !self.external_packages.contains_key(config.module_name());

        if should_install_module {
            let dir_path = self.install_dir.join(&config.module_name);
            std::fs::create_dir_all(&dir_path)?;
            let source_path = dir_path.join("mod.ts");
            let mut file = std::fs::File::create(source_path)?;

            let generator = CodeGenerator::new(config);
            generator.output(&mut file, registry)?;
        }

        Ok(())
    }

    fn install_serde_runtime(&mut self) -> Result<(), Self::Error> {
        self.install_runtime(&include_dir!("runtime/typescript/serde"), "serde")
    }

    fn install_bincode_runtime(&self) -> Result<(), Self::Error> {
        self.install_runtime(&include_dir!("runtime/typescript/bincode"), "bincode")
    }

    fn install_bcs_runtime(&self) -> Result<(), Self::Error> {
        self.install_runtime(&include_dir!("runtime/typescript/bcs"), "bcs")
    }

    fn install_manifest(&self, package_name: &str) -> std::result::Result<(), Self::Error> {
        let manifest = self.make_manifest(package_name);
        let manifest = serde_json::to_string_pretty(&manifest)?;

        let manifest_path = self.install_dir.join("package.json");
        let mut file = std::fs::File::create(manifest_path)?;
        file.write_all(manifest.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "installer_tests.rs"]
mod installer_tests;
