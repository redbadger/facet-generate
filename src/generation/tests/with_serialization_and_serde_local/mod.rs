use std::{env, fs, path::PathBuf};

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    generation::{
        Encoding, ExternalPackage, Language, PackageLocation, SourceInstaller as _, java, module,
        swift::Installer,
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

    let registry = reflect!(Parent).unwrap();

    let this_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(file!())
        .parent()
        .unwrap()
        .join("snapshots");

    for target in [
        Language::Java,
        // Language::Kotlin,
        Language::Swift,
        Language::TypeScript,
    ] {
        let tmp_dir = tempdir().unwrap();
        let tmp_path = tmp_dir.path();

        let snapshot_dir = this_dir.join(target.to_string().to_lowercase());
        fs::create_dir_all(&snapshot_dir).unwrap();

        match target {
            Language::Java => {
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
                    let config = module
                        .config()
                        .clone()
                        .with_parent(package_name)
                        .with_encoding(Encoding::Bincode);
                    installer.install_module(&config, registry).unwrap();
                }
            }
            Language::Kotlin => {}
            Language::Swift => {
                let package_name = "Example";
                let mut installer = Installer::new(
                    package_name,
                    tmp_path,
                    &[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("../Serde".to_string()),
                        module_name: None,
                        version: None,
                    }],
                );
                installer.install_serde_runtime().unwrap();

                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone().with_encoding(Encoding::Bincode);
                    installer.install_module(&config, registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();
            }
            Language::TypeScript => {
                let package_name = "example";
                let mut installer = typescript::Installer::new(
                    tmp_path,
                    &[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("./serde".to_string()),
                        module_name: None,
                        version: None,
                    }],
                    InstallTarget::Node,
                );
                installer.install_serde_runtime().unwrap();

                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone().with_encoding(Encoding::Bincode);
                    installer.install_module(&config, registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();
            }
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}
