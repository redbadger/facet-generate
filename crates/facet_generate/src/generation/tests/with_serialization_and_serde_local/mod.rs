use std::fs;

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    generation::{
        Encoding, ExternalPackage, PackageLocation, kotlin, swift,
        tests::{TargetLanguage, check, read_files_and_create_expect_dirs},
        typescript,
    },
    reflect, source_dir,
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

        match target {
            TargetLanguage::Kotlin => {
                kotlin::Installer::new("com.example", tmp_path)
                    .encoding(Encoding::Bincode)
                    .external_packages(&[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("com.novi.serde".to_string()),
                        module_name: None,
                        version: None,
                    }])
                    .generate(&registry)
                    .unwrap();
            }
            TargetLanguage::Swift => {
                swift::Installer::new("Example", tmp_path)
                    .encoding(Encoding::Bincode)
                    .external_packages(&[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("../Serde".to_string()),
                        module_name: None,
                        version: None,
                    }])
                    .generate(&registry)
                    .unwrap();
            }
            TargetLanguage::TypeScript => {
                typescript::Installer::new("example", tmp_path)
                    .encoding(Encoding::Bincode)
                    .external_packages(&[ExternalPackage {
                        for_namespace: "serde".to_string(),
                        location: PackageLocation::Path("./serde".to_string()),
                        module_name: None,
                        version: None,
                    }])
                    .generate(&registry)
                    .unwrap();
            }
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}
