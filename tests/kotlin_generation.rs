#![cfg(feature = "kotlin")]

use std::process::Command;

use facet_generate::{
    generation::{SourceInstaller, kotlin, module},
    reflect,
};
use tempfile::tempdir;

pub mod common;

#[test]
// #[ignore = "This test is currently failing due to i128/u128 support"]
fn test_that_kotlin_code_compiles() {
    type Test = common::PrimitiveTypes;

    let registry = reflect!(Test);
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    let package_name = "com.example.testing";

    let mut installer = kotlin::Installer::new(package_name, &dir, &[]);
    for (module, registry) in &module::split(package_name, &registry) {
        installer.install_module(module.config(), registry).unwrap();
    }
    installer.install_manifest(package_name).unwrap();

    let args = ["--configuration-cache"];

    let status = Command::new("gradle")
        .args(args)
        .arg("--version")
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("gradle")
        .args(args)
        .arg("build")
        .current_dir(&dir)
        .status()
        .unwrap();
    assert!(status.success());
}
