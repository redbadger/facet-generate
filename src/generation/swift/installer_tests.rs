use facet::Facet;

use crate::{
    generation::{
        Encoding, ExternalPackage, PackageLocation, SourceInstaller as _, module::split,
        swift::installer::Installer,
    },
    reflect,
};

#[test]
fn simple_manifest() {
    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(package_name, install_dir.path(), &[]);

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

    let registry = reflect!(MyStruct);

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path(), &[]);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone().with_encoding(Encoding::Bincode);
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

    let registry = reflect!(MyStruct);

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(
        package_name,
        install_dir.path(),
        &[ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Url("https://github.com/serde-rs/serde".to_string()),
            module_name: None,
            version: Some("1.0.137".to_string()),
        }],
    );

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone().with_encoding(Encoding::Bincode);
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

    let registry = reflect!(MyStruct);

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(
        package_name,
        install_dir.path(),
        &[ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Path("../Serde".to_string()),
            module_name: None,
            version: None,
        }],
    );

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone().with_encoding(Encoding::Bincode);
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
    #[facet(namespace = "another_target")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = reflect!(Root);

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    let mut installer = Installer::new(package_name, install_dir.path(), &[]);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone().with_encoding(Encoding::Bincode);
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
    #[facet(namespace = "another_namespace")]
    struct Another {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        id: u32,
    }

    let registry = reflect!(Root, Another);

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    let mut installer = Installer::new(package_name, install_dir.path(), &[]);

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone().with_encoding(Encoding::Bincode);
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

    let installer = Installer::new(
        package_name,
        install_dir.path(),
        &[ExternalPackage {
            for_namespace: "AnotherPackage".to_string(),
            location: PackageLocation::Url(
                "https://github.com/example/another_package".to_string(),
            ),
            module_name: None,
            version: Some("1.0".to_string()),
        }],
    );

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
    #[facet(namespace = "another_package")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = reflect!(Root);

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(
        package_name,
        install_dir.path(),
        &[ExternalPackage {
            for_namespace: "another_package".to_string(),
            location: PackageLocation::Url(
                "https://github.com/example/another_package".to_string(),
            ),
            module_name: None,
            version: Some("1.0".to_string()),
        }],
    );

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone().with_encoding(Encoding::Bincode);
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
    #[facet(namespace = "another_namespace")]
    struct Another {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        id: u32,
    }

    let registry = reflect!(Root, Another);

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(
        package_name,
        install_dir.path(),
        &[ExternalPackage {
            for_namespace: "another_namespace".to_string(),
            location: PackageLocation::Url(
                "https://github.com/example/another_package".to_string(),
            ),
            module_name: None,
            version: Some("1.0".to_string()),
        }],
    );

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone().with_encoding(Encoding::Bincode);
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
    #[facet(namespace = "api")]
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

    let registry = reflect!(Parent);

    let package_name = "App";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(
        package_name,
        install_dir.path(),
        &[ExternalPackage {
            for_namespace: "api".to_string(),
            location: PackageLocation::Path("../Api".to_string()),
            module_name: None,
            version: None,
        }],
    );

    for (module, registry) in split(package_name, &registry) {
        let config = module.config().clone().with_encoding(Encoding::Bincode);

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
