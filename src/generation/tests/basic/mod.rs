use std::{env, fs, path::PathBuf};

use expect_test::expect_file;
use facet::Facet;
use ignore::WalkBuilder;
use tempfile::TempDir;

use crate::{
    generation::{SourceInstaller as _, java, module, swift, typescript},
    reflection::RegistryBuilder,
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

    let registry = RegistryBuilder::new().add_type::<Parent>().build();
    let registries = module::split("root", &registry);

    let this_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(file!())
        .parent()
        .unwrap()
        .join("snapshots");

    for target in ["typescript", "swift", "java"] {
        let out_dir = this_dir.join(target);
        fs::create_dir_all(&out_dir).unwrap();

        for (module, registry) in &registries {
            let config = module.config();

            let module_name = config.module_name();

            let mut source = Vec::new();
            match target {
                "typescript" => {
                    let generator = typescript::CodeGenerator::new(config);
                    generator.output(&mut source, registry).unwrap();
                    let file = out_dir.join(format!("{module_name}.ts"));
                    expect_file!(&file).assert_eq(&String::from_utf8(source).unwrap());
                }
                "swift" => {
                    let generator = swift::CodeGenerator::new(config);
                    generator.output(&mut source, registry).unwrap();
                    let file = out_dir.join(format!("{module_name}.swift"));
                    expect_file!(&file).assert_eq(&String::from_utf8(source).unwrap());
                }
                "java" => {
                    let tmp_dir = TempDir::new().unwrap();
                    let tmp_path = tmp_dir.path();
                    let mut installer = java::Installer::new(tmp_path.to_path_buf());
                    installer.install_module(config, registry).unwrap();

                    for entry in WalkBuilder::new(tmp_path)
                        .hidden(false)
                        .follow_links(true)
                        .build()
                    {
                        if let Ok(entry) = entry
                            && let Some(file_type) = entry.file_type()
                            && file_type.is_file()
                        {
                            let relative_path = entry.path().strip_prefix(tmp_path).unwrap();
                            let expected = out_dir.join(relative_path);
                            fs::create_dir_all(out_dir.join(expected.parent().unwrap())).unwrap();

                            let actual = fs::read_to_string(entry.path()).unwrap();

                            expect_file!(&expected).assert_eq(&actual);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
