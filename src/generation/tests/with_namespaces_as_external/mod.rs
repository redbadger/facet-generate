use std::{env, fs, path::PathBuf};

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    generation::{
        ExternalPackage, PackageLocation, SourceInstaller as _, java, module, swift,
        tests::{check, read_files_and_create_expect_dirs},
        typescript::{self, InstallTarget},
    },
    reflection::RegistryBuilder,
};

#[test]
#[allow(clippy::too_many_lines)]
fn test() {
    #[derive(Facet)]
    #[facet(namespace = "other")]
    pub struct OtherChild {
        name: String,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[facet(namespace = "other")]
    #[allow(unused)]
    pub enum OtherParent {
        Child(OtherChild),
    }

    #[derive(Facet)]
    struct Child {
        external: OtherParent,
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
        let tmp_dir = tempdir().unwrap();
        let tmp_path = tmp_dir.path();

        let snapshot_dir = this_dir.join(target);
        fs::create_dir_all(&snapshot_dir).unwrap();

        match target {
            "java" => {
                let package_name = "com.example";
                let mut installer = java::Installer::new(
                    package_name,
                    tmp_path,
                    &[ExternalPackage {
                        for_namespace: "other".to_string(),
                        location: PackageLocation::Path("com.example2.other".to_string()),
                        module_name: None,
                        version: None,
                    }],
                );
                for (module, registry) in &module::split(package_name, &registry) {
                    let this_module = &module.config().module_name;
                    let is_root_package = package_name == this_module;
                    let module = if is_root_package {
                        module
                    } else {
                        &module::Module::new([package_name, this_module].join("."))
                    };
                    let config = module.config().clone().without_serialization();
                    installer.install_module(&config, registry).unwrap();
                }
            }
            "swift" => {
                let package_name = "Example";
                let mut installer = swift::Installer::new(
                    package_name,
                    tmp_path,
                    &[ExternalPackage {
                        for_namespace: "other".to_string(),
                        location: PackageLocation::Url(
                            "https://github.com/example/other".to_string(),
                        ),
                        module_name: None,
                        version: Some("1.0.0".to_string()),
                    }],
                );
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone().without_serialization();
                    installer.install_module(&config, registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();
            }
            "typescript" => {
                let package_name = "example";
                let mut installer = typescript::Installer::new(
                    tmp_path,
                    &[ExternalPackage {
                        for_namespace: "other".to_string(),
                        location: PackageLocation::Url(
                            "https://registry.npmjs.org/other".to_string(),
                        ),
                        module_name: None,
                        version: Some("^1.0.0".to_string()),
                    }],
                    InstallTarget::Node,
                );

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
