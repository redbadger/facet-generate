use std::{
    fs,
    path::{Path, PathBuf},
};

use expect_test::ExpectFile;
use ignore::WalkBuilder;

mod basic;
mod with_imported_dependencies;
mod with_namespaces_as_dependencies;
mod with_namespaces_as_targets;
mod with_serialization;
mod with_serialization_and_serde_as_separate_package;
mod with_serialization_and_serde_dependency;
mod with_serialization_and_serde_target;

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
