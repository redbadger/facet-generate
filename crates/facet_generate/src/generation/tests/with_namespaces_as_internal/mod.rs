use std::fs;

use facet::Facet;
use tempfile::tempdir;

use crate::{self as fg, source_dir};
use crate::{
    generation::{
        kotlin, swift,
        tests::{TargetLanguage, check_roots},
        typescript,
    },
    reflect,
};

#[test]
fn test() {
    #[derive(Facet)]
    #[facet(fg::namespace = "other")]
    pub struct OtherChild {
        name: String,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[facet(fg::namespace = "other")]
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

        check_roots(tmp_path, snapshot_dir);
    }
}
