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

/// The root package `com.kv` ends in the name of namespace `kv`, which is
/// still its own package, `com.kv.kv`: the root module refers to it there, and
/// the `kv` module refers to its own types and to ROOT ones as before (#164).
#[test]
fn root_module_refers_to_a_namespace_named_like_the_package_leaf() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Entry {
        id: u32,
        tag: Tag,
        shared: Shared,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Tag {
        name: String,
    }

    #[derive(Facet)]
    #[facet(fg::namespace)]
    struct Shared {
        id: u32,
    }

    #[derive(Facet)]
    struct App {
        entry: Entry,
    }

    let registry = reflect!(App).unwrap();
    let install_dir = tempfile::tempdir().unwrap();

    Installer::new("com.kv", install_dir.path())
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let root = std::fs::read_to_string(install_dir.path().join("com/kv/Kv.kt")).unwrap();
    assert!(root.starts_with("package com.kv\n"), "{root}");
    assert!(root.contains("val entry: com.kv.kv.Entry,"), "{root}");
    assert!(
        root.contains("val entry = com.kv.kv.Entry.deserialize(deserializer)"),
        "{root}"
    );
    assert!(!root.contains("com.kv.Entry"), "{root}");

    let kv = std::fs::read_to_string(install_dir.path().join("com/kv/kv/Kv.kt")).unwrap();
    assert!(kv.starts_with("package com.kv.kv\n"), "{kv}");
    assert!(kv.contains("val tag: com.kv.kv.Tag,"), "{kv}");
    assert!(kv.contains("val shared: com.kv.Shared,"), "{kv}");
    assert!(!kv.contains("com.kv.kv.kv"), "{kv}");
}

/// The root module is named after the package, whose last segment `shared`
/// is spelled like an external namespace: it is still written, and the
/// external namespace's module is not (#186).
#[test]
fn root_module_is_written_when_the_package_ends_in_an_external_namespace() {
    use crate as fg;

    #[derive(Facet)]
    #[facet(fg::namespace = "shared")]
    struct Ext {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        e: Ext,
    }

    let registry = reflect!(App).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("com.acme.shared", install_dir.path())
        .plugin(BincodePlugin)
        .external_packages(&[ExternalPackage {
            for_namespace: "shared".to_string(),
            module_name: None,
            location: PackageLocation::Path("../shared".to_string()),
            version: None,
        }])
        .generate(&registry)
        .unwrap();

    let root =
        std::fs::read_to_string(install_dir.path().join("com/acme/shared/Shared.kt")).unwrap();
    assert!(root.contains("data class App("), "{root}");
    assert!(
        !install_dir
            .path()
            .join("com/acme/shared/shared/Shared.kt")
            .exists()
    );

    let manifest = std::fs::read_to_string(install_dir.path().join("build.gradle.kts")).unwrap();
    assert!(manifest.contains(r#"files("../shared")"#), "{manifest}");
}
