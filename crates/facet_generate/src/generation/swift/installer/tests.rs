//! Snapshot tests for the Swift [`Installer`] — **project scaffolding**.
//!
//! These tests verify the `Package.swift` manifest that the installer
//! generates, and the companion files it writes. They cover:
//!
//! - Basic manifest structure: SPM targets, library products.
//! - External URL dependencies: remote package references with version
//!   strings.
//! - External path dependencies: local file-system dependencies via
//!   `.package(path: "…")`.
//! - Serde runtime target registration and dependency edges.
//! - Multi-module (namespace) scenarios where each namespace becomes a
//!   separate SPM target.
//! - Plugin-provided package and target dependencies, and deployment
//!   platforms.
//! - Plugin companion files written beside the generated module.

use facet::Facet;
use indoc::indoc;

use crate as fg;
use crate::{
    generation::{
        CodeGeneratorConfig, ExternalPackage, PackageLocation, SourceInstaller as _,
        bincode::BincodePlugin,
        module::split,
        plugin::{CompanionFile, EmitterPlugin},
        swift::{Swift, installer::Installer},
    },
    reflect,
};

/// A plugin standing in for one that bridges to an FFI package: it adds a
/// package dependency, a target dependency, and a companion source file.
#[derive(Debug)]
struct FfiPlugin;

impl EmitterPlugin<Swift> for FfiPlugin {
    fn manifest_dependencies(&self) -> Vec<String> {
        vec![
            indoc! {r#"
            .package(
                path: "../Shared"
            )"#}
            .to_string(),
        ]
    }

    fn target_dependencies(&self, _config: &CodeGeneratorConfig) -> Vec<String> {
        vec![r#".product(name: "Shared", package: "Shared")"#.to_string()]
    }

    fn companion_files(&self, _config: &CodeGeneratorConfig) -> Vec<CompanionFile> {
        vec![CompanionFile {
            file_name: "FfiBridge.swift".to_string(),
            imports: vec!["Shared".to_string()],
            contents: indoc! {r"
                public struct FfiBridge {
                    public init() {}
                }"}
            .to_string(),
        }]
    }
}

#[test]
fn simple_manifest() {
    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(package_name, install_dir.path());

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "MyPackage",
        products: [
            .library(
                name: "MyPackage",
                targets: ["MyPackage"]
            )
        ],
        targets: [
            .target(
                name: "MyPackage",
                dependencies: []
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_serde_as_target() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
        name: String,
    }

    let registry = reflect!(MyStruct).unwrap();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path()).plugin(BincodePlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    installer.install_serde_runtime().unwrap();

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "MyPackage",
        products: [
            .library(
                name: "MyPackage",
                targets: ["MyPackage"]
            )
        ],
        targets: [
            .target(
                name: "MyPackage",
                dependencies: ["Serde"]
            ),
            .target(
                name: "Serde",
                dependencies: []
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_serde_as_a_remote_dependency() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
        name: String,
    }

    let registry = reflect!(MyStruct).unwrap();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path())
        .external_packages(&[ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Url("https://github.com/serde-rs/serde".to_string()),
            module_name: None,
            version: Some("1.0.137".to_string()),
        }])
        .plugin(BincodePlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "MyPackage",
        products: [
            .library(
                name: "MyPackage",
                targets: ["MyPackage"]
            )
        ],
        dependencies: [
            .package(
                url: "https://github.com/serde-rs/serde",
                from: "1.0.137"
            )
        ],
        targets: [
            .target(
                name: "MyPackage",
                dependencies: ["Serde"]
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_serde_as_a_local_dependency() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
        name: String,
    }

    let registry = reflect!(MyStruct).unwrap();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path())
        .external_packages(&[ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Path("../Serde".to_string()),
            module_name: None,
            version: None,
        }])
        .plugin(BincodePlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "MyPackage",
        products: [
            .library(
                name: "MyPackage",
                targets: ["MyPackage"]
            )
        ],
        dependencies: [
            .package(
                path: "../Serde"
            )
        ],
        targets: [
            .target(
                name: "MyPackage",
                dependencies: ["Serde"]
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_namespaces() {
    #[derive(Facet)]
    #[facet(fg::namespace = "another_target")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = reflect!(Root).unwrap();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    let mut installer = Installer::new(package_name, install_dir.path()).plugin(BincodePlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "MyPackage",
        products: [
            .library(
                name: "MyPackage",
                targets: ["MyPackage"]
            )
        ],
        targets: [
            .target(
                name: "AnotherTarget",
                dependencies: ["Serde"]
            ),
            .target(
                name: "MyPackage",
                dependencies: ["AnotherTarget", "Serde"]
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_disjoint_namespaces() {
    #[derive(Facet)]
    #[facet(fg::namespace = "another_namespace")]
    struct Another {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        id: u32,
    }

    let registry = reflect!(Root, Another).unwrap();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    let mut installer = Installer::new(package_name, install_dir.path()).plugin(BincodePlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "MyPackage",
        products: [
            .library(
                name: "MyPackage",
                targets: ["AnotherNamespace", "MyPackage"]
            )
        ],
        targets: [
            .target(
                name: "AnotherNamespace",
                dependencies: ["Serde"]
            ),
            .target(
                name: "MyPackage",
                dependencies: ["Serde"]
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_remote_dependencies() {
    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let installer =
        Installer::new(package_name, install_dir.path()).external_packages(&[ExternalPackage {
            for_namespace: "AnotherPackage".to_string(),
            location: PackageLocation::Url(
                "https://github.com/example/another_package".to_string(),
            ),
            module_name: None,
            version: Some("1.0".to_string()),
        }]);

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "MyPackage",
        products: [
            .library(
                name: "MyPackage",
                targets: ["MyPackage"]
            )
        ],
        dependencies: [
            .package(
                url: "https://github.com/example/another_package",
                from: "1.0"
            )
        ],
        targets: [
            .target(
                name: "MyPackage",
                dependencies: []
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_namespaces_and_dependencies() {
    #[derive(Facet)]
    #[facet(fg::namespace = "another_package")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = reflect!(Root).unwrap();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path())
        .external_packages(&[ExternalPackage {
            for_namespace: "another_package".to_string(),
            location: PackageLocation::Url(
                "https://github.com/example/another_package".to_string(),
            ),
            module_name: None,
            version: Some("1.0".to_string()),
        }])
        .plugin(BincodePlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "MyPackage",
        products: [
            .library(
                name: "MyPackage",
                targets: ["MyPackage"]
            )
        ],
        dependencies: [
            .package(
                url: "https://github.com/example/another_package",
                from: "1.0"
            )
        ],
        targets: [
            .target(
                name: "MyPackage",
                dependencies: ["AnotherPackage", "Serde"]
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_disjoint_namespaces_and_dependencies() {
    #[derive(Facet)]
    #[facet(fg::namespace = "another_namespace")]
    struct Another {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        id: u32,
    }

    let registry = reflect!(Root, Another).unwrap();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path())
        .external_packages(&[ExternalPackage {
            for_namespace: "another_namespace".to_string(),
            location: PackageLocation::Url(
                "https://github.com/example/another_package".to_string(),
            ),
            module_name: None,
            version: Some("1.0".to_string()),
        }])
        .plugin(BincodePlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "MyPackage",
        products: [
            .library(
                name: "MyPackage",
                targets: ["MyPackage"]
            )
        ],
        dependencies: [
            .package(
                url: "https://github.com/example/another_package",
                from: "1.0"
            )
        ],
        targets: [
            .target(
                name: "MyPackage",
                dependencies: ["Serde"]
            ),
        ]
    )
    "#);
}

#[test]
fn external_dependencies_collected_across_multiple_types_in_same_namespace() {
    // This test ensures that when multiple types belong to the same namespace,
    // ALL external dependencies from ALL types are collected properly.
    // Previously, only the first type's external dependencies were preserved.
    #[derive(Facet)]
    #[facet(fg::namespace = "api")]
    struct GrandChild {
        test: String,
    }

    #[derive(Facet)]
    struct Child {
        api: GrandChild,
    }

    #[derive(Facet)]
    struct Parent {
        event: Child,
    }

    let registry = reflect!(Parent).unwrap();

    let package_name = "App";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path())
        .external_packages(&[ExternalPackage {
            for_namespace: "api".to_string(),
            location: PackageLocation::Path("../Api".to_string()),
            module_name: None,
            version: None,
        }])
        .plugin(BincodePlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "App",
        products: [
            .library(
                name: "App",
                targets: ["App"]
            )
        ],
        dependencies: [
            .package(
                path: "../Api"
            )
        ],
        targets: [
            .target(
                name: "App",
                dependencies: ["Api", "Serde"]
            ),
        ]
    )
    "#);
}

#[test]
fn external_dependency_references_local_dependency() {
    #[derive(Facet)]
    #[facet(fg::namespace = "local_dependency")]
    struct GrandChild {
        inner: GreatGrandChild,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "external_dependency")]
    struct GreatGrandChild {
        inner: String,
    }

    #[derive(Facet)]
    struct Child {
        inner: GrandChild,
    }

    #[derive(Facet)]
    struct Parent {
        inner: Child,
    }

    let registry = reflect!(Parent).unwrap();

    let package_name = "App";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path())
        .external_packages(&[ExternalPackage {
            for_namespace: "external_dependency".to_string(),
            location: PackageLocation::Path("../ExternalDependency".to_string()),
            module_name: None,
            version: None,
        }])
        .plugin(BincodePlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "App",
        products: [
            .library(
                name: "App",
                targets: ["App"]
            )
        ],
        dependencies: [
            .package(
                path: "../ExternalDependency"
            )
        ],
        targets: [
            .target(
                name: "App",
                dependencies: ["ExternalDependency", "LocalDependency", "Serde"]
            ),
            .target(
                name: "LocalDependency",
                dependencies: ["ExternalDependency", "Serde"]
            ),
        ]
    )
    "#);
}

/// The same, for a plugin whose target edge belongs to one module: the
/// package's own, not the module a namespace was generated into.
#[derive(Debug)]
struct AppOnlyFfiPlugin {
    package: &'static str,
}

impl EmitterPlugin<Swift> for AppOnlyFfiPlugin {
    fn manifest_dependencies(&self) -> Vec<String> {
        vec![
            indoc! {r#"
            .package(
                path: "../Shared"
            )"#}
            .to_string(),
        ]
    }

    fn target_dependencies(&self, config: &CodeGeneratorConfig) -> Vec<String> {
        if config.module_name() != self.package {
            return vec![];
        }
        vec![r#".product(name: "Shared", package: "Shared")"#.to_string()]
    }
}

#[test]
fn manifest_with_plugin_dependencies() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
    }

    let registry = reflect!(MyStruct).unwrap();

    let package_name = "App";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path())
        .plugin(BincodePlugin)
        .plugin(FfiPlugin);

    installer.install_serde_runtime().unwrap();

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    // The plugin's package dependency is listed, and its target edge is on the
    // generated module's target only — never on `Serde`, and never quoted.
    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "App",
        products: [
            .library(
                name: "App",
                targets: ["App"]
            )
        ],
        dependencies: [
            .package(
                path: "../Shared"
            )
        ],
        targets: [
            .target(
                name: "App",
                dependencies: ["Serde", .product(name: "Shared", package: "Shared")]
            ),
            .target(
                name: "Serde",
                dependencies: []
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_plugin_and_external_dependencies() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
    }

    let registry = reflect!(MyStruct).unwrap();

    let package_name = "App";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path())
        .external_packages(&[ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Path("../Serde".to_string()),
            module_name: None,
            version: None,
        }])
        .plugin(BincodePlugin)
        .plugin(FfiPlugin);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "App",
        products: [
            .library(
                name: "App",
                targets: ["App"]
            )
        ],
        dependencies: [
            .package(
                path: "../Serde"
            ),
            .package(
                path: "../Shared"
            )
        ],
        targets: [
            .target(
                name: "App",
                dependencies: ["Serde", .product(name: "Shared", package: "Shared")]
            ),
        ]
    )
    "#);
}

#[test]
fn manifest_with_platforms() {
    let package_name = "App";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(package_name, install_dir.path())
        .platforms(&[".iOS(.v16)".to_string(), ".macOS(.v13)".to_string()]);

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "App",
        platforms: [.iOS(.v16), .macOS(.v13)],
        products: [
            .library(
                name: "App",
                targets: ["App"]
            )
        ],
        targets: [
            .target(
                name: "App",
                dependencies: []
            ),
        ]
    )
    "#);
}

#[test]
fn companion_file_is_written_beside_the_module() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
    }

    let registry = reflect!(MyStruct).unwrap();

    let install_dir = tempfile::tempdir().unwrap();

    Installer::new("App", install_dir.path())
        .plugin(BincodePlugin)
        .plugin(FfiPlugin)
        .generate(&registry)
        .unwrap();

    let companion =
        std::fs::read_to_string(install_dir.path().join("Sources/App/FfiBridge.swift")).unwrap();

    // The module's own imports, merged with the companion's, and none of the
    // module helpers.
    insta::assert_snapshot!(companion, @r"
    import Serde
    import Shared

    public struct FfiBridge {
        public init() {}
    }
    ");
}

#[test]
fn companion_file_imports_foundation_when_the_module_uses_uuid() {
    #[derive(Facet)]
    struct MyStruct {
        id: uuid::Uuid,
    }

    let registry = reflect!(MyStruct).unwrap();

    let install_dir = tempfile::tempdir().unwrap();

    Installer::new("App", install_dir.path())
        .plugin(BincodePlugin)
        .plugin(FfiPlugin)
        .generate(&registry)
        .unwrap();

    let companion =
        std::fs::read_to_string(install_dir.path().join("Sources/App/FfiBridge.swift")).unwrap();

    insta::assert_snapshot!(companion, @r"
    import Foundation
    import Serde
    import Shared

    public struct FfiBridge {
        public init() {}
    }
    ");
}

#[test]
fn companion_file_is_written_when_serde_is_external() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
    }

    let registry = reflect!(MyStruct).unwrap();

    let install_dir = tempfile::tempdir().unwrap();

    Installer::new("App", install_dir.path())
        .external_packages(&[ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Path("../Serde".to_string()),
            module_name: None,
            version: None,
        }])
        .plugin(BincodePlugin)
        .plugin(FfiPlugin)
        .generate(&registry)
        .unwrap();

    // Runtime files are skipped when serde comes from a package…
    assert!(!install_dir.path().join("Sources/Serde").exists());
    // …but the companion file belongs to the module, so it is still written.
    let companion =
        std::fs::read_to_string(install_dir.path().join("Sources/App/FfiBridge.swift")).unwrap();

    insta::assert_snapshot!(companion, @r"
    import Serde
    import Shared

    public struct FfiBridge {
        public init() {}
    }
    ");
}

/// Every module's plugins are asked for their target edges, because every
/// module is a target of its own — so an edge that belongs to one module has
/// to be scoped there, and the config is what makes that possible.
///
/// Without it a plugin bridging the package to an FFI would put that edge on
/// every feature target as well as the app's. Harmless to SPM, and wrong.
#[test]
fn a_plugin_can_scope_its_target_edge_to_one_module() {
    #[derive(Facet)]
    #[facet(fg::namespace = "feature")]
    struct Inner {
        id: u32,
    }

    #[derive(Facet)]
    struct MyStruct {
        inner: Inner,
    }

    let registry = reflect!(MyStruct).unwrap();

    let package_name = "App";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path())
        .plugin(BincodePlugin)
        .plugin(AppOnlyFfiPlugin {
            package: package_name,
        });

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone();
        installer.install_module(&config, &registry).unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "App",
        products: [
            .library(
                name: "App",
                targets: ["App"]
            )
        ],
        dependencies: [
            .package(
                path: "../Shared"
            )
        ],
        targets: [
            .target(
                name: "App",
                dependencies: ["Feature", "Serde", .product(name: "Shared", package: "Shared")]
            ),
            .target(
                name: "Feature",
                dependencies: ["Serde"]
            ),
        ]
    )
    "#);
}
