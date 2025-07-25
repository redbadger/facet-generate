use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use include_dir::include_dir;

use crate::{
    Registry,
    generation::{CodeGeneratorConfig, SourceInstaller, java::CodeGenerator},
};

/// Installer for generated source files in Java.
pub struct Installer {
    install_dir: PathBuf,
}

impl Installer {
    #[must_use]
    pub fn new(install_dir: impl AsRef<Path>) -> Self {
        Installer {
            install_dir: install_dir.as_ref().to_path_buf(),
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
        let generator = CodeGenerator::new(config);
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
