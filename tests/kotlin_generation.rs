#![cfg(feature = "kotlin")]

use std::process::Command;

use facet_generate::{generation::kotlin, reflect};
use tempfile::tempdir;

pub mod common;

fn gradle_command() -> Command {
    Command::new("gradle")
}

#[test]
fn test_that_kotlin_code_compiles() {
    type Test = common::PrimitiveTypes;

    let registry = reflect!(Test).unwrap();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    let package_name = "com.example.testing";

    kotlin::Installer::new(package_name, &dir)
        .generate(&registry)
        .unwrap();

    let args = ["--configuration-cache"];

    let status = gradle_command()
        .args(args)
        .arg("--version")
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success());

    let status = gradle_command()
        .args(args)
        .arg("build")
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success());
}
