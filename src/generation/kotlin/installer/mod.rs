use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use heck::ToPascalCase;
use indoc::{formatdoc, indoc};

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, Error, ExternalPackage, ExternalPackages, PackageLocation,
        SourceInstaller, kotlin::CodeGenerator, module,
    },
};

/// Installer for generated source files in Kotlin.
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
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
            let config = m
                .config()
                .clone()
                .with_parent(&self.package_name)
                .with_encoding(self.encoding);
            self.install_module(&config, &module_registry)?;
        }

        // Write the package manifest
        let package_name = self.package_name.clone();
        self.install_manifest(&package_name)?;

        Ok(())
    }

    fn install_runtime(&self, source_dir: &str, path: &str) -> std::result::Result<(), Error> {
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
        // TODO: this should come from somewhere
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
    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Error> {
        // Extract the namespace from the module name to check if it's external
        let module_parts: Vec<&str> = config.module_name().split('.').collect();
        let namespace = module_parts.last().map_or("", |v| *v);
        let skip_module = self.external_packages.contains_key(namespace);

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

    fn install_serde_runtime(&mut self) -> std::result::Result<(), Error> {
        // Install the common serde runtime files needed for bincode
        let runtime_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/kotlin/com/novi/serde");

        if runtime_dir.exists() {
            self.install_runtime(runtime_dir.to_str().unwrap(), "com/novi/serde")?;
        }

        Ok(())
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Error> {
        // Install the bincode-specific runtime files
        let runtime_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("runtime/kotlin/com/novi/bincode");

        if runtime_dir.exists() {
            self.install_runtime(runtime_dir.to_str().unwrap(), "com/novi/bincode")?;
        }

        Ok(())
    }

    fn install_manifest(&self, package_name: &str) -> std::result::Result<(), Error> {
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
mod tests;
