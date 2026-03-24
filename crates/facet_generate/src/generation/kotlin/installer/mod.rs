//! Project scaffolding — writes a ready-to-build Kotlin project to disk.
//!
//! The [`Installer`] is the final stage of the Kotlin generation pipeline.
//! While [`KotlinCodeGenerator`] produces the *contents* of a single source file,
//! the installer is responsible for the surrounding project structure:
//!
//! 1. **Runtime files** — copies the serde and/or bincode runtime `.kt`
//!    sources from `runtime/kotlin/` into the output directory. These provide
//!    the `Serializer`, `Deserializer`, `BincodeSerializer`, etc. that the
//!    generated `serialize`/`deserialize` methods call into.
//!
//! 2. **Per-namespace source files** — splits the registry by namespace
//!    (via [`module::split`]) and calls [`KotlinCodeGenerator`] once per namespace,
//!    writing each to its own `.kt` file under the appropriate package
//!    directory (e.g. `com/example/types/Types.kt`).
//!
//! 3. **`build.gradle.kts`** — generates a Gradle build script with the
//!    correct plugins, dependencies (`kotlinx-serialization-json` for JSON
//!    encoding, or external package references), and JAR manifest metadata.

use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
};

use heck::ToPascalCase;
use indoc::{formatdoc, indoc};

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, Error, ExternalPackage, ExternalPackages, PackageLocation,
        SERDE_NAMESPACE, SourceInstaller,
        kotlin::{Kotlin, KotlinCodeGenerator},
        module,
        plugin::EmitterPlugin,
    },
};

/// Writes a complete Kotlin project (source files, runtime, build script)
/// into an output directory.
///
/// Configured via builder methods, then consumed by [`generate`](Self::generate).
pub struct Installer {
    package_name: String,
    install_dir: PathBuf,
    external_packages: ExternalPackages,
    encoding: Encoding,
    plugins: Vec<Arc<dyn EmitterPlugin<Kotlin>>>,
}

impl Installer {
    /// Create a new installer for the given package name and output directory.
    ///
    /// Use the builder methods [`encoding`](Self::encoding),
    /// [`plugin`](Self::plugin), and
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
    ///
    /// Note: if plugins have been added explicitly via [`plugin`](Self::plugin),
    /// those take priority and this setting is ignored.
    #[must_use]
    pub const fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Add a plugin to be used during code generation.
    ///
    /// When plugins are added explicitly, they take priority over the
    /// [`encoding`](Self::encoding) setting. Multiple plugins can be added
    /// and they are invoked in the order they were registered.
    #[must_use]
    pub fn plugin(mut self, plugin: impl Into<Arc<dyn EmitterPlugin<Kotlin>>>) -> Self {
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
                Encoding::Bincode => {
                    vec![Arc::new(crate::generation::bincode::BincodePlugin)
                        as Arc<dyn EmitterPlugin<Kotlin>>]
                }
                Encoding::Json => {
                    vec![Arc::new(crate::generation::json::JsonPlugin)
                        as Arc<dyn EmitterPlugin<Kotlin>>]
                }
                Encoding::None => vec![],
            };
        }

        let lang = {
            let mut base = Kotlin::new(&config, registry);
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
            let config = m.config().clone().with_parent(&self.package_name);
            self.install_module(&config, &module_registry)?;
        }

        // Write the package manifest
        let package_name = self.package_name.clone();
        self.install_manifest(&package_name)?;

        Ok(())
    }

    /// Installs the serde Kotlin runtime sources into the output directory.
    ///
    /// Delegates to `JsonPlugin::runtime_files` which embeds the serde
    /// sources via `include_dir!`.  This method is provided for callers that
    /// need fine-grained control; most callers should prefer [`generate`](Self::generate).
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O fails.
    pub fn install_serde_runtime(&mut self) -> Result<(), Error> {
        let config = CodeGeneratorConfig::new(String::new());
        let lang = Kotlin::new(&config, &BTreeMap::default())
            .with_plugin(Arc::new(crate::generation::json::JsonPlugin));
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

    /// Installs the bincode Kotlin runtime sources into the output directory.
    ///
    /// Delegates to `BincodePlugin::runtime_files`, writing only the
    /// `com/novi/bincode/` files.  Most callers should prefer
    /// [`generate`](Self::generate).
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O fails.
    pub fn install_bincode_runtime(&self) -> Result<(), Error> {
        let config = CodeGeneratorConfig::new(String::new());
        let lang = Kotlin::new(&config, &BTreeMap::default())
            .with_plugin(Arc::new(crate::generation::bincode::BincodePlugin));
        for plugin in lang.plugins() {
            for file in plugin
                .runtime_files()
                .into_iter()
                .filter(|f| f.relative_path.starts_with("com/novi/bincode/"))
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

    /// Produces the contents of a `build.gradle.kts` file.
    ///
    /// Includes `kotlinx-serialization-json` when not using bincode, and adds
    /// `implementation(files(…))` or `implementation("artifact:version")` for
    /// each configured external package.
    #[must_use]
    pub fn make_manifest(&self, package_name: &str) -> String {
        // TODO: this should come from somewhere
        const VERSION: &str = "1.0.0";

        // Collect manifest dependencies from active plugins. For example,
        // JsonPlugin contributes the kotlinx-serialization-json dependency.
        let plugin_config = CodeGeneratorConfig::new(package_name.to_string());
        let resolved = if self.plugins.is_empty() {
            match self.encoding {
                Encoding::Bincode => {
                    vec![Arc::new(crate::generation::bincode::BincodePlugin)
                        as Arc<dyn EmitterPlugin<Kotlin>>]
                }
                Encoding::Json => {
                    vec![Arc::new(crate::generation::json::JsonPlugin)
                        as Arc<dyn EmitterPlugin<Kotlin>>]
                }
                Encoding::None => vec![],
            }
        } else {
            self.plugins.clone()
        };
        let lang = {
            let mut base = Kotlin::new(&plugin_config, &BTreeMap::default());
            for p in &resolved {
                base = base.with_plugin(p.clone());
            }
            base
        };
        let mut dependencies: Vec<String> = lang
            .plugins()
            .iter()
            .flat_map(|p| p.manifest_dependencies())
            .collect();

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

        let dependencies_str = if dependencies.is_empty() {
            String::new()
        } else {
            format!("\n{}\n", dependencies.join("\n"))
        };

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

                dependencies {{{dependencies_str}}}

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
    /// Generates a single `.kt` source file for one namespace module.
    ///
    /// Skips modules whose namespace matches an external package (those types
    /// are provided by the external dependency, not generated). For the rest,
    /// converts the dot-separated module name to a directory path, creates it,
    /// and writes the output of [`KotlinCodeGenerator::output`] into a `PascalCased`
    /// `.kt` file (e.g. module `com.example.types` → `com/example/types/Types.kt`).
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
            .unwrap_or_else(|| config.module_name())
            .to_pascal_case();

        let source_path = module_dir.join(format!("{file_name}.kt"));
        let mut file = std::fs::File::create(source_path)?;

        // Update config with external packages from installer
        let mut updated_config = config.clone();
        updated_config.external_packages = self.external_packages.clone();

        let generator = KotlinCodeGenerator::new(&updated_config)
            .with_encoding(self.encoding)
            .with_plugins(self.plugins.clone());
        generator.output(&mut file, registry)?;

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

#[cfg(test)]
mod tests;
