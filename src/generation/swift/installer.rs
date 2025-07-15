use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write as _,
    path::PathBuf,
};

use heck::ToUpperCamelCase as _;
use include_dir::include_dir;
use indent::indent_all_with;
use indoc::formatdoc;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, ExternalPackage, ExternalPackages, SourceInstaller,
        swift::generator::CodeGenerator,
    },
};

/// Installer for generated source files in Swift.
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    targets: BTreeMap<String, BTreeSet<String>>,
    external_packages: ExternalPackages,
}

impl Installer {
    #[must_use]
    pub fn new(
        package_name: String,
        install_dir: PathBuf,
        external_packages: Vec<ExternalPackage>,
    ) -> Self {
        let targets = BTreeMap::new();

        let external_packages = external_packages
            .into_iter()
            .map(|d| {
                (
                    d.for_namespace.clone(),
                    ExternalPackage {
                        for_namespace: d.for_namespace,
                        location: d.location,
                        version: d.version,
                    },
                )
            })
            .collect();

        Installer {
            package_name,
            install_dir,
            targets,
            external_packages,
        }
    }

    fn install_runtime(
        &self,
        source_dir: &include_dir::Dir,
        path: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
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

        if self.external_packages.is_empty() {
            formatdoc! {r#"
                // swift-tools-version: 5.8
                import PackageDescription

                let package = Package(
                    name: "{package}",
                    products: [
                        .library(
                            name: "{package}",
                            targets: ["{package}"]
                        )
                    ],
                    targets: [{targets}]
                )
                "#,
                package = self.package_name,
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
                            targets: ["{package}"]
                        )
                    ],
                    dependencies: [{dependencies}],
                    targets: [{targets}]
                )
                "#,
                package = self.package_name,
                dependencies = dependencies_section,
                targets = format!("\n{}\n    ", targets.join("\n"))
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
    ) -> std::result::Result<(), Self::Error> {
        let should_install_module = !self.external_packages.contains_key(config.module_name());

        if should_install_module {
            let module_name = config.module_name().to_upper_camel_case();

            let targets = self.targets.entry(module_name.clone()).or_default();
            for target in config.external_definitions.keys() {
                targets.insert(target.clone());
            }

            if config.serialization {
                targets.insert("Serde".to_string());
            }

            let dir_path = self.install_dir.join("Sources").join(&module_name);
            std::fs::create_dir_all(&dir_path)?;
            let source_path = dir_path.join(format!("{module_name}.swift"));

            let mut file = std::fs::File::create(source_path)?;
            let generator = CodeGenerator::new(config);
            generator.output(&mut file, registry)?;
        }

        Ok(())
    }

    fn install_serde_runtime(&mut self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            &include_dir!("runtime/swift/Sources/Serde"),
            "Sources/Serde",
        )?;

        // Register Serde as a local target
        self.targets.insert("Serde".to_string(), BTreeSet::new());

        Ok(())
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        // Ignored. Currently always installed with Serde.
        Ok(())
    }

    fn install_bcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        // Ignored. Currently always installed with Serde.
        Ok(())
    }

    fn install_manifest(
        &self,
        package_name: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let manifest = self.make_manifest(package_name);

        let manifest_path = self.install_dir.join("Package.swift");
        let mut file = std::fs::File::create(manifest_path)?;
        file.write_all(manifest.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
#[path = "installer_tests.rs"]
mod installer_tests;
