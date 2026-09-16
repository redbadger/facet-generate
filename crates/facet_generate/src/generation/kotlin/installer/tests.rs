//! Snapshot tests for the Kotlin [`Installer`] — **project scaffolding**.
//!
//! These tests verify the `build.gradle.kts` manifest that the installer
//! generates, and the companion files it writes. They cover:
//!
//! - Basic manifest structure: Kotlin JVM and serialization plugins, `group`
//!   metadata.
//! - External URL dependencies: Maven-style `implementation` entries with
//!   version strings, plus the `kotlinx-serialization-json` runtime.
//! - External path dependencies: local file-system dependencies via
//!   `files("…")`.
//! - Plugin companion files written into the module's package directory.

use facet::Facet;
use indoc::indoc;

use crate::{
    generation::{
        CodeGeneratorConfig, ExternalPackage, PackageLocation,
        bincode::BincodePlugin,
        json::JsonPlugin,
        kotlin::{Installer, Kotlin},
        plugin::{CompanionFile, EmitterPlugin},
    },
    reflect,
};

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
        .plugin(JsonPlugin)
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

/// A plugin standing in for one that bridges to an FFI package: it contributes
/// a companion source file that needs an import of its own.
#[derive(Debug)]
struct FfiPlugin;

impl EmitterPlugin<Kotlin> for FfiPlugin {
    fn companion_files(&self, _config: &CodeGeneratorConfig) -> Vec<CompanionFile> {
        vec![CompanionFile {
            file_name: "FfiBridge.kt".to_string(),
            imports: vec!["import com.example.shared.CoreFfi".to_string()],
            contents: indoc! {r"
            class FfiBridge(private val ffi: CoreFfi = CoreFfi())"}
            .to_string(),
        }]
    }
}

#[test]
fn companion_file_is_written_in_the_package_directory() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
    }

    let registry = reflect!(MyStruct).unwrap();

    let install_dir = tempfile::tempdir().unwrap();

    Installer::new("com.example", install_dir.path())
        .plugin(BincodePlugin)
        .plugin(FfiPlugin)
        .generate(&registry)
        .unwrap();

    let companion =
        std::fs::read_to_string(install_dir.path().join("com/example/FfiBridge.kt")).unwrap();

    // The module's package and imports, merged with the companion's, and none
    // of the module helpers.
    insta::assert_snapshot!(companion, @r"
    package com.example

    import com.example.shared.CoreFfi
    import com.novi.bincode.BincodeDeserializer
    import com.novi.bincode.BincodeSerializer
    import com.novi.serde.DeserializationError
    import com.novi.serde.Deserializer
    import com.novi.serde.Serializer

    class FfiBridge(private val ffi: CoreFfi = CoreFfi())
    ");
}
