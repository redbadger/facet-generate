// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

mod common;

use facet_generate::generation::{
    CodeGeneratorConfig, Encoding, Serialization, SourceInstaller,
    typescript::{self, InstallTarget},
};
use regex::Regex;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::File,
    path::Path,
    process::{Command, Stdio},
};
use tempfile::tempdir;

fn test_typescript_code_compiles_with_config(
    dir_path: &Path,
    config: &CodeGeneratorConfig,
) -> std::path::PathBuf {
    let registry = common::get_registry();
    make_output_file(dir_path);

    let mut installer = typescript::Installer::new(dir_path, &[], InstallTarget::Deno);
    installer.install_serde_runtime().unwrap();
    assert_deno_info(dir_path.join("serde/mod.ts").as_path());

    installer.install_bcs_runtime().unwrap();
    assert_deno_info(dir_path.join("bcs/mod.ts").as_path());

    let source_path = dir_path.join("testing").join("test.ts");
    let mut source = File::create(&source_path).unwrap();

    let generator = typescript::CodeGenerator::new(config, InstallTarget::Deno);
    generator.output(&mut source, &registry).unwrap();

    assert_deno_info(&source_path);
    dir_path.join("testing")
}

fn assert_deno_info(ts_path: &Path) {
    let output = Command::new("deno")
        .arg("info")
        .arg(ts_path)
        .stderr(Stdio::inherit())
        .output()
        .expect("deno info failed, is deno installed? brew install deno");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        !is_error_output(stdout.as_str()),
        "deno info detected an error\n{stdout}"
    );
}

fn is_error_output(output: &str) -> bool {
    let re = Regex::new(r"\berror\b").unwrap();
    re.is_match(output)
}

fn make_output_file(dir: &Path) {
    std::fs::create_dir_all(dir.join("testing")).unwrap_or(());
}

#[test]
fn test_is_error_output() {
    let table = vec![
        (
            "file:///var/folders/l0/x592_pjj18n6r2m0nqn05vmc0000gn/T/.tmp5NPlE2/serde/mod.ts (176B)",
            false,
        ),
        (
            "https://deno.land/std@0.85.0/node/_errors.ts (60.89KB)",
            false,
        ),
        (
            "error: Cannot resolve module \"file:///var/folders/l0/x592_pjj18n6r2m0nqn05vmc0000gn/T/.tmp5NPlE2/bcs/mod.ts\"",
            true,
        ),
        (
            "file:///var/folders/l0/x592_pjj18n6r2m0nqn05vmc0000gn/T/.tmpG1an6c/something/noSerializer.ts (error)",
            true,
        ),
    ];

    for (input, expectation) in table {
        assert_eq!(is_error_output(input), expectation);
    }
}

#[test]
fn test_typescript_code_compiles_with_bcs() {
    let dir = tempdir().unwrap();
    let config = CodeGeneratorConfig::new("testing".to_string()).with_encodings([Encoding::Bcs]);
    test_typescript_code_compiles_with_config(dir.path(), &config);
}

#[test]
fn test_typescript_code_compiles_with_comments() {
    let dir = tempdir().unwrap();
    let comments = vec![(vec!["SerdeData".to_string()], "Some\ncomments".to_string())]
        .into_iter()
        .collect();
    let config = CodeGeneratorConfig::new("testing".to_string()).with_comments(comments);

    let path = test_typescript_code_compiles_with_config(dir.path(), &config);
    // Comment was correctly generated.
    let content = std::fs::read_to_string(path.join("test.ts")).unwrap();
    assert!(content.contains(
        r"
/**
 * Some
 * comments
 */
"
    ));
}

#[test]
fn test_typescript_code_compiles_with_external_definitions() {
    let dir = tempdir().unwrap();

    // create external definition
    std::fs::create_dir_all(dir.path().join("external")).unwrap_or(());
    std::fs::write(
        dir.path().join("external/mod.ts"),
        "export const CustomType = 5;",
    )
    .unwrap();

    let mut external_definitions = BTreeMap::new();
    external_definitions.insert(String::from("external"), vec![String::from("CustomType")]);
    let config = CodeGeneratorConfig::new("testing".to_string())
        .with_external_definitions(external_definitions);

    test_typescript_code_compiles_with_config(dir.path(), &config);
}

#[test]
fn test_typescript_manifest_generation() {
    let dir = tempdir().unwrap();

    let installer = typescript::Installer::new(dir.path(), &[], InstallTarget::Deno);
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
fn test_typescript_code_generation_without_extensions() {
    let dir = tempdir().unwrap();
    let registry = common::get_registry();

    // Test with extensionless_imports = true
    let config = CodeGeneratorConfig::new("testing".to_string())
        .with_encodings([Encoding::Bcs])
        .with_serialization(Serialization::Bcs);

    let mut installer = typescript::Installer::new(dir.path(), &[], InstallTarget::Node);
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    // Check that the generated module file is index.ts instead of mod.ts
    let module_path = dir.path().join("testing").join("index.ts");
    assert!(module_path.exists());

    // Check that the generated content doesn't have .ts extensions in imports
    let content = std::fs::read_to_string(&module_path).unwrap();
    assert!(content.contains("from '../serde'"));
    assert!(content.contains("from '../bcs'"));
    assert!(!content.contains("from '../serde/mod.ts'"));
    assert!(!content.contains("from '../bcs/mod.ts'"));

    // Check that runtime files were transformed
    let serde_index = dir.path().join("serde").join("index.ts");
    assert!(serde_index.exists());

    let serde_content = std::fs::read_to_string(&serde_index).unwrap();
    assert!(serde_content.contains("from \"./types\""));
    assert!(!serde_content.contains("from \"./types.ts\""));

    // Check that other serde files were also transformed
    let binary_deserializer = dir.path().join("serde").join("binaryDeserializer.ts");
    assert!(binary_deserializer.exists());

    let binary_deserializer_content = std::fs::read_to_string(&binary_deserializer).unwrap();
    assert!(binary_deserializer_content.contains("from \"./deserializer\""));
    assert!(!binary_deserializer_content.contains("from \"./deserializer.ts\""));
}

#[test]
fn test_typescript_code_generation_with_extensions() {
    let dir = tempdir().unwrap();
    let registry = common::get_registry();

    // Test with extensionless_imports = false (default)
    let config = CodeGeneratorConfig::new("testing".to_string())
        .with_encodings([Encoding::Bcs])
        .with_serialization(Serialization::Bcs);

    let mut installer = typescript::Installer::new(dir.path(), &[], InstallTarget::Deno);
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    // Check that the generated module file is mod.ts
    let module_path = dir.path().join("testing").join("mod.ts");
    assert!(module_path.exists());

    // Check that the generated content has .ts extensions in imports
    let content = std::fs::read_to_string(&module_path).unwrap();
    assert!(content.contains("from '../serde/mod.ts'"));
    assert!(content.contains("from '../bcs/mod.ts'"));

    // Check that runtime files kept original structure
    let serde_mod = dir.path().join("serde").join("mod.ts");
    assert!(serde_mod.exists());

    let serde_content = std::fs::read_to_string(&serde_mod).unwrap();
    assert!(serde_content.contains("from \"./types.ts\""));
}
