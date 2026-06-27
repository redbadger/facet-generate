//! Keeps the generated-code examples in the top-level `README.md` in sync with
//! what the generators actually produce.
//!
//! For each language we reflect both `Point` (a struct) and `Shape` (an enum)
//! and emit the full generated module, stripping only language-level preamble
//! (import/package/using declarations). The result is embedded in the README
//! inside collapsible `<details>` blocks.
//!
//! Each language's output is compared against the fenced code block embedded in
//! the README between marker comments such as:
//!
//! ```text
//! <!-- generated:swift:start -->
//! ```swift
//! ...generated code...
//! ```
//! <!-- generated:swift:end -->
//! ```
//!
//! Run `UPDATE_EXPECT=1 cargo test -p facet_generate --test readme` (or
//! `just test`, which enables snapshot updates) to rewrite the README blocks
//! from the real generator output.

#![cfg(all(
    feature = "swift",
    feature = "kotlin",
    feature = "typescript",
    feature = "csharp"
))]

use std::{
    fs,
    path::{Path, PathBuf},
};

use facet::Facet;
use facet_generate::{
    Registry,
    generation::{bincode::BincodePlugin, csharp, kotlin, swift, typescript},
    reflection::RegistryBuilder,
};

#[derive(Facet)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Facet)]
#[repr(C)]
#[allow(dead_code)]
enum Shape {
    Circle {
        centre: Point,
        radius: f64,
    },
    Rectangle {
        position: Point,
        width: f64,
        height: f64,
    },
}

/// The README block we keep in sync, identified by the marker name and the
/// language used for the fenced code block.
struct Block {
    /// Marker name, e.g. `swift` in `<!-- generated:swift:start -->`.
    marker: &'static str,
    /// Language tag for the ```` ```lang ```` fence.
    fence: &'static str,
    /// The generated source code.
    code: String,
}

#[test]
fn readme_examples_are_up_to_date() {
    let registry = RegistryBuilder::new()
        .add_type::<Shape>()
        .unwrap()
        .build()
        .unwrap();

    let blocks = vec![
        Block {
            marker: "swift",
            fence: "swift",
            code: generate_swift(&registry),
        },
        Block {
            marker: "kotlin",
            fence: "kotlin",
            code: generate_kotlin(&registry),
        },
        Block {
            marker: "typescript",
            fence: "typescript",
            code: generate_typescript(&registry),
        },
        Block {
            marker: "csharp",
            fence: "csharp",
            code: generate_csharp(&registry),
        },
    ];

    let readme_path = readme_path();
    let original = fs::read_to_string(&readme_path).expect("failed to read README.md");

    let mut updated = original.clone();
    for block in &blocks {
        updated = replace_block(&updated, block);
    }

    if updated == original {
        return;
    }

    if update_enabled() {
        fs::write(&readme_path, updated).expect("failed to write README.md");
    } else {
        panic!(
            "README.md generated-code blocks are out of date.\n\
             Run `UPDATE_EXPECT=1 cargo test -p facet_generate --test readme` to update them."
        );
    }
}

fn update_enabled() -> bool {
    std::env::var_os("UPDATE_EXPECT").is_some() || std::env::var_os("UPDATE_README").is_some()
}

fn readme_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md")
}

// --- generation helpers ----------------------------------------------------

fn generate_swift(registry: &Registry) -> String {
    let dir = tempfile::tempdir().unwrap();
    swift::Installer::new("Example", dir.path())
        .plugin(BincodePlugin)
        .generate(registry)
        .unwrap();
    let source = find_source_with_token(dir.path(), "Point").expect("Swift module not found");
    strip_preamble(&source)
}

fn generate_kotlin(registry: &Registry) -> String {
    let dir = tempfile::tempdir().unwrap();
    kotlin::Installer::new("com.example", dir.path())
        .plugin(BincodePlugin)
        .generate(registry)
        .unwrap();
    let source = find_source_with_token(dir.path(), "Point").expect("Kotlin module not found");
    strip_preamble(&source)
}

fn generate_typescript(registry: &Registry) -> String {
    let dir = tempfile::tempdir().unwrap();
    typescript::Installer::new("example", dir.path())
        .plugin(BincodePlugin)
        .generate(registry)
        .unwrap();
    let source = find_source_with_token(dir.path(), "Point").expect("TypeScript module not found");
    strip_preamble(&source)
}

fn generate_csharp(registry: &Registry) -> String {
    let dir = tempfile::tempdir().unwrap();
    csharp::Installer::new("Example", dir.path())
        .plugin(BincodePlugin)
        .generate(registry)
        .unwrap();
    let source = find_source_with_token(dir.path(), "Point").expect("C# module not found");
    strip_preamble(&source)
}

/// Strips language preamble (import / package / using lines and blank lines
/// between them) from the start of a generated source file.
fn strip_preamble(source: &str) -> String {
    source
        .lines()
        .skip_while(|line| {
            let t = line.trim();
            t.is_empty()
                || t.starts_with("import ")
                || t.starts_with("package ")
                || t.starts_with("using ")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim_start()
        .to_string()
}

/// Walks `dir` and returns the contents of the first source file that contains
/// `token` as a word.
fn find_source_with_token(dir: &Path, token: &str) -> Option<String> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(contents) = fs::read_to_string(&path)
                && contents.contains(token)
            {
                return Some(contents);
            }
        }
    }
    None
}

// --- README rewriting -------------------------------------------------------

fn replace_block(readme: &str, block: &Block) -> String {
    let start_marker = format!("<!-- generated:{}:start -->", block.marker);
    let end_marker = format!("<!-- generated:{}:end -->", block.marker);

    let start = readme
        .find(&start_marker)
        .unwrap_or_else(|| panic!("missing `{start_marker}` in README.md"));
    let end = readme
        .find(&end_marker)
        .unwrap_or_else(|| panic!("missing `{end_marker}` in README.md"));
    assert!(
        end > start,
        "`{end_marker}` appears before `{start_marker}` in README.md"
    );

    let replacement = format!(
        "{start_marker}\n\n```{fence}\n{code}\n```\n\n",
        fence = block.fence,
        code = block.code,
    );

    let mut out = String::with_capacity(readme.len());
    out.push_str(&readme[..start]);
    out.push_str(&replacement);
    out.push_str(&readme[end..]);
    out
}
