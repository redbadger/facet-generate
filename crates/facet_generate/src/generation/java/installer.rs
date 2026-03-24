use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use include_dir::include_dir;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, Error, ExternalPackage, ExternalPackages, SERDE_NAMESPACE,
        SourceInstaller, java::JavaCodeGenerator, module,
    },
};

/// Installer for generated source files in Java.
#[deprecated(
    since = "0.16.0",
    note = "The Java generator is deprecated. Use the Kotlin generator instead."
)]
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
        Self {
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
    pub const fn encoding(mut self, encoding: Encoding) -> Self {
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
    ///
    /// # Errors
    ///
    /// Returns an error if any file operation or code generation step fails.
    pub fn generate(mut self, registry: &Registry) -> Result<(), Error> {
        // Install runtimes if an encoding is configured,
        // unless an external package provides them
        if !self.encoding.is_none() && !self.external_packages.contains_key(SERDE_NAMESPACE) {
            self.install_runtime(
                &include_dir!("$CARGO_MANIFEST_DIR/runtime/java/com/novi/serde"),
                "com/novi/serde",
            )?;
            if self.encoding == Encoding::Bincode {
                self.install_runtime(
                    &include_dir!("$CARGO_MANIFEST_DIR/runtime/java/com/novi/bincode"),
                    "com/novi/bincode",
                )?;
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

        Ok(())
    }

    /// Installs the serde Java runtime sources into the output directory.
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O fails.
    pub fn install_serde_runtime(&mut self) -> std::result::Result<(), Error> {
        self.install_runtime(
            &include_dir!("$CARGO_MANIFEST_DIR/runtime/java/com/novi/serde"),
            "com/novi/serde",
        )
    }

    /// Installs the bincode Java runtime sources into the output directory.
    ///
    /// # Errors
    ///
    /// Returns an error if any file I/O fails.
    pub fn install_bincode_runtime(&self) -> std::result::Result<(), Error> {
        self.install_runtime(
            &include_dir!("$CARGO_MANIFEST_DIR/runtime/java/com/novi/bincode"),
            "com/novi/bincode",
        )
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

        // Update config with external packages from installer
        let mut updated_config = config.clone();
        updated_config.external_packages = self.external_packages.clone();

        let generator = JavaCodeGenerator::new(&updated_config);
        generator.write_source_files(self.install_dir.clone(), registry)?;
        Ok(())
    }
}
