use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write as _,
    path::{Path, PathBuf},
};

use heck::ToUpperCamelCase as _;
use include_dir::include_dir;
use indent::indent_all_with;
use indoc::formatdoc;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, Error, ExternalPackage, ExternalPackages, SourceInstaller,
        module, swift::generator::CodeGenerator,
    },
};

/// Installer for generated source files in Swift.
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    targets: BTreeMap<String, BTreeSet<String>>,
    external_packages: ExternalPackages,
    encoding: Encoding,
}

impl Installer {
    /// Create a new installer for the given package name and output directory.
    ///
    /// Use the builder methods [`encoding`](Self::encoding) and
    /// [`external_packages`](Self::external_packages) to configure, then call
    /// [`generate`](Self::generate) to produce the output.
    #[must_use]
    pub fn new(package_name: &str, install_dir: impl AsRef<Path>) -> Self {
        Installer {
            package_name: package_name.to_string(),
            install_dir: install_dir.as_ref().to_path_buf(),
            targets: BTreeMap::new(),
            external_packages: ExternalPackages::new(),
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
        // Install runtimes if an encoding is configured
        if !self.encoding.is_none() {
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

    fn install_runtime(
        &self,
        source_dir: &include_dir::Dir,
        path: &str,
    ) -> std::result::Result<(), Error> {
        let dir_path = self.install_dir.join(path);
        std::fs::create_dir_all(&dir_path)?;

        for entry in source_dir.files() {
            let mut file = std::fs::File::create(dir_path.join(entry.path()))?;
            file.write_all(entry.contents())?;
        }

        Ok(())
    }

    #[must_use]
    pub fn make_manifest(&self, package_name: &str) -> String {
        let mut all_targets = self.targets.clone();

        let mut package_targets = BTreeSet::new();

        for targets in all_targets.values() {
            for target in targets {
                package_targets.insert(target.to_upper_camel_case());
            }
        }
        all_targets.insert(package_name.to_upper_camel_case(), package_targets);

        // Get names of external dependencies to exclude from target creation
        let external_package_names: BTreeSet<String> = self
            .external_packages
            .values()
            .map(|d| d.for_namespace.to_upper_camel_case())
            .collect();

        // Find all dependencies referenced by any target
        let mut all_dependencies = BTreeSet::new();
        for dependencies in all_targets.values() {
            for dep in dependencies {
                all_dependencies.insert(dep.clone());
            }
        }

        // Determine which targets are top-level (not dependencies of other targets)
        let top_level_targets: Vec<String> = all_targets
            .keys()
            .filter(|name| {
                !external_package_names.contains(*name) && !all_dependencies.contains(*name)
            })
            .cloned()
            .collect();

        // If no top-level targets found (all are dependencies), include the main package
        let library_targets = if top_level_targets.is_empty() {
            vec![package_name.to_string()]
        } else {
            top_level_targets
        };

        let targets: Vec<String> = all_targets
            .iter()
            .filter(|(name, _)| !external_package_names.contains(*name))
            .map(|(name, dependencies)| {
                let dependencies = dependencies
                    .iter()
                    .map(|dep| format!(r#""{dep}""#))
                    .collect::<Vec<String>>()
                    .join(", ");

                let base_target = formatdoc! {r#"
                    .target(
                        name: "{name}",
                        dependencies: [{dependencies}]
                    ),"#};

                indent_all_with("        ", &base_target)
            })
            .collect();

        let library_targets_str = library_targets
            .iter()
            .map(|t| format!(r#""{t}""#))
            .collect::<Vec<_>>()
            .join(", ");

        if self.external_packages.is_empty() {
            formatdoc! {r#"
                // swift-tools-version: 5.8
                import PackageDescription

                let package = Package(
                    name: "{package}",
                    products: [
                        .library(
                            name: "{package}",
                            targets: [{library_targets}]
                        )
                    ],
                    targets: [{targets}]
                )
                "#,
                package = self.package_name,
                library_targets = library_targets_str,
                targets = format!("\n{}\n    ", targets.join("\n"))
            }
        } else {
            let external_packages = self
                .external_packages
                .values()
                .cloned()
                .map(|d| ExternalPackage::to_swift(d, 2))
                .collect::<Vec<_>>()
                .join(",\n");

            let dependencies_section = format!("\n{external_packages}\n    ");
            formatdoc! {r#"
                // swift-tools-version: 5.8
                import PackageDescription

                let package = Package(
                    name: "{package}",
                    products: [
                        .library(
                            name: "{package}",
                            targets: [{library_targets}]
                        )
                    ],
                    dependencies: [{dependencies}],
                    targets: [{targets}]
                )
                "#,
                package = self.package_name,
                library_targets = library_targets_str,
                dependencies = dependencies_section,
                targets = format!("\n{}\n    ", targets.join("\n"))
            }
        }
    }
}

impl SourceInstaller for Installer {
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Error> {
        let skip_module = self.external_packages.contains_key(config.module_name());

        if skip_module {
            return Ok(());
        }

        let module_name = config.module_name().to_upper_camel_case();

        let targets = self.targets.entry(module_name.clone()).or_default();
        for target in config.external_definitions.keys() {
            targets.insert(target.to_upper_camel_case());
        }

        if config.has_encoding() {
            targets.insert("Serde".to_string());
        }

        let dir_path = self.install_dir.join("Sources").join(&module_name);
        std::fs::create_dir_all(&dir_path)?;
        let source_path = dir_path.join(format!("{module_name}.swift"));

        let mut file = std::fs::File::create(source_path)?;

        // Update config with external packages from installer
        let mut updated_config = config.clone();
        updated_config.external_packages = self.external_packages.clone();

        let generator = CodeGenerator::new(&updated_config);
        generator.output(&mut file, registry)?;

        Ok(())
    }

    fn install_serde_runtime(&mut self) -> std::result::Result<(), Error> {
        self.install_runtime(
            &include_dir!("$CARGO_MANIFEST_DIR/runtime/swift/Sources/Serde"),
            "Sources/Serde",
        )?;

        // Register Serde as a local target
        self.targets.insert("Serde".to_string(), BTreeSet::new());

        Ok(())
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Error> {
        // Ignored. Currently always installed with Serde.
        Ok(())
    }

    fn install_manifest(&self, package_name: &str) -> std::result::Result<(), Error> {
        let manifest = self.make_manifest(package_name);

        let manifest_path = self.install_dir.join("Package.swift");
        let mut file = std::fs::File::create(manifest_path)?;
        file.write_all(manifest.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
