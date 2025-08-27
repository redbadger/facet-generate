use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use heck::ToPascalCase;
use indoc::{formatdoc, indoc};

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, ExternalPackage, ExternalPackages, PackageLocation,
        SourceInstaller, kotlin::CodeGenerator,
    },
};

/// Installer for generated source files in Kotlin.
pub struct Installer {
    #[allow(dead_code)]
    package_name: String,
    install_dir: PathBuf,
    external_packages: ExternalPackages,
    encoding: Encoding,
}

impl Installer {
    #[must_use]
    pub fn new(
        package_name: &str,
        install_dir: impl AsRef<Path>,
        external_packages: &[ExternalPackage],
    ) -> Self {
        let external_packages = external_packages
            .iter()
            .map(|d| (d.for_namespace.clone(), d.clone()))
            .collect();

        Installer {
            package_name: package_name.to_string(),
            install_dir: install_dir.as_ref().to_path_buf(),
            external_packages,
            encoding: Encoding::default(),
        }
    }

    fn install_runtime(
        &self,
        source_dir: &str,
        path: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let target_dir = self.install_dir.join(path);
        std::fs::create_dir_all(&target_dir)?;

        let source_path = std::path::Path::new(source_dir);
        if source_path.exists() {
            copy_dir_contents(source_path, &target_dir)?;
        }

        Ok(())
    }

    #[must_use]
    pub fn make_manifest(&self, package_name: &str) -> String {
        // TODO: this should come from somehwere
        const VERSION: &str = "1.0.0";

        let mut dependencies = Vec::new();

        // Add kotlinx.serialization only if not using bincode
        let uses_bincode = self.encoding == Encoding::Bincode;
        if !uses_bincode {
            dependencies.push(
                r#"    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")"#
                    .to_string(),
            );
        }

        // Add external package dependencies
        for external_package in self.external_packages.values() {
            match &external_package.location {
                PackageLocation::Path(path) => {
                    dependencies.push(format!(r#"    implementation(files("{path}"))"#));
                }
                PackageLocation::Url(url) => {
                    let default_version = "1.0.0".to_string();
                    let version = external_package
                        .version
                        .as_ref()
                        .unwrap_or(&default_version);

                    // Extract artifact name from URL or use namespace
                    let artifact_name = url.split('/').next_back().map_or_else(
                        || external_package.for_namespace.clone(),
                        ToString::to_string,
                    );

                    dependencies.push(format!(
                        r#"    implementation("{artifact_name}:{version}")"#
                    ));
                }
            }
        }

        let dependencies_str = dependencies.join("\n");

        let plugins = indoc!(
            r#"
            plugins {
                kotlin("jvm") version "2.2.0"
                kotlin("plugin.serialization") version "2.2.0"
                `java-library`
            }"#
        );

        formatdoc!(
            r#"
                {plugins}

                group = "{package_name}"
                version = "{VERSION}"

                repositories {{
                    mavenCentral()
                }}

                dependencies {{
                {dependencies_str}
                }}

                tasks.withType<Jar> {{
                    manifest {{
                        attributes["Implementation-Title"] = "{package_name}"
                        attributes["Implementation-Version"] = "{VERSION}"
                    }}
                }}
            "#
        )
    }
}

impl SourceInstaller for Installer {
    type Error = Box<dyn std::error::Error>;

    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
        let skip_module = self.external_packages.contains_key(config.module_name());

        if skip_module {
            return Ok(());
        }

        // Track encodings used in this module
        self.encoding = config.encoding;

        // Convert module name to package path (e.g., "com.example.types" -> "com/example/types")
        let package_path = config.module_name().replace('.', "/");
        let module_dir = self.install_dir.join(&package_path);
        std::fs::create_dir_all(&module_dir)?;

        // All types in the module go into a single file
        // Use the last part of the module name as the file name
        let file_name = config
            .module_name()
            .split('.')
            .next_back()
            .unwrap_or(config.module_name())
            .to_pascal_case();

        let source_path = module_dir.join(format!("{file_name}.kt"));
        let mut file = std::fs::File::create(source_path)?;

        // Update config with external packages from installer
        let mut updated_config = config.clone();
        updated_config.external_packages = self.external_packages.clone();

        let generator = CodeGenerator::new(&updated_config);
        generator.output(&mut file, registry)?;

        Ok(())
    }

    fn install_serde_runtime(&mut self) -> std::result::Result<(), Self::Error> {
        // Install the common serde runtime files needed for bincode
        let runtime_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/java/com/novi/serde");

        if runtime_dir.exists() {
            self.install_runtime(runtime_dir.to_str().unwrap(), "com/novi/serde")?;
        }

        Ok(())
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        // Install the bincode-specific runtime files
        let runtime_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/java/com/novi/bincode");

        if runtime_dir.exists() {
            self.install_runtime(runtime_dir.to_str().unwrap(), "com/novi/bincode")?;
        }

        Ok(())
    }

    fn install_bcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        // BCS runtime would be installed similarly to bincode if implemented
        // For now, BCS is not supported in Kotlin
        Ok(())
    }

    fn install_manifest(&self, package_name: &str) -> std::result::Result<(), Self::Error> {
        let manifest = self.make_manifest(package_name);

        let manifest_path = self.install_dir.join("build.gradle.kts");
        let mut file = std::fs::File::create(manifest_path)?;
        file.write_all(manifest.as_bytes())?;

        Ok(())
    }
}

fn copy_dir_contents(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            let dst_dir = dst.join(entry.file_name());
            std::fs::create_dir_all(&dst_dir)?;
            copy_dir_contents(&entry.path(), &dst_dir)?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "installer_tests.rs"]
mod installer_tests;
