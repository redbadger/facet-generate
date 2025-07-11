use std::{
    env, fs,
    path::{Path, PathBuf},
};

use expect_test::{ExpectFile, expect_file};
use facet::Facet;
use ignore::WalkBuilder;
use tempfile::TempDir;

use crate::{
    generation::{
        SourceInstaller as _, java,
        module::{self, Module},
        swift, typescript,
    },
    reflection::RegistryBuilder,
};

#[test]
fn test() {
    #[derive(Facet)]
    #[facet(namespace = "other")]
    pub struct OtherChild {
        name: String,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[facet(namespace = "other")]
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

    let registry = RegistryBuilder::new().add_type::<Parent>().build();

    let this_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(file!())
        .parent()
        .unwrap()
        .join("snapshots");

    for target in ["java", "swift", "typescript"] {
        let tmp_dir = TempDir::new().unwrap();
        let tmp_path = tmp_dir.path();

        let snapshot_dir = this_dir.join(target);
        fs::create_dir_all(&snapshot_dir).unwrap();

        match target {
            "java" => {
                let package_name = "com.example";
                let mut installer = java::Installer::new(tmp_path.to_path_buf());
                for (module, registry) in &module::split(package_name, &registry) {
                    let this_module = &module.config().module_name;
                    let is_root_package = package_name == this_module;
                    let module = if is_root_package {
                        module
                    } else {
                        &Module::new([package_name, this_module].join("."))
                    };
                    let config = module.config();
                    installer.install_module(config, registry).unwrap();
                }
            }
            "swift" => {
                let package_name = "Example";
                let mut installer =
                    swift::Installer::new(package_name.to_string(), tmp_path.to_path_buf(), None);
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config();
                    installer.install_module(config, registry).unwrap();
                    installer.install_manifest(package_name).unwrap();
                }
            }
            "typescript" => {
                let package_name = "example";
                let mut installer = typescript::Installer::new(tmp_path.to_path_buf());
                for (module, registry) in &module::split(package_name, &registry) {
                    let config = module.config();
                    installer.install_module(config, registry).unwrap();
                }
            }
            _ => unreachable!(),
        }

        for (actual, expected) in find_files(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}

fn find_files(tmp_path: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> Vec<(String, PathBuf)> {
    let mut files = Vec::new();
    for entry in WalkBuilder::new(&tmp_path)
        .hidden(false)
        .follow_links(true)
        .build()
    {
        if let Ok(entry) = entry
            && let Some(file_type) = entry.file_type()
            && file_type.is_file()
        {
            let relative_path = entry.path().strip_prefix(&tmp_path).unwrap();
            let expected = out_dir.as_ref().join(relative_path);
            fs::create_dir_all(out_dir.as_ref().join(expected.parent().unwrap())).unwrap();

            let actual = fs::read_to_string(entry.path()).unwrap();

            files.push((actual, expected));
        }
    }
    files
}

fn check(actual: &str, expect: &ExpectFile) {
    expect.assert_eq(actual);
}
