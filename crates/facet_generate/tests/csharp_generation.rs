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
fn test_that_csharp_code_compiles_with_optional_c_style_enums() {
    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum ContactGroup {
        Align,
        Partner,
    }

    #[derive(Facet)]
    struct ContactFilter {
        group: Option<ContactGroup>,
        groups: Vec<Option<ContactGroup>>,
    }

    let registry = reflect!(ContactFilter).unwrap();
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

/// A type named `Set` must not break the generated namespace, and the
/// `using`-imported collection types must stay reachable.
#[test]
fn test_that_csharp_code_shadowing_builtin_names_compiles_with_bincode() {
    let registry = common::get_shadowing_registry();
    let dir = tempdir().unwrap();

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    dotnet_build(&dir);
}

/// A unit-only enum from another namespace goes through its helper class,
/// qualified like the type (`Example.Kit.PresenceBincode`), and the ROOT
/// `Unit` struct, which shadows the runtime's `Unit` in the `Kit` namespace
/// too, makes that namespace qualify it.
///
/// A reference out of a namespaced module, to a sibling namespace
/// (`Example.B.Status` from `Example.A`) or to a ROOT type (`Example.Shared`
/// from `Example.Kv`), is rooted at the root package rather than at the
/// referring module, with an undotted root package and with a dotted one.
#[test]
fn test_that_csharp_code_with_types_from_other_namespaces_compiles() {
    for registry in [
        common::across_namespaces::get_registry(),
        common::across_namespaces::get_sibling_registry(),
        common::across_namespaces::to_root::get_registry(),
        common::across_namespaces::inherited::get_registry(),
    ] {
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

    for registry in [
        common::across_namespaces::get_sibling_registry(),
        common::across_namespaces::to_root::get_registry(),
    ] {
        let dir = tempdir().unwrap();
        csharp::Installer::new("Company.Models", &dir)
            .plugin(BincodePlugin)
            .generate(&registry)
            .unwrap();
        dotnet_build(&dir);
    }
}
