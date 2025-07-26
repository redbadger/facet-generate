use facet::Facet;
use serde_json::Value;

use crate::{
    generation::{
        ExternalPackage, PackageLocation, SourceInstaller as _, module::split,
        typescript::installer::Installer,
    },
    reflection::RegistryBuilder,
};

#[test]
fn simple_manifest() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(install_dir.path(), &[], false);

    let manifest = installer.make_manifest(package_name);
    let expected: Value = serde_json::json!({
        "name": "my-package",
        "version": "0.1.0",
        "devDependencies": {
            "typescript": "^5.8.3"
        }
    });

    assert_eq!(manifest, expected);
}

#[test]
fn manifest_with_dependencies() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_packages = vec![
        ExternalPackage {
            for_namespace: "lodash".to_string(),
            location: PackageLocation::Url("https://registry.npmjs.org/lodash".to_string()),
            version: Some("^4.17.21".to_string()),
        },
        ExternalPackage {
            for_namespace: "axios".to_string(),
            location: PackageLocation::Url("https://registry.npmjs.org/axios".to_string()),
            version: Some("^1.6.0".to_string()),
        },
    ];

    let installer = Installer::new(install_dir.path(), &external_packages, false);

    let manifest = installer.make_manifest(package_name);
    let expected: Value = serde_json::json!({
        "name": "my-package",
        "version": "0.1.0",
        "dependencies": {
            "lodash": "^4.17.21",
            "axios": "^1.6.0"
        },
        "devDependencies": {
            "typescript": "^5.8.3"
        }
    });

    assert_eq!(manifest, expected);
}

#[test]
fn manifest_with_local_dependencies() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_packages = vec![ExternalPackage {
        for_namespace: "shared-types".to_string(),
        location: PackageLocation::Path("../shared-types".to_string()),
        version: None,
    }];

    let installer = Installer::new(install_dir.path(), &external_packages, false);

    let manifest = installer.make_manifest(package_name);
    let expected: Value = serde_json::json!({
        "name": "my-package",
        "version": "0.1.0",
        "dependencies": {
            "shared-types": "file:../shared-types"
        },
        "devDependencies": {
            "typescript": "^5.8.3"
        }
    });

    assert_eq!(manifest, expected);
}

#[test]
fn manifest_with_mixed_dependencies() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_packages = vec![
        ExternalPackage {
            for_namespace: "lodash".to_string(),
            location: PackageLocation::Url("https://registry.npmjs.org/lodash".to_string()),
            version: Some("^4.17.21".to_string()),
        },
        ExternalPackage {
            for_namespace: "shared-types".to_string(),
            location: PackageLocation::Path("../shared-types".to_string()),
            version: None,
        },
    ];

    let installer = Installer::new(install_dir.path(), &external_packages, false);

    let manifest = installer.make_manifest(package_name);
    let expected: Value = serde_json::json!({
        "name": "my-package",
        "version": "0.1.0",
        "dependencies": {
            "lodash": "^4.17.21",
            "shared-types": "file:../shared-types"
        },
        "devDependencies": {
            "typescript": "^5.8.3"
        }
    });

    assert_eq!(manifest, expected);
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

    let mut installer = Installer::new(install_dir.path(), &[], false);

    for (module, registry) in split(package_name, &registry) {
        installer
            .install_module(module.config(), &registry)
            .unwrap();
    }

    installer.install_serde_runtime().unwrap();

    let manifest = installer.make_manifest(package_name);
    let expected: Value = serde_json::json!({
        "name": "my-package",
        "version": "0.1.0",
        "devDependencies": {
            "typescript": "^5.8.3"
        }
    });

    assert_eq!(manifest, expected);
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
    let mut installer = Installer::new(install_dir.path(), &[], false);

    for (module, registry) in split(package_name, &registry) {
        installer
            .install_module(module.config(), &registry)
            .unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    let expected: Value = serde_json::json!({
        "name": "my-package",
        "version": "0.1.0",
        "devDependencies": {
            "typescript": "^5.8.3"
        }
    });

    assert_eq!(manifest, expected);
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
        version: Some("^1.0.0".to_string()),
    }];

    let mut installer = Installer::new(install_dir.path(), &external_packages, false);

    for (module, registry) in split(package_name, &registry) {
        installer
            .install_module(module.config(), &registry)
            .unwrap();
    }

    let manifest = installer.make_manifest(package_name);
    let expected: Value = serde_json::json!({
        "name": "my-package",
        "version": "0.1.0",
        "dependencies": {
            "external-package": "^1.0.0"
        },
        "devDependencies": {
            "typescript": "^5.8.3"
        }
    });

    assert_eq!(manifest, expected);
}

#[test]
fn manifest_with_scoped_package() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_packages = vec![ExternalPackage {
        for_namespace: "types".to_string(),
        location: PackageLocation::Url("https://registry.npmjs.org/@types/node".to_string()),
        version: Some("^20.0.0".to_string()),
    }];

    let installer = Installer::new(install_dir.path(), &external_packages, false);

    let manifest = installer.make_manifest(package_name);
    let expected: Value = serde_json::json!({
        "name": "my-package",
        "version": "0.1.0",
        "dependencies": {
            "@types/node": "^20.0.0"
        },
        "devDependencies": {
            "typescript": "^5.8.3"
        }
    });

    assert_eq!(manifest, expected);
}
