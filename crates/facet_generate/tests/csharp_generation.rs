#![cfg(feature = "csharp")]

use std::process::Command;

use facet::Facet;
use facet_generate as fg;
use facet_generate::{
    generation::{Encoding, csharp},
    reflect,
};
use serde::{Deserialize, Serialize};
use tempfile::{TempDir, tempdir};

pub mod common;

fn dotnet_build(dir: &TempDir) {
    // Each test gets its own TMPDIR so that parallel `dotnet` invocations
    // don't race on the shared-memory mutex directory (/tmp/.dotnet/shm/)
    // that the .NET NuGet migration runner creates on Linux.
    let private_tmp = dir.path().join(".tmp");
    std::fs::create_dir_all(&private_tmp).unwrap();

    let status = Command::new("dotnet")
        .arg("build")
        .current_dir(dir)
        .env("TMPDIR", &private_tmp)
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
        .encoding(Encoding::Bincode)
        .generate(&registry)
        .unwrap();

    dotnet_build(&dir);
}

#[test]
fn test_that_csharp_code_compiles_with_json() {
    let registry = common::get_registry();
    let dir = tempdir().unwrap();

    csharp::Installer::new("Example.Testing", &dir)
        .encoding(Encoding::Json)
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
        .encoding(Encoding::None)
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
