//! Keeps the generated-code examples in the top-level `README.md` in sync with
//! what the generators actually produce.
//!
//! For Swift, Kotlin, and C# we reflect `Point` (a representative struct) and
//! extract just its declaration. For TypeScript we show the full generated
//! module (struct + enum), because TypeScript enums now produce a discriminated
//! union type rather than a class and the declaration-extraction logic does not
//! handle that format.
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
    /// The generated source for the `HttpHeader` declaration.
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
    extract_declaration(dir.path(), "Point")
}

fn generate_kotlin(registry: &Registry) -> String {
    let dir = tempfile::tempdir().unwrap();
    kotlin::Installer::new("com.example", dir.path())
        .plugin(BincodePlugin)
        .generate(registry)
        .unwrap();
    extract_declaration(dir.path(), "Point")
}

/// For TypeScript we show the full generated module (minus import statements)
/// because enums now produce discriminated union types rather than classes, and
/// the brace-balanced `extract_declaration` helper cannot handle that format.
fn generate_typescript(registry: &Registry) -> String {
    let dir = tempfile::tempdir().unwrap();
    typescript::Installer::new("example", dir.path())
        .plugin(BincodePlugin)
        .generate(registry)
        .unwrap();
    let source = find_source_with_declaration(dir.path(), "Point")
        .expect("generated TypeScript module not found");
    source
        .lines()
        .skip_while(|line| line.starts_with("import "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_start()
        .to_string()
}

fn generate_csharp(registry: &Registry) -> String {
    let dir = tempfile::tempdir().unwrap();
    csharp::Installer::new("Example", dir.path())
        .plugin(BincodePlugin)
        .generate(registry)
        .unwrap();
    extract_declaration(dir.path(), "Point")
}

/// Walks `dir`, finds the generated module file that declares `type_name`, and
/// returns the dedented source for just that declaration (including any
/// immediately-preceding attribute/doc-comment lines).
fn extract_declaration(dir: &Path, type_name: &str) -> String {
    let source = find_source_with_declaration(dir, type_name).unwrap_or_else(|| {
        panic!(
            "no generated file declares `{type_name}` under {}",
            dir.display()
        )
    });
    extract_block(&source, type_name)
}

fn find_source_with_declaration(dir: &Path, type_name: &str) -> Option<String> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(contents) = fs::read_to_string(&path)
                && declares_type(&contents, type_name)
            {
                return Some(contents);
            }
        }
    }
    None
}

fn declares_type(source: &str, type_name: &str) -> bool {
    source.lines().any(|line| is_declaration(line, type_name))
}

fn is_declaration(line: &str, type_name: &str) -> bool {
    const KEYWORDS: [&str; 4] = ["struct ", "class ", "interface ", "enum "];
    KEYWORDS.iter().any(|kw| line.contains(kw)) && line.contains(type_name)
}

/// Extracts the brace-balanced block declaring `type_name`, including any
/// directly preceding attribute (`[...]`) or doc-comment (`///`) lines, then
/// removes common leading indentation.
fn extract_block(source: &str, type_name: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let decl = lines
        .iter()
        .position(|line| is_declaration(line, type_name))
        .expect("declaration line not found");

    // Include directly-preceding attribute / doc-comment lines.
    let mut start = decl;
    while start > 0 {
        let prev = lines[start - 1].trim_start();
        if prev.starts_with("///") || prev.starts_with('[') {
            start -= 1;
        } else {
            break;
        }
    }

    let mut depth = 0i32;
    let mut seen_brace = false;
    let mut end = decl;
    for (offset, line) in lines[decl..].iter().enumerate() {
        for ch in line.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    seen_brace = true;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        if seen_brace && depth == 0 {
            end = decl + offset;
            break;
        }
    }

    dedent(&lines[start..=end])
}

fn dedent(lines: &[&str]) -> String {
    let indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|line| {
            if line.len() >= indent {
                &line[indent..]
            } else {
                line.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
