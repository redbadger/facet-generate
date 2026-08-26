use std::fs;

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{self as fg, source_dir};
use crate::{
    generation::{
        kotlin, swift,
        tests::{TargetLanguage, check, read_files_and_create_expect_dirs},
        typescript,
    },
    reflect,
};

/// Two *named* namespaces, with one referencing the other.
///
/// The other namespace fixtures declare a single namespace referenced only from
/// the root module, so none of them exercises a reference between two named
/// namespaces — the case where the referring module's own name is already
/// namespace-qualified. Kotlin got that wrong: it rooted the sibling's package
/// at the current module rather than at the shared parent, emitting
/// `com.example.feature.kit.Row` for a type declared in `com.example.kit`, so
/// the generated source did not compile.
#[test]
fn test() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    pub struct Row {
        label: String,
    }

    // Lives in a *different* named namespace and holds a `kit` type, so the
    // emitted reference has to reach sideways rather than downwards.
    #[derive(Facet)]
    #[facet(fg::namespace = "feature")]
    pub struct FeatureView {
        rows: Vec<Row>,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Root {
        Feature(FeatureView),
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
}
