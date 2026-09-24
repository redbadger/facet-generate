use std::fs;

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{self as fg, source_dir};
use crate::{
    generation::{
        csharp, kotlin, swift,
        tests::{TargetLanguage, check, read_files_and_create_expect_dirs},
        typescript,
    },
    reflect,
};

/// Two different Rust types with the same name, one explicitly in namespace `a` and one inherited
/// into namespace `b` (#138).
///
/// They are two types, not one type with an ambiguous namespace, so each is generated in its own
/// namespace, both under the name `Child`.
#[test]
fn test() {
    mod one {
        use crate as fg;
        use facet::Facet;

        #[derive(Facet)]
        #[facet(fg::namespace = "a")]
        pub struct Child {
            pub x: u8,
        }
    }
    mod two {
        use facet::Facet;

        #[derive(Facet)]
        pub struct Child {
            pub y: u8,
        }
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "b")]
    pub struct Parent {
        first: one::Child,
        second: two::Child, // inherits "b"
    }

    #[derive(Facet)]
    pub struct Root {
        parent: Parent,
    }

    let registry = reflect!(Root).unwrap();

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
                    .generate(&registry)
                    .unwrap();
            }
            TargetLanguage::Swift => {
                swift::Installer::new("Example", tmp_path)
                    .generate(&registry)
                    .unwrap();
            }
            TargetLanguage::TypeScript => {
                typescript::Installer::new("example", tmp_path)
                    .generate(&registry)
                    .unwrap();
            }
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }

    // C# is not one of the `TargetLanguage`s the other fixtures loop over.
    let tmp_dir = tempdir().unwrap();
    let tmp_path = tmp_dir.path();
    let snapshot_dir = this_dir.join("csharp");
    fs::create_dir_all(&snapshot_dir).unwrap();
    csharp::Installer::new("Example", tmp_path)
        .generate(&registry)
        .unwrap();
    for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
        check(&actual, &expect_file!(&expected));
    }
}
