use std::{
    collections::{HashMap, HashSet},
    fs,
};

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    generation::{
        bincode::BincodePlugin,
        kotlin, swift,
        tests::{TargetLanguage, check, read_files_and_create_expect_dirs},
        typescript,
    },
    reflect, source_dir,
};

#[test]
fn test1() {
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

    #[derive(Facet)]
    struct MyStruct {
        string_to_int: HashMap<String, i32>,
        map_to_list: HashMap<String, Vec<i32>>,
        option_of_vec_of_set: Option<Vec<HashSet<String>>>,
        parent: Parent,
    }

    let registry = reflect!(MyStruct).unwrap();

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
