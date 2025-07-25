use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use include_dir::include_dir;

use crate::{
    Registry,
    generation::{CodeGeneratorConfig, SourceInstaller, typescript::CodeGenerator},
};

/// Installer for generated source files in TypeScript.
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
    ) -> Result<(), Box<dyn std::error::Error>> {
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
    ) -> Result<(), Self::Error> {
        let dir_path = self.install_dir.join(&config.module_name);
        std::fs::create_dir_all(&dir_path)?;
        let source_path = dir_path.join("mod.ts");
        let mut file = std::fs::File::create(source_path)?;

        let generator = CodeGenerator::new(config);
        generator.output(&mut file, registry)?;
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
}
