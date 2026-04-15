use std::fs;

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    generation::{
        Encoding, ExternalPackage, PackageLocation, SourceInstaller as _, kotlin, module,
        swift::Installer,
        tests::{TargetLanguage, check, read_files_and_create_expect_dirs},
        typescript,
    },
    reflect, source_dir,
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

    let registry = reflect!(Parent).unwrap();

    let this_dir = source_dir!().join("snapshots");

    for target in [
        TargetLanguage::Kotlin,
        TargetLanguage::Swift,
        TargetLanguage::TypeScript,
    ] {
        let tmp_dir = tempdir().unwrap();
        let tmp_path = tmp_dir.path();

        let snapshot_dir = this_dir.join(target.to_string().to_lowercase());
        fs::create_dir_all(&snapshot_dir).unwrap();

        #[allow(clippy::match_same_arms)]
        match target {
            TargetLanguage::Kotlin => {
                let package_name = "com.example";
                let mut installer = kotlin::Installer::new(package_name, tmp_path)
                    .encoding(Encoding::Bincode)
                    .external_packages(&[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("com.novi.serde".to_string()),
                        module_name: None,
                        version: None,
                    }]);
                installer.install_serde_runtime().unwrap();
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone().with_parent(package_name);
                    installer.install_module(&config, registry).unwrap();
                }
            }
            TargetLanguage::Swift => {
                let package_name = "Example";
                let mut installer = Installer::new(package_name, tmp_path.join(package_name))
                    .encoding(Encoding::Bincode)
                    .external_packages(&[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("../Serde".to_string()),
                        module_name: None,
                        version: None,
                    }]);
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone();
                    installer.install_module(&config, registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();

                let package_name = "Serde";
                let mut installer = Installer::new(package_name, tmp_path.join(package_name));
                installer.install_serde_runtime().unwrap();
                installer.install_manifest(package_name).unwrap();
            }
            TargetLanguage::TypeScript => {
                let package_name = "example";
                let mut installer =
                    typescript::Installer::new(package_name, tmp_path.join(package_name))
                        .encoding(Encoding::Bincode)
                        .external_packages(&[ExternalPackage {
                            for_namespace: "serde".to_string(),
                            location: PackageLocation::Path("../serde".to_string()),
                            module_name: None,
                            version: None,
                        }]);

                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config().clone();
                    installer.install_module(&config, registry).unwrap();
                }
                installer.install_manifest(package_name).unwrap();

                let package_name = "serde";
                let mut installer =
                    typescript::Installer::new(package_name, tmp_path.join(package_name));
                installer.install_serde_runtime().unwrap();
                installer.install_manifest(package_name).unwrap();
            }
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}
