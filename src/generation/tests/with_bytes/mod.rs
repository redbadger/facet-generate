use std::{env, fs, path::PathBuf};

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    generation::{
        Encoding, Language, SourceInstaller as _, kotlin, module,
        tests::{check, read_files_and_create_expect_dirs},
    },
    reflect,
};

#[test]
fn test() {
    #[derive(Facet)]
    struct StructWithBytes {
        #[facet(bytes)]
        data: Vec<u8>,
        name: String,
    }

    let registry = reflect!(StructWithBytes).unwrap();

    let this_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(file!())
        .parent()
        .unwrap()
        .join("snapshots");

    for target in [Language::Kotlin] {
        let tmp_dir = tempdir().unwrap();
        let tmp_path = tmp_dir.path();

        let snapshot_dir = this_dir.join(target.to_string().to_lowercase());
        fs::create_dir_all(&snapshot_dir).unwrap();

        match target {
            Language::Kotlin => {
                let package_name = "com.example";
                let mut installer = kotlin::Installer::new(package_name, tmp_path, &[]);
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module
                        .config()
                        .clone()
                        .with_parent(package_name)
                        .with_encoding(Encoding::Bincode);
                    installer.install_module(&config, registry).unwrap();
                }
            }
            _ => unreachable!(),
        }

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}
