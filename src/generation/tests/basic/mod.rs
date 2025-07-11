use std::{env, fs, path::PathBuf};

use expect_test::expect_file;
use facet::Facet;
use tempfile::TempDir;

use crate::{
    generation::{
        SourceInstaller as _, java,
        module::{self, Module},
        swift::Installer,
        tests::{check, find_files},
        typescript,
    },
    reflection::RegistryBuilder,
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

    let registry = RegistryBuilder::new().add_type::<Parent>().build();

    let this_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(file!())
        .parent()
        .unwrap()
        .join("snapshots");

    for target in ["java", "swift", "typescript"] {
        let tmp_dir = TempDir::new().unwrap();
        let tmp_path = tmp_dir.path();

        let snapshot_dir = this_dir.join(target);
        fs::create_dir_all(&snapshot_dir).unwrap();

        match target {
            "java" => {
                let package_name = "com.example";
                let mut installer = java::Installer::new(tmp_path.to_path_buf());
                for (module, registry) in &module::split(package_name, &registry) {
                    let this_module = &module.config().module_name;
                    let is_root_package = package_name == this_module;
                    let module = if is_root_package {
                        module
                    } else {
                        &Module::new([package_name, this_module].join("."))
                    };
                    let config = module.config().clone().with_serialization(false);
                    installer.install_module(&config, registry).unwrap();
                }
            }
            "swift" => {
                let package_name = "Example";
                let mut installer =
                    Installer::new(package_name.to_string(), tmp_path.to_path_buf(), vec![]);
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone().with_serialization(false);
                    installer.install_module(&config, registry).unwrap();
                    installer.install_manifest(package_name).unwrap();
                }
            }
            "typescript" => {
                let package_name = "example";
                let mut installer = typescript::Installer::new(tmp_path.to_path_buf());
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone().with_serialization(false);
                    installer.install_module(&config, registry).unwrap();
                }
            }
            _ => unreachable!(),
        }

        for (actual, expected) in find_files(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}
