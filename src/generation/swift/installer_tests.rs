use facet::Facet;

use crate::{
    generation::{
        ExternalPackage, PackageLocation, SourceInstaller as _, module::split,
        swift::installer::Installer,
    },
    reflection::RegistryBuilder,
};

#[test]
fn simple_manifest() {
    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        vec![],
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
fn manifest_with_serde_as_target() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
        name: String,
    }

    let registry = RegistryBuilder::new().add_type::<MyStruct>().build();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        vec![],
    );

    for (module, registry) in split(package_name, &registry) {
        installer
            .install_module(module.config(), &registry)
            .unwrap();
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

    let registry = RegistryBuilder::new().add_type::<MyStruct>().build();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        vec![ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Url("https://github.com/serde-rs/serde".to_string()),
            version: Some("1.0.137".to_string()),
        }],
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

    let registry = RegistryBuilder::new().add_type::<MyStruct>().build();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        vec![ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Path("../Serde".to_string()),
            version: None,
        }],
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

    let registry = RegistryBuilder::new().add_type::<Root>().build();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();
    let mut installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        vec![],
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
fn manifest_with_remote_dependencies() {
    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        vec![ExternalPackage {
            for_namespace: "AnotherPackage".to_string(),
            location: PackageLocation::Url(
                "https://github.com/example/another_package".to_string(),
            ),
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

    let registry = RegistryBuilder::new().add_type::<Root>().build();

    let package_name = "MyPackage";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(
        package_name.to_string(),
        install_dir.path().to_path_buf(),
        vec![ExternalPackage {
            for_namespace: "another_package".to_string(),
            location: PackageLocation::Url(
                "https://github.com/example/another_package".to_string(),
            ),
            version: Some("1.0".to_string()),
        }],
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
