use std::fs;

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    self as fg,
    generation::{
        bincode::BincodePlugin,
        kotlin,
        tests::{TargetLanguage, check, read_files_and_create_expect_dirs},
    },
    reflect, source_dir,
};

#[test]
fn test() {
    #[derive(Facet)]
    struct StructWithBytes {
        #[facet(fg::bytes)]
        data: Vec<u8>,
        name: String,
    }

    let registry = reflect!(StructWithBytes).unwrap();

    let this_dir = source_dir!().join("snapshots");

    for target in [TargetLanguage::Kotlin] {
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
            _ => unreachable!(),
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}
