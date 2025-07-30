use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use include_dir::include_dir;

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, ExternalPackage, ExternalPackages, SourceInstaller,
        java::CodeGenerator,
    },
};

/// Installer for generated source files in Java.
pub struct Installer {
    install_dir: PathBuf,
    external_packages: ExternalPackages,
}

impl Installer {
    #[must_use]
    pub fn new(
        _package_name: &str,
        install_dir: impl AsRef<Path>,
        external_packages: &[ExternalPackage],
    ) -> Self {
        let external_packages = external_packages
            .iter()
            .map(|dependency| (dependency.for_namespace.clone(), dependency.clone()))
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
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
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
    type Error = Box<dyn std::error::Error>;

    fn install_module(
        &mut self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::result::Result<(), Self::Error> {
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

        let generator = CodeGenerator::new(&updated_config);
        generator.write_source_files(self.install_dir.clone(), registry)?;
        Ok(())
    }

    fn install_serde_runtime(&mut self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            &include_dir!("runtime/java/com/novi/serde"),
            "com/novi/serde",
        )
    }

    fn install_bincode_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(
            &include_dir!("runtime/java/com/novi/bincode"),
            "com/novi/bincode",
        )
    }

    fn install_bcs_runtime(&self) -> std::result::Result<(), Self::Error> {
        self.install_runtime(&include_dir!("runtime/java/com/novi/bcs"), "com/novi/bcs")
    }
}
