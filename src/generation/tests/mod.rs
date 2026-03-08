use std::{
    fmt::{self, Display, Formatter},
    fs,
    path::{Path, PathBuf},
};

use expect_test::ExpectFile;
use ignore::WalkBuilder;

/// Target languages included in snapshot integration tests. Used to loop over languages
/// in a single test, dispatching to the appropriate installer for each.
pub enum TargetLanguage {
    #[deprecated(
        since = "0.16.0",
        note = "The Java generator is deprecated. Use Kotlin instead."
    )]
    Java,
    Kotlin,
    Swift,
    TypeScript,
}

#[expect(
    deprecated,
    reason = "Display must handle all variants including deprecated TargetLanguage::Java"
)]
impl Display for TargetLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TargetLanguage::Java => write!(f, "Java"),
            TargetLanguage::Kotlin => write!(f, "Kotlin"),
            TargetLanguage::Swift => write!(f, "Swift"),
            TargetLanguage::TypeScript => write!(f, "TypeScript"),
        }
    }
}

#[expect(
    deprecated,
    reason = "snapshot tests cover the deprecated Java generator"
)]
mod basic;
mod with_bytes;
#[expect(
    deprecated,
    reason = "snapshot tests cover the deprecated Java generator"
)]
mod with_namespaces_as_external;
#[expect(
    deprecated,
    reason = "snapshot tests cover the deprecated Java generator"
)]
mod with_namespaces_as_internal;
#[expect(
    deprecated,
    reason = "snapshot tests cover the deprecated Java generator"
)]
mod with_serialization;
#[expect(
    deprecated,
    reason = "snapshot tests cover the deprecated Java generator"
)]
mod with_serialization_and_namespaces_as_external;
#[expect(
    deprecated,
    reason = "snapshot tests cover the deprecated Java generator"
)]
mod with_serialization_and_serde_external;
#[expect(
    deprecated,
    reason = "snapshot tests cover the deprecated Java generator"
)]
mod with_serialization_and_serde_internal;
#[expect(
    deprecated,
    reason = "snapshot tests cover the deprecated Java generator"
)]
mod with_serialization_and_serde_local;

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

fn check(actual: &str, expect: &ExpectFile) {
    expect.assert_eq(actual);
}
