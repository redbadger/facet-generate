//! Snapshot tests for the Kotlin [`Installer`] — **project scaffolding**.
//!
//! These tests verify the `build.gradle.kts` manifest that the installer
//! generates, without writing anything to disk. They cover:
//!
//! - Basic manifest structure: Kotlin JVM and serialization plugins, `group`
//!   metadata.
//! - External URL dependencies: Maven-style `implementation` entries with
//!   version strings, plus the `kotlinx-serialization-json` runtime.
//! - External path dependencies: local file-system dependencies via
//!   `files("…")`.

use crate::generation::{Encoding, ExternalPackage, PackageLocation, kotlin::Installer};

#[test]
fn test_new_installer() {
    let _installer = Installer::new("test-package", "/tmp");
    // Basic smoke test - just ensure we can create an installer without panicking
}

#[test]
fn test_make_manifest_basic() {
    let installer = Installer::new("test-package", "/tmp");
    let manifest = installer.make_manifest("test-package");

    // Check that the manifest contains expected Kotlin/Gradle content
    assert!(manifest.contains(r#"kotlin("jvm")"#));
    assert!(manifest.contains(r#"kotlin("plugin.serialization")"#));
    assert!(manifest.contains(r#"group = "test-package""#));
}

#[test]
fn test_make_manifest_with_external_packages() {
    let external_packages = vec![ExternalPackage {
        for_namespace: "external.package".to_string(),
        module_name: Some("external.package".to_string()),
        location: PackageLocation::Url("https://example.com/external-lib".to_string()),
        version: Some("2.0.0".to_string()),
    }];

    let installer = Installer::new("test-package", "/tmp")
        .encoding(Encoding::Json)
        .external_packages(&external_packages);
    let manifest = installer.make_manifest("test-package");

    // Check that external dependencies are included
    assert!(manifest.contains("external-lib:2.0.0"));
    assert!(manifest.contains("kotlinx-serialization-json"));
}

#[test]
fn test_make_manifest_with_path_dependency() {
    let external_packages = vec![ExternalPackage {
        for_namespace: "local.package".to_string(),
        module_name: Some("local.package".to_string()),
        location: PackageLocation::Path("../local-lib".to_string()),
        version: None,
    }];

    let installer = Installer::new("test-package", "/tmp").external_packages(&external_packages);
    let manifest = installer.make_manifest("test-package");

    // Check that path dependencies are handled correctly
    assert!(manifest.contains(r#"files("../local-lib")"#));
}
