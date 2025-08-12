use std::{env, fs, path::PathBuf};

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    generation::{
        ExternalPackage, PackageLocation, SourceInstaller as _, java,
        module::{self, Module},
        swift::Installer,
        tests::{check, read_files_and_create_expect_dirs},
        typescript::{self, InstallTarget},
    },
    reflect,
};

#[test]
#[allow(clippy::too_many_lines)]
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

    for target in ["java", "swift", "typescript"] {
        let tmp_dir = tempdir().unwrap();
        let tmp_path = tmp_dir.path();

        let snapshot_dir = this_dir.join(target);
        fs::create_dir_all(&snapshot_dir).unwrap();

        #[allow(clippy::match_same_arms)]
        match target {
            "java" => {
                let package_name = "com.example";
                let mut installer = java::Installer::new(
                    package_name,
                    tmp_path,
                    &[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("com.novi.serde".to_string()),
                        module_name: None,
                        version: None,
                    }],
                );
                installer.install_serde_runtime().unwrap();
                for (module, registry) in &module::split(package_name, &registry) {
                    let this_module = &module.config().module_name;
                    let is_root_package = package_name == this_module;
                    let module = if is_root_package {
                        module
                    } else {
                        &Module::new([package_name, this_module].join("."))
                    };
                    let config = module.config();
                    installer.install_module(config, registry).unwrap();
                }
            }
            "swift" => {
                let package_name = "Example";
                let mut installer = Installer::new(
                    package_name,
                    tmp_path.join(package_name),
                    &[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("../Serde".to_string()),
                        module_name: None,
                        version: None,
                    }],
                );
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config();
                    installer.install_module(config, registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();

                let package_name = "Serde";
                let mut installer = Installer::new(package_name, tmp_path.join(package_name), &[]);
                installer.install_serde_runtime().unwrap();
                installer.install_manifest(package_name).unwrap();
            }
            "typescript" => {
                let package_name = "example";
                let mut installer = typescript::Installer::new(
                    tmp_path.join(package_name),
                    &[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("../serde".to_string()),
                        module_name: None,
                        version: None,
                    }],
                    InstallTarget::Node,
                );

                for (module, registry) in &module::split(package_name, &registry) {
                    installer.install_module(module.config(), registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();

                let package_name = "serde";
                let mut installer = typescript::Installer::new(
                    tmp_path.join(package_name),
                    &[],
                    InstallTarget::Node,
                );
                installer.install_serde_runtime().unwrap();
                installer.install_manifest(package_name).unwrap();
            }
            _ => unreachable!(),
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}
