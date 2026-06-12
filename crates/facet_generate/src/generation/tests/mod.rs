use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    fs,
    path::{Path, PathBuf},
};

use expect_test::{ExpectFile, expect_file};
use ignore::WalkBuilder;

/// Target languages included in snapshot integration tests. Used to loop over languages
/// in a single test, dispatching to the appropriate installer for each.
pub enum TargetLanguage {
    Kotlin,
    Swift,
    TypeScript,
}

impl Display for TargetLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kotlin => write!(f, "Kotlin"),
            Self::Swift => write!(f, "Swift"),
            Self::TypeScript => write!(f, "TypeScript"),
        }
    }
}

mod basic;
mod with_bytes;
mod with_namespaces_as_external;
mod with_namespaces_as_internal;
mod with_serialization;
mod with_serialization_and_namespaces_as_external;
mod with_serialization_and_serde_external;
mod with_serialization_and_serde_internal;
mod with_serialization_and_serde_local;
mod with_uuid;

fn read_files_and_create_expect_dirs(
    tmp_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Vec<(String, PathBuf)> {
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

            fs::create_dir_all(expected.parent().unwrap()).unwrap();

            let actual = fs::read_to_string(entry.path()).unwrap();

            files.push((actual, expected));
        }
    }
    files
}

pub fn check_roots(sut_root: impl AsRef<Path>, snapshot_root: impl AsRef<Path>) {
    let mut all_expected_paths = all_files(&snapshot_root);

    for (actual, expected) in read_files_and_create_expect_dirs(&sut_root, &snapshot_root) {
        check(&actual, &expect_file!(&expected));

        all_expected_paths.remove(
            expected
                .as_path()
                .strip_prefix(&snapshot_root.as_ref())
                .unwrap(),
        );
    }
    // TODO: uncomment this assert once tests cover any necessary variant
    // assert!(
    //     all_expected_paths.is_empty(),
    //     "missing expected files from the snapshots:\n{}",
    //     all_expected_paths
    //         .into_iter()
    //         .filter_map(|path| {
    //             let to_string = path.display().to_string();
    //             (!to_string.is_empty()).then_some(to_string)
    //         })
    //         .collect::<Vec<_>>()
    //         .join("\n")
    // );
}

fn all_files(root: impl AsRef<Path>) -> HashSet<PathBuf> {
    WalkBuilder::new(&root)
        .hidden(false)
        .follow_links(true)
        .build()
        .filter_map(|dir_entry| {
            dir_entry
                .map(|entry| {
                    let path = entry
                        .path()
                        .to_owned()
                        .strip_prefix(&root)
                        .map(std::path::Path::to_path_buf);
                    // skip directories from the list
                    (entry.metadata().unwrap().is_file()).then_some(path)
                })
                .unwrap()
        })
        .collect::<Result<HashSet<_>, _>>()
        .unwrap()
}

fn check(actual: &str, expect: &ExpectFile) {
    expect.assert_eq(actual);
}
