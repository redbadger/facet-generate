use facet::Facet;

use crate::{
    generation::{SourceInstaller as _, module::split},
    reflection::RegistryBuilder,
};

use super::*;

#[test]
fn manifest_without_serde() {
    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    let installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        None,
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
fn manifest_with_serde_installed() {
    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    // No dependencies and no Serde runtime installed
    let mut installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        None,
    );

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
fn manifest_with_serde_as_dependency() {
    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    let deps = vec![(
        "serde".to_string(),
        Dependency {
            name: "Serde".to_string(),
            location: "https://github.com/serde-rs/serde".to_string(),
            version: Some("1.0.137".to_string()),
        },
    )];

    let installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        Some(deps),
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
                name: "Serde",
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
fn manifest_with_namespaces() {
    #[derive(Facet)]
    #[facet(namespace = "my_namespace")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = RegistryBuilder::new().add_type::<Root>().build();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    let mut installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        None,
    );

    for (module, registry) in split(package_name, &registry) {
        installer
            .install_module(module.config(), &registry)
            .unwrap();
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
                    name: "MyNamespace",
                    dependencies: []
                ),
                .target(
                    name: "MyPackage",
                    dependencies: ["MyNamespace"]
                ),
            ]
        )
        "#);
}

#[test]
fn manifest_with_dependencies() {
    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    let deps = vec![(
        "serde".to_string(),
        Dependency {
            name: "Serde".to_string(),
            location: "https://github.com/serde-rs/serde".to_string(),
            version: Some("1.0.137".to_string()),
        },
    )];
    let installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        Some(deps),
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
                name: "Serde",
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
fn manifest_with_namespaces_and_dependencies() {
    #[derive(Facet)]
    #[facet(namespace = "my_namespace")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = RegistryBuilder::new().add_type::<Root>().build();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let deps = vec![(
        "my_namespace".to_string(),
        Dependency {
            name: "MyNamespace".to_string(),
            location: "https://github.com/example/my_namespace".to_string(),
            version: Some("1.0".to_string()),
        },
    )];

    let mut installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        Some(deps),
    );

    for (module, registry) in split(package_name, &registry) {
        installer
            .install_module(module.config(), &registry)
            .unwrap();
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
                name: "MyNamespace",
                url: "https://github.com/example/my_namespace",
                from: "1.0"
            )
        ],
        targets: [
            .target(
                name: "MyPackage",
                dependencies: ["MyNamespace"]
            ),
        ]
    )
    "#);
}
