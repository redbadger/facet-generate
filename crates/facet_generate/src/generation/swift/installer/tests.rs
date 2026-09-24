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
//! - Plugin-declared type references, which import and depend on their
//!   module's target like the registry's own, cycle check included.

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
    reflection::format::QualifiedTypeName,
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

#[derive(Facet)]
#[facet(fg::namespace)]
struct Shared {
    id: u32,
}

/// A namespaced module that references a ROOT type depends on the root
/// package's target, which in turn no longer aggregates it.
#[test]
fn namespace_referencing_root_depends_on_the_root_target() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Entry {
        shared: Shared,
    }

    // `app` reaches the root target only through `kv`.
    #[derive(Facet)]
    #[facet(fg::namespace = "app")]
    struct View {
        entry: Entry,
    }

    let registry = reflect!(View).unwrap();

    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("Example", install_dir.path())
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let manifest = std::fs::read_to_string(install_dir.path().join("Package.swift")).unwrap();
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "Example",
        products: [
            .library(
                name: "Example",
                targets: ["App"]
            )
        ],
        targets: [
            .target(
                name: "App",
                dependencies: ["Kv", "Serde"]
            ),
            .target(
                name: "Example",
                dependencies: ["Serde"]
            ),
            .target(
                name: "Kv",
                dependencies: ["Example", "Serde"]
            ),
            .target(
                name: "Serde",
                dependencies: []
            ),
        ]
    )
    "#);

    let kv = std::fs::read_to_string(install_dir.path().join("Sources/Kv/Kv.swift")).unwrap();
    assert!(kv.starts_with("import Example\nimport Serde\n"), "{kv}");
    assert!(kv.contains("public var shared: Example.Shared\n"), "{kv}");
}

/// ROOT and `kv` referencing each other would make their targets depend on
/// each other, which `SwiftPM` rejects, so no manifest is written.
#[test]
fn root_and_namespace_referencing_each_other_is_rejected() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Entry {
        shared: Shared,
    }

    #[derive(Facet)]
    struct App {
        entry: Entry,
    }

    let registry = reflect!(App).unwrap();

    let install_dir = tempfile::tempdir().unwrap();
    let error = Installer::new("Example", install_dir.path())
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "Swift targets cannot depend on each other in a cycle, and these would: \
         `Example` references `Entry` in `Kv`; `Kv` references `Shared` in `Example`. \
         Move the types that one of these targets references into a namespace of their \
         own (`#[facet(fg::namespace = \"…\")]`), which the targets can both depend on"
    );
    assert!(!install_dir.path().join("Package.swift").exists());
}

/// The same for two named namespaces.
#[test]
fn namespaces_referencing_each_other_are_rejected() {
    #[derive(Facet)]
    #[facet(fg::namespace = "b")]
    struct Leaf {
        id: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "a")]
    struct Up {
        down: Down,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "b")]
    struct Down {
        leaf: Leaf,
        up: Option<Box<Up>>,
    }

    let registry = reflect!(Up).unwrap();

    let error = Installer::new("Example", tempfile::tempdir().unwrap().path())
        .generate(&registry)
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("`A` references `Down` in `B`; `B` references `Up` in `A`."),
        "{error}"
    );
}

/// A plugin whose output, in the module `module`, names `types`.
#[derive(Debug)]
struct ReferencesPlugin {
    module: &'static str,
    types: Vec<QualifiedTypeName>,
}

impl EmitterPlugin<Swift> for ReferencesPlugin {
    fn referenced_types(&self, config: &CodeGeneratorConfig) -> Vec<QualifiedTypeName> {
        if config.module_name() == self.module {
            self.types.clone()
        } else {
            vec![]
        }
    }
}

fn kit_presence() -> QualifiedTypeName {
    QualifiedTypeName::namespaced("kit".to_string(), "Presence".to_string())
}

#[derive(Facet)]
#[facet(fg::namespace = "kit")]
struct Presence {
    online: bool,
}

/// A namespace that only a plugin's output names is imported, and its target
/// is a dependency of the module's.
#[test]
fn a_plugin_s_reference_imports_and_depends_on_its_namespace() {
    #[derive(Facet)]
    struct App {
        id: u32,
    }

    let registry = reflect!(App, Presence).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("Example", install_dir.path())
        .plugin(ReferencesPlugin {
            module: "Example",
            types: vec![kit_presence()],
        })
        .generate(&registry)
        .unwrap();

    let manifest = std::fs::read_to_string(install_dir.path().join("Package.swift")).unwrap();
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "Example",
        products: [
            .library(
                name: "Example",
                targets: ["Example"]
            )
        ],
        targets: [
            .target(
                name: "Example",
                dependencies: ["Kit", "Serde"]
            ),
            .target(
                name: "Kit",
                dependencies: ["Serde"]
            ),
        ]
    )
    "#);

    let root =
        std::fs::read_to_string(install_dir.path().join("Sources/Example/Example.swift")).unwrap();
    assert_eq!(root.matches("import Kit\n").count(), 1, "{root}");
}

/// A namespace that both the registry and a plugin reference is imported, and
/// depended on, once.
#[test]
fn a_plugin_s_reference_to_a_namespace_the_registry_references_is_not_repeated() {
    #[derive(Facet)]
    struct App {
        presence: Presence,
    }

    let registry = reflect!(App).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("Example", install_dir.path())
        .plugin(ReferencesPlugin {
            module: "Example",
            types: vec![kit_presence(), kit_presence()],
        })
        .generate(&registry)
        .unwrap();

    let manifest = std::fs::read_to_string(install_dir.path().join("Package.swift")).unwrap();
    assert!(
        manifest.contains(concat!(
            r#"name: "Example","#,
            "\n",
            "            ",
            r#"dependencies: ["Kit", "Serde"]"#
        )),
        "{manifest}"
    );

    let root =
        std::fs::read_to_string(install_dir.path().join("Sources/Example/Example.swift")).unwrap();
    assert_eq!(root.matches("import Kit\n").count(), 1, "{root}");
}

/// A plugin's reference takes part in the cycle check: `kit` references a
/// ROOT type, so the root module naming a `kit` type would close a cycle.
#[test]
fn a_plugin_s_reference_that_closes_a_cycle_is_rejected() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    struct Row {
        shared: Shared,
    }

    let registry = reflect!(Row).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    let error = Installer::new("Example", install_dir.path())
        .plugin(ReferencesPlugin {
            module: "Example",
            types: vec![QualifiedTypeName::namespaced(
                "kit".to_string(),
                "Row".to_string(),
            )],
        })
        .generate(&registry)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "Swift targets cannot depend on each other in a cycle, and these would: \
         `Example` references `Row` in `Kit`; `Kit` references `Shared` in `Example`. \
         Move the types that one of these targets references into a namespace of their \
         own (`#[facet(fg::namespace = \"…\")]`), which the targets can both depend on"
    );
    assert!(!install_dir.path().join("Package.swift").exists());
}

/// A ROOT type a plugin names in a namespaced module makes it import, and
/// depend on, the root package's target.
#[test]
fn a_plugin_s_reference_to_a_root_type_depends_on_the_root_target() {
    let registry = reflect!(Shared, Presence).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("Example", install_dir.path())
        .plugin(ReferencesPlugin {
            module: "kit",
            types: vec![QualifiedTypeName::root("Shared".to_string())],
        })
        .generate(&registry)
        .unwrap();

    let manifest = std::fs::read_to_string(install_dir.path().join("Package.swift")).unwrap();
    assert!(
        manifest.contains(concat!(
            r#"name: "Kit","#,
            "\n",
            "            ",
            r#"dependencies: ["Example", "Serde"]"#
        )),
        "{manifest}"
    );

    let kit = std::fs::read_to_string(install_dir.path().join("Sources/Kit/Kit.swift")).unwrap();
    assert!(kit.starts_with("import Example\n"), "{kit}");
}

/// A plugin naming a type the registry does not have is a bug in the plugin.
#[test]
fn a_plugin_s_reference_to_an_unregistered_type_is_rejected() {
    let registry = reflect!(Shared).unwrap();
    let error = Installer::new("Example", tempfile::tempdir().unwrap().path())
        .plugin(ReferencesPlugin {
            module: "Example",
            types: vec![kit_presence()],
        })
        .generate(&registry)
        .unwrap_err()
        .to_string();

    assert!(
        error.ends_with(
            "declares that module `Example` references `kit::Presence`, which is not a \
             type in the registry"
        ),
        "{error}"
    );
}

#[derive(Facet)]
#[facet(fg::namespace = "b")]
struct Inner {
    x: u32,
}

#[derive(Facet)]
#[facet(fg::namespace = "a")]
struct Outer {
    inner: Inner,
}

/// With every type in a named namespace the root module has no types, so the
/// package declares no target of its own, and the library product lists the
/// top-level namespace target instead (#158).
#[test]
fn manifest_with_no_root_types() {
    let registry = reflect!(Outer).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("Example", install_dir.path())
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let manifest = std::fs::read_to_string(install_dir.path().join("Package.swift")).unwrap();
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "Example",
        products: [
            .library(
                name: "Example",
                targets: ["A"]
            )
        ],
        targets: [
            .target(
                name: "A",
                dependencies: ["B", "Serde"]
            ),
            .target(
                name: "B",
                dependencies: ["Serde"]
            ),
            .target(
                name: "Serde",
                dependencies: []
            ),
        ]
    )
    "#);
    assert!(!install_dir.path().join("Sources/Example").exists());
}

/// A plugin's target dependencies go to the namespace targets it was asked
/// about, and none to a root target the package does not declare (#158).
#[test]
fn a_plugin_s_target_dependencies_with_no_root_types() {
    let registry = reflect!(Outer).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("Example", install_dir.path())
        .plugin(BincodePlugin)
        .plugin(FfiPlugin)
        .generate(&registry)
        .unwrap();

    let manifest = std::fs::read_to_string(install_dir.path().join("Package.swift")).unwrap();
    insta::assert_snapshot!(manifest, @r#"
    // swift-tools-version: 5.8
    import PackageDescription

    let package = Package(
        name: "Example",
        products: [
            .library(
                name: "Example",
                targets: ["A"]
            )
        ],
        dependencies: [
            .package(
                path: "../Shared"
            )
        ],
        targets: [
            .target(
                name: "A",
                dependencies: ["B", "Serde", .product(name: "Shared", package: "Shared")]
            ),
            .target(
                name: "B",
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

/// A plugin with no output of its own: with it, the generated types declare
/// their `Hashable` and `Equatable` conformance, and nothing else.
#[derive(Debug)]
struct DeclaresConformance;

impl EmitterPlugin<Swift> for DeclaresConformance {}

/// Generates `registry` as the package `Example` with
/// [`DeclaresConformance`] and the given external packages, returning the
/// line declaring each type, keyed by module.
fn declarations(
    registry: &crate::Registry,
    external_packages: &[ExternalPackage],
) -> std::collections::BTreeMap<String, Vec<String>> {
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("Example", install_dir.path())
        .external_packages(external_packages)
        .plugin(DeclaresConformance)
        .generate(registry)
        .unwrap();

    let mut declarations = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(install_dir.path().join("Sources")).unwrap() {
        let module = entry.unwrap().file_name().into_string().unwrap();
        let source = std::fs::read_to_string(
            install_dir
                .path()
                .join("Sources")
                .join(&module)
                .join(format!("{module}.swift")),
        )
        .unwrap();
        declarations.insert(
            module,
            source
                .lines()
                .filter(|line| line.starts_with("public struct") || line.contains("public enum"))
                .map(str::to_string)
                .collect(),
        );
    }
    declarations
}

/// A type holding a type from another module that is `Equatable` but not
/// `Hashable` (a native tuple field) is declared `Equatable` only (#156).
#[test]
fn a_type_holding_a_non_hashable_type_from_another_module_is_not_hashable() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    struct Holder {
        t: (u32, u32),
    }

    #[derive(Facet)]
    struct SwHash {
        h: Holder,
    }

    let registry = reflect!(SwHash).unwrap();

    insta::assert_debug_snapshot!(declarations(&registry, &[]), @r#"
    {
        "Example": [
            "public struct SwHash: Equatable {",
        ],
        "Kit": [
            "public struct Holder: Equatable {",
        ],
    }
    "#);
}

/// A type holding a type from another module that is neither `Equatable` nor
/// `Hashable` (a `Void` field) is declared neither (#156).
#[test]
fn a_type_holding_a_non_equatable_type_from_another_module_is_neither() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    struct Holder {
        u: (),
    }

    #[derive(Facet)]
    struct SwEq {
        h: Holder,
    }

    let registry = reflect!(SwEq).unwrap();

    insta::assert_debug_snapshot!(declarations(&registry, &[]), @r#"
    {
        "Example": [
            "public struct SwEq {",
        ],
        "Kit": [
            "public struct Holder {",
        ],
    }
    "#);
}

/// Non-conformance propagates across two module boundaries, root → kit →
/// other, through generic containers (#156).
#[test]
fn non_conformance_propagates_across_two_modules() {
    #[derive(Facet)]
    #[facet(fg::namespace = "other")]
    struct Pair {
        pair: (u32, u32),
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "other")]
    struct Nothing {
        id: u32,
        unit: (),
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    struct Pairs {
        pairs: Vec<Pair>,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    struct Nothings {
        nothings: std::collections::BTreeMap<String, Nothing>,
    }

    #[derive(Facet)]
    struct Top {
        pairs: Option<Pairs>,
        nothings: Box<Nothings>,
    }

    #[derive(Facet)]
    struct TopPairs {
        pairs: Vec<Option<Pairs>>,
    }

    let registry = reflect!(Top, TopPairs).unwrap();

    insta::assert_debug_snapshot!(declarations(&registry, &[]), @r#"
    {
        "Example": [
            "public struct Top {",
            "public struct TopPairs: Equatable {",
        ],
        "Kit": [
            "public struct Nothings {",
            "public struct Pairs: Equatable {",
        ],
        "Other": [
            "public struct Nothing {",
            "public struct Pair: Equatable {",
        ],
    }
    "#);
}

/// A type in a cycle is not `Hashable` when another type in the cycle isn't,
/// even when the type looked at first is the one that isn't, and a type in
/// another module holding it is not either (#156). `Ping` comes first; it
/// holds a native tuple and a `Pong`, which holds only a `Ping`.
#[test]
fn a_type_in_a_non_hashable_cycle_is_not_hashable_in_any_module() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Ping {
        Pong(Box<Pong>),
        Pair((u32, u32)),
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kit")]
    #[repr(C)]
    #[allow(dead_code)]
    enum Pong {
        Done,
        Ping(Box<Ping>),
    }

    #[derive(Facet)]
    struct HoldsPong {
        pong: Pong,
    }

    let registry = reflect!(HoldsPong).unwrap();

    insta::assert_debug_snapshot!(declarations(&registry, &[]), @r#"
    {
        "Example": [
            "public struct HoldsPong: Equatable {",
        ],
        "Kit": [
            "indirect public enum Ping: Equatable {",
            "indirect public enum Pong: Equatable {",
        ],
    }
    "#);
}

/// A type from an external package is generated elsewhere, so a type holding
/// it assumes it conforms to both protocols, whatever its fields.
#[test]
fn a_type_from_an_external_package_is_assumed_to_conform() {
    #[derive(Facet)]
    #[facet(fg::namespace = "api")]
    struct Holder {
        t: (u32, u32),
    }

    #[derive(Facet)]
    struct SwHash {
        h: Holder,
    }

    let registry = reflect!(SwHash).unwrap();

    insta::assert_debug_snapshot!(
        declarations(
            &registry,
            &[ExternalPackage {
                for_namespace: "api".to_string(),
                location: PackageLocation::Path("../Api".to_string()),
                module_name: None,
                version: None,
            }]
        ),
        @r#"
    {
        "Example": [
            "public struct SwHash: Hashable, Equatable {",
        ],
    }
    "#
    );
}
