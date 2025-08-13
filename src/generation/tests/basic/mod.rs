use std::{env, fs, path::PathBuf};

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    generation::{
        SourceInstaller as _, java, kotlin, module, swift,
        tests::{check, read_files_and_create_expect_dirs},
        typescript::{self, InstallTarget},
    },
    reflect,
};

#[test]
fn test() {
    #[derive(Facet)]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Parent {
        Child(Child),
    }

    let registry = reflect!(Parent);

    let this_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(file!())
        .parent()
        .unwrap()
        .join("snapshots");

    for target in ["java", "kotlin", "swift", "typescript"] {
        let tmp_dir = tempdir().unwrap();
        let tmp_path = tmp_dir.path();

        let snapshot_dir = this_dir.join(target);
        fs::create_dir_all(&snapshot_dir).unwrap();

        match target {
            "java" => {
                let package_name = "com.example";
                let mut installer = java::Installer::new(package_name, tmp_path, &[]);
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module
                        .with_parent(package_name)
                        .config()
                        .clone()
                        .without_serialization();
                    installer.install_module(&config, registry).unwrap();
                }
            }
            "kotlin" => {
                let package_name = "com.example";
                let mut installer = kotlin::Installer::new(package_name, tmp_path, &[]);
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone().without_serialization();
                    installer.install_module(&config, registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();
            }
            "swift" => {
                let package_name = "Example";
                let mut installer = swift::Installer::new(package_name, tmp_path, &[]);
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone().without_serialization();
                    installer.install_module(&config, registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();
            }
            "typescript" => {
                let package_name = "example";
                let mut installer = typescript::Installer::new(tmp_path, &[], InstallTarget::Node);
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone().without_serialization();
                    installer.install_module(&config, registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();
            }
            _ => unreachable!(),
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}
