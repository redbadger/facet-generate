use std::fs;

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;
use uuid::Uuid;

use crate::{
    generation::{
        bincode::BincodePlugin,
        json::JsonPlugin,
        kotlin, swift,
        tests::{TargetLanguage, check, read_files_and_create_expect_dirs},
        typescript,
    },
    reflect, source_dir,
};

/// A struct with a required and an optional UUID field, exercising both
/// bincode (16 raw bytes) and JSON (hyphenated string) encoding paths.
#[test]
fn test_bincode() {
    #[derive(Facet)]
    struct StructWithUuid {
        id: Uuid,
        parent_id: Option<Uuid>,
        name: String,
    }

    let registry = reflect!(StructWithUuid).unwrap();

    let this_dir = source_dir!().join("snapshots_bincode");

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
                    .plugin(BincodePlugin)
                    .generate(&registry)
                    .unwrap();
            }
            TargetLanguage::Swift => {
                swift::Installer::new("Example", tmp_path)
                    .plugin(BincodePlugin)
                    .generate(&registry)
                    .unwrap();
            }
            TargetLanguage::TypeScript => {
                typescript::Installer::new("example", tmp_path)
                    .plugin(BincodePlugin)
                    .generate(&registry)
                    .unwrap();
            }
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}

#[test]
fn test_json() {
    #[derive(Facet)]
    struct StructWithUuid {
        id: Uuid,
        parent_id: Option<Uuid>,
        name: String,
    }

    let registry = reflect!(StructWithUuid).unwrap();

    let this_dir = source_dir!().join("snapshots_json");

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
                    .plugin(JsonPlugin)
                    .generate(&registry)
                    .unwrap();
            }
            TargetLanguage::Swift => {
                swift::Installer::new("Example", tmp_path)
                    .plugin(JsonPlugin)
                    .generate(&registry)
                    .unwrap();
            }
            TargetLanguage::TypeScript => {
                typescript::Installer::new("example", tmp_path)
                    .plugin(JsonPlugin)
                    .generate(&registry)
                    .unwrap();
            }
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}
