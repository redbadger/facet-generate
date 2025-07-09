use facet::Facet;

use crate::{
    generation::{SourceInstaller as _, module::split},
    reflection::RegistryBuilder,
};

use super::*;

#[test]
fn simple_manifest() {
    let package_name = "MyPackage";
    let install_dir = std::path::PathBuf::from("/tmp");
    let deps = vec![Dependency {
        name: "Serde".to_string(),
        location: "https://github.com/serde-rs/serde".to_string(),
        version: Some("1.0.137".to_string()),
    }];
    let installer = Installer::new(package_name.to_string(), install_dir, Some(deps));

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
    let install_dir = std::path::PathBuf::from("/tmp");
    let mut installer = Installer::new(package_name.to_string(), install_dir, None);

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
                    dependencies: ["Serde"]
                ),
                .target(
                    name: "MyPackage",
                    dependencies: ["MyNamespace", "Serde"]
                ),
                .target(
                    name: "Serde",
                    dependencies: []
                ),
            ]
        )
        "#);
}
