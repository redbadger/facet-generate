use facet::Facet;

use crate::{
    generation::{
        ExternalPackage, PackageLocation, SourceInstaller as _,
        module::split,
        typescript::{InstallTarget, installer::Installer},
    },
    reflection::RegistryBuilder,
};

#[test]
fn simple_manifest() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(install_dir.path(), &[], InstallTarget::Node);

    let manifest = installer.make_manifest(package_name);

    insta::assert_json_snapshot!(manifest, @r#"
    {
      "devDependencies": {
        "typescript": "^5.8.3"
      },
      "name": "my-package",
      "version": "0.1.0"
    }
    "#);
}

#[test]
fn manifest_with_dependencies() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_packages = vec![
        ExternalPackage {
            for_namespace: "lodash".to_string(),
            location: PackageLocation::Url("https://registry.npmjs.org/lodash".to_string()),
            module_name: None,
            version: Some("^4.17.21".to_string()),
        },
        ExternalPackage {
            for_namespace: "axios".to_string(),
            location: PackageLocation::Url("https://registry.npmjs.org/axios".to_string()),
            module_name: None,
            version: Some("^1.6.0".to_string()),
        },
    ];

    let installer = Installer::new(install_dir.path(), &external_packages, InstallTarget::Node);

    let manifest = installer.make_manifest(package_name);

    insta::assert_json_snapshot!(manifest, @r#"
    {
      "dependencies": {
        "axios": "^1.6.0",
        "lodash": "^4.17.21"
      },
      "devDependencies": {
        "typescript": "^5.8.3"
      },
      "name": "my-package",
      "version": "0.1.0"
    }
    "#);
}

#[test]
fn manifest_with_local_dependencies() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_packages = vec![ExternalPackage {
        for_namespace: "shared-types".to_string(),
        location: PackageLocation::Path("../shared-types".to_string()),
        module_name: None,
        version: None,
    }];

    let installer = Installer::new(install_dir.path(), &external_packages, InstallTarget::Node);

    let manifest = installer.make_manifest(package_name);
    insta::assert_json_snapshot!(manifest, @r#"
    {
      "dependencies": {
        "shared-types": "file:../shared-types"
      },
      "devDependencies": {
        "typescript": "^5.8.3"
      },
      "name": "my-package",
      "version": "0.1.0"
    }
    "#);
}

#[test]
fn manifest_with_mixed_dependencies() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_packages = vec![
        ExternalPackage {
            for_namespace: "lodash".to_string(),
            location: PackageLocation::Url("https://registry.npmjs.org/lodash".to_string()),
            module_name: None,
            version: Some("^4.17.21".to_string()),
        },
        ExternalPackage {
            for_namespace: "shared-types".to_string(),
            location: PackageLocation::Path("../shared-types".to_string()),
            module_name: None,

            version: None,
        },
    ];

    let installer = Installer::new(install_dir.path(), &external_packages, InstallTarget::Node);

    let manifest = installer.make_manifest(package_name);
    insta::assert_json_snapshot!(manifest, @r#"
    {
      "dependencies": {
        "lodash": "^4.17.21",
        "shared-types": "file:../shared-types"
      },
      "devDependencies": {
        "typescript": "^5.8.3"
      },
      "name": "my-package",
      "version": "0.1.0"
    }
    "#);
}

#[test]
fn manifest_with_serde_module() {
    #[derive(Facet)]
    struct MyStruct {
        id: u32,
        name: String,
    }

    let registry = RegistryBuilder::new().add_type::<MyStruct>().build();

    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(install_dir.path(), &[], InstallTarget::Node);

    for (module, registry) in split(package_name, &registry) {
        installer
            .install_module(module.config(), &registry)
            .unwrap();
    }

    installer.install_serde_runtime().unwrap();

    let manifest = installer.make_manifest(package_name);
    insta::assert_json_snapshot!(manifest, @r#"
    {
      "devDependencies": {
        "typescript": "^5.8.3"
      },
      "name": "my-package",
      "version": "0.1.0"
    }
    "#);
}

#[test]
fn manifest_with_namespaces() {
    #[derive(Facet)]
    #[facet(namespace = "another_module")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = RegistryBuilder::new().add_type::<Root>().build();

    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();
    let mut installer = Installer::new(install_dir.path(), &[], InstallTarget::Node);

    for (module, registry) in split(package_name, &registry) {
        installer
            .install_module(module.config(), &registry)
            .unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_json_snapshot!(manifest, @r#"
    {
      "devDependencies": {
        "typescript": "^5.8.3"
      },
      "name": "my-package",
      "version": "0.1.0"
    }
    "#);
}

#[test]
fn manifest_with_external_namespace_dependencies() {
    #[derive(Facet)]
    #[facet(namespace = "external_package")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = RegistryBuilder::new().add_type::<Root>().build();

    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_packages = vec![ExternalPackage {
        for_namespace: "external_package".to_string(),
        location: PackageLocation::Url("https://registry.npmjs.org/external-package".to_string()),
        module_name: None,
        version: Some("^1.0.0".to_string()),
    }];

    let mut installer = Installer::new(install_dir.path(), &external_packages, InstallTarget::Node);

    for (module, registry) in split(package_name, &registry) {
        installer
            .install_module(module.config(), &registry)
            .unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    insta::assert_json_snapshot!(manifest, @r#"
    {
      "dependencies": {
        "external-package": "^1.0.0"
      },
      "devDependencies": {
        "typescript": "^5.8.3"
      },
      "name": "my-package",
      "version": "0.1.0"
    }
    "#);
}

#[test]
fn manifest_with_scoped_package() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_packages = vec![ExternalPackage {
        for_namespace: "types".to_string(),
        location: PackageLocation::Url("https://registry.npmjs.org/@types/node".to_string()),
        module_name: None,
        version: Some("^20.0.0".to_string()),
    }];

    let installer = Installer::new(install_dir.path(), &external_packages, InstallTarget::Node);

    let manifest = installer.make_manifest(package_name);
    insta::assert_json_snapshot!(manifest, @r#"
    {
      "dependencies": {
        "@types/node": "^20.0.0"
      },
      "devDependencies": {
        "typescript": "^5.8.3"
      },
      "name": "my-package",
      "version": "0.1.0"
    }
    "#);
}
