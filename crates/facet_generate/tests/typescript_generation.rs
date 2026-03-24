#![cfg(feature = "typescript")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod common;

use facet::Facet;
use facet_generate::generation::{CodeGeneratorConfig, Encoding, SourceInstaller, typescript};
use facet_generate::reflection::RegistryBuilder;
use serde_json::Value;
use std::{collections::BTreeMap, fs::File, path::Path};
use tempfile::tempdir;

fn test_typescript_code_generates_with_config(
    dir_path: &Path,
    config: &CodeGeneratorConfig,
    encoding: Encoding,
) -> std::path::PathBuf {
    let registry = common::get_registry();
    std::fs::create_dir_all(dir_path.join("testing")).unwrap_or(());

    let mut installer = typescript::Installer::new("testing", dir_path);
    installer.install_serde_runtime().unwrap();

    let source_path = dir_path.join("testing").join("test.ts");
    let mut source = File::create(&source_path).unwrap();

    let generator = typescript::TypeScriptCodeGenerator::new(config).with_encoding(encoding);
    generator.output(&mut source, &registry).unwrap();

    dir_path.join("testing")
}

#[test]
fn test_typescript_code_generates_with_bincode() {
    let dir = tempdir().unwrap();
    let config = CodeGeneratorConfig::new("testing".to_string());
    test_typescript_code_generates_with_config(dir.path(), &config, Encoding::Bincode);
}

#[test]
fn test_typescript_code_generates_with_comments() {
    /// Some
    /// comments
    #[derive(Facet)]
    struct CommentedType {
        value: String,
    }

    let dir = tempdir().unwrap();
    let registry = RegistryBuilder::new()
        .add_type::<CommentedType>()
        .unwrap()
        .build()
        .unwrap();

    let config = CodeGeneratorConfig::new("testing".to_string());
    let source_path = dir.path().join("testing");
    std::fs::create_dir_all(&source_path).unwrap();
    let source_file = source_path.join("test.ts");
    let mut file = File::create(&source_file).unwrap();

    let generator = typescript::TypeScriptCodeGenerator::new(&config);
    generator.output(&mut file, &registry).unwrap();

    let content = std::fs::read_to_string(&source_file).unwrap();
    assert!(
        content.contains("/// Some\n/// comments\n"),
        "Doc comments should be present in output:\n{content}"
    );
}

#[test]
fn test_typescript_code_generates_with_external_definitions() {
    let dir = tempdir().unwrap();

    // create external definition
    std::fs::create_dir_all(dir.path().join("external")).unwrap_or(());
    std::fs::write(
        dir.path().join("external/index.ts"),
        "export const CustomType = 5;",
    )
    .unwrap();

    let mut external_definitions = BTreeMap::new();
    external_definitions.insert(String::from("external"), vec![String::from("CustomType")]);
    let config = CodeGeneratorConfig::new("testing".to_string())
        .with_external_definitions(external_definitions);

    test_typescript_code_generates_with_config(dir.path(), &config, Encoding::None);
}

#[test]
fn test_typescript_manifest_generation() {
    let dir = tempdir().unwrap();

    let installer = typescript::Installer::new("my-typescript-package", dir.path());
    installer.install_manifest("my-typescript-package").unwrap();

    // Check that package.json was created
    let manifest_path = dir.path().join("package.json");
    assert!(manifest_path.exists());

    // Check that package.json has correct content
    let manifest_content = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest: Value = serde_json::from_str(&manifest_content).unwrap();

    assert_eq!(manifest["name"], "my-typescript-package");
    assert_eq!(manifest["version"], "0.1.0");
    assert_eq!(manifest["devDependencies"]["typescript"], "^5.8.3");
}

#[test]
fn test_typescript_code_generation_file_layout() {
    let dir = tempdir().unwrap();
    let registry = common::get_registry();

    let config = CodeGeneratorConfig::new("testing".to_string());

    let mut installer =
        typescript::Installer::new("testing", dir.path()).encoding(Encoding::Bincode);
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();

    // Module is written as a flat .ts file (Node convention)
    let module_path = dir.path().join("testing.ts");
    assert!(module_path.exists());

    // Generated content uses extensionless serde import (Node convention)
    let content = std::fs::read_to_string(&module_path).unwrap();
    assert!(content.contains(r#"from "./serde""#));
    assert!(!content.contains(r#"from "./serde/mod.ts""#));

    // Runtime entry point is index.ts (Node convention)
    let serde_index = dir.path().join("serde").join("index.ts");
    assert!(serde_index.exists());

    // Runtime imports have .ts extensions stripped
    let serde_content = std::fs::read_to_string(&serde_index).unwrap();
    assert!(serde_content.contains("from \"./types\""));
    assert!(!serde_content.contains("from \"./types.ts\""));

    // Other serde files also have .ts stripped
    let binary_deserializer = dir.path().join("serde").join("binaryDeserializer.ts");
    assert!(binary_deserializer.exists());

    let binary_deserializer_content = std::fs::read_to_string(&binary_deserializer).unwrap();
    assert!(binary_deserializer_content.contains("from \"./deserializer\""));
    assert!(!binary_deserializer_content.contains("from \"./deserializer.ts\""));
}
