#![cfg(feature = "csharp")]

use std::process::Command;

use facet::Facet;
use facet_generate as fg;
use facet_generate::{
    generation::{bincode::BincodePlugin, csharp, json::JsonPlugin},
    reflect,
};
use serde::{Deserialize, Serialize};
use tempfile::{TempDir, tempdir};

pub mod common;

fn dotnet_build(dir: &TempDir) {
    let status = Command::new("dotnet")
        .arg("build")
        .current_dir(dir)
        .env("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1")
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .env("DOTNET_NOLOGO", "1")
        .status()
        .unwrap();
    assert!(status.success(), "dotnet build failed");
}

#[test]
fn test_that_csharp_code_compiles_with_bincode() {
    let registry = common::get_registry();
    let dir = tempdir().unwrap();

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    dotnet_build(&dir);
}

#[test]
fn test_that_csharp_code_with_keyword_fields_compiles_with_bincode() {
    #[derive(Facet)]
    #[allow(clippy::struct_excessive_bools)]
    struct KeywordFields {
        event: bool,
        lock: bool,
        class: bool,
        namespace: bool,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum KeywordVariant {
        Values { event: bool, lock: bool },
    }

    let registry = reflect!(KeywordFields, KeywordVariant).unwrap();
    let dir = tempdir().unwrap();

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    dotnet_build(&dir);
}

#[test]
fn test_that_csharp_code_compiles_with_json() {
    let registry = common::get_registry();
    let dir = tempdir().unwrap();

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(JsonPlugin)
        .generate(&registry)
        .unwrap();

    dotnet_build(&dir);
}

#[test]
fn test_that_csharp_code_compiles_without_serialization() {
    #[derive(Facet)]
    struct Child {
        name: String,
        age: u32,
        tags: Vec<String>,
        nickname: Option<String>,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Parent {
        Single(Child),
        Pair { left: Child, right: Child },
        Empty,
    }

    let registry = reflect!(Parent).unwrap();
    let dir = tempdir().unwrap();

    csharp::Installer::new("Example.Testing", &dir)
        .generate(&registry)
        .unwrap();

    dotnet_build(&dir);
}

#[test]
fn test_that_csharp_code_compiles_with_primitive_types() {
    type Test = common::PrimitiveTypes;

    let registry = reflect!(Test).unwrap();
    let dir = tempdir().unwrap();

    csharp::Installer::new("Example.Testing", &dir)
        .generate(&registry)
        .unwrap();

    dotnet_build(&dir);
}

#[test]
fn test_csharp_code_with_external_definitions() {
    #[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
    #[repr(C)]
    pub enum TestData {
        Tree(#[facet(fg::namespace = "foo")] common::Tree<Box<common::SerdeData>>),
        SerdeData(common::SerdeData),
    }

    let registry = reflect!(TestData).unwrap();
    let dir = tempdir().unwrap();
    let source_dir = dir.path().to_path_buf().join("testing");

    let generator = csharp::Installer::new("Example.Testing", &source_dir);

    // Just verify code generation succeeds with external namespaces.
    // We can't compile because the external types don't have real implementations,
    // but we can verify the generated source references them correctly.
    generator.generate(&registry).unwrap();

    let generated = std::fs::read_to_string(source_dir.join("Example/Testing/Testing.cs")).unwrap();
    assert!(
        generated.contains("Example.Testing.Foo"),
        "Generated code should reference external namespace: {generated}"
    );
}

/// A unit-only enum from another namespace goes through its helper class,
/// qualified like the type (`Example.Kit.PresenceBincode`), rather than being
/// serialized as if it were a class (#154).
///
/// A reference between two named namespaces (`common::across_namespaces::
/// get_sibling_registry`) is not compiled: the type itself is still rooted at
/// the wrong namespace there (#149).
#[test]
fn test_that_csharp_code_with_enums_from_other_namespaces_compiles() {
    let registry = common::across_namespaces::get_registry();

    let dir = tempdir().unwrap();
    csharp::Installer::new("Example", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();
    dotnet_build(&dir);

    let dir = tempdir().unwrap();
    csharp::Installer::new("Example", &dir)
        .plugin(JsonPlugin)
        .generate(&registry)
        .unwrap();
    dotnet_build(&dir);
}
