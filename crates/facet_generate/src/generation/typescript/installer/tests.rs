//! Snapshot tests for the TypeScript [`Installer`] — **project scaffolding**.
//!
//! These tests verify the `package.json` manifest that the installer
//! generates, and the file layout produced by `install_module`. They cover:
//!
//! - Basic manifest structure: package name, version, devDependencies.
//! - External URL dependencies: registry package references with version
//!   strings (including scoped `@org/package` names).
//! - External path dependencies: local file-system dependencies via
//!   `file:` paths.
//! - Serde/bincode runtime installation.
//! - Multi-module (namespace) scenarios where each namespace becomes a
//!   separate `.ts` file.
//! - Plugin-provided dependency pairs, merged with the external ones.
//! - Plugin-declared type references, imported like the registry's own.

use facet::Facet;

use crate as fg;
use crate::{
    generation::{
        CodeGeneratorConfig, ExternalPackage, PackageLocation, SourceInstaller as _, module::split,
        plugin::EmitterPlugin, typescript::TypeScript,
    },
    reflect,
    reflection::format::QualifiedTypeName,
};

use super::Installer;

#[test]
fn simple_manifest() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(package_name, install_dir.path());

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

    let external_pkgs = vec![
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

    let installer =
        Installer::new(package_name, install_dir.path()).external_packages(&external_pkgs);

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

    let external_pkgs = vec![ExternalPackage {
        for_namespace: "shared-types".to_string(),
        location: PackageLocation::Path("../shared-types".to_string()),
        module_name: None,
        version: None,
    }];

    let installer =
        Installer::new(package_name, install_dir.path()).external_packages(&external_pkgs);

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

    let external_pkgs = vec![
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

    let installer =
        Installer::new(package_name, install_dir.path()).external_packages(&external_pkgs);

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

    let registry = reflect!(MyStruct).unwrap();

    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let mut installer = Installer::new(package_name, install_dir.path());

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
    #[facet(fg::namespace = "another_module")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = reflect!(Root).unwrap();

    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();
    let mut installer = Installer::new(package_name, install_dir.path());

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
    #[facet(fg::namespace = "external_package")]
    struct Child {
        name: String,
    }

    #[derive(Facet)]
    struct Root {
        child: Child,
    }

    let registry = reflect!(Root).unwrap();

    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let external_pkgs = vec![ExternalPackage {
        for_namespace: "external_package".to_string(),
        location: PackageLocation::Url("https://registry.npmjs.org/external-package".to_string()),
        module_name: None,
        version: Some("^1.0.0".to_string()),
    }];

    let mut installer =
        Installer::new(package_name, install_dir.path()).external_packages(&external_pkgs);

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

    let external_pkgs = vec![ExternalPackage {
        for_namespace: "types".to_string(),
        location: PackageLocation::Url("https://registry.npmjs.org/@types/node".to_string()),
        module_name: None,
        version: Some("^20.0.0".to_string()),
    }];

    let installer =
        Installer::new(package_name, install_dir.path()).external_packages(&external_pkgs);

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

/// A plugin standing in for one that bridges to an FFI package: it contributes
/// a `package.json` dependency pair.
#[derive(Debug)]
struct FfiPlugin;

impl EmitterPlugin<TypeScript> for FfiPlugin {
    fn manifest_dependencies(&self) -> Vec<String> {
        vec![r#""shared": "file:../pkg""#.to_string()]
    }
}

#[test]
fn manifest_with_plugin_dependencies() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(package_name, install_dir.path()).plugin(FfiPlugin);

    let manifest = installer.make_manifest(package_name);

    insta::assert_json_snapshot!(manifest, @r#"
    {
      "dependencies": {
        "shared": "file:../pkg"
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
fn manifest_with_plugin_and_external_dependencies() {
    let package_name = "my-package";
    let install_dir = tempfile::tempdir().unwrap();

    let installer = Installer::new(package_name, install_dir.path())
        .external_packages(&[ExternalPackage {
            for_namespace: "serde".to_string(),
            location: PackageLocation::Path("../serde".to_string()),
            module_name: None,
            version: None,
        }])
        .plugin(FfiPlugin);

    let manifest = installer.make_manifest(package_name);

    insta::assert_json_snapshot!(manifest, @r#"
    {
      "dependencies": {
        "serde": "file:../serde",
        "shared": "file:../pkg"
      },
      "devDependencies": {
        "typescript": "^5.8.3"
      },
      "name": "my-package",
      "version": "0.1.0"
    }
    "#);
}

/// A namespaced module imports the root types it references from the root
/// package's module, bound and located the way every namespace import is:
/// `my-package` is written to `my-package.ts` and bound as `MyPackage`.
#[test]
fn namespaced_module_imports_the_root_package_for_root_types() {
    #[derive(Facet)]
    #[facet(fg::namespace)]
    struct Shared {
        id: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Entry {
        shared: Shared,
    }

    #[derive(Facet)]
    struct App {
        entry: Entry,
        shared: Shared,
    }

    let registry = reflect!(App).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("my-package", install_dir.path())
        .generate(&registry)
        .unwrap();

    let kv = std::fs::read_to_string(install_dir.path().join("kv.ts")).unwrap();
    insta::assert_snapshot!(kv, @r#"
    import * as MyPackage from "./my-package";

    export class Entry {
        constructor (public shared: MyPackage.Shared) {
        }
    }
    "#);

    // The root module's own references are unchanged.
    let root = std::fs::read_to_string(install_dir.path().join("my-package.ts")).unwrap();
    assert!(root.contains("constructor (public entry: Kv.Entry, public shared: Shared)"));
}

/// A plugin whose output, in the module `module`, names `types`.
#[derive(Debug)]
struct ReferencesPlugin {
    module: &'static str,
    types: Vec<QualifiedTypeName>,
}

impl EmitterPlugin<TypeScript> for ReferencesPlugin {
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

/// A namespace that only a plugin's output names is imported all the same.
#[test]
fn a_plugin_s_reference_imports_its_namespace() {
    #[derive(Facet)]
    struct App {
        id: u32,
    }

    let registry = reflect!(App, Presence).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("my-package", install_dir.path())
        .plugin(ReferencesPlugin {
            module: "my-package",
            types: vec![kit_presence()],
        })
        .generate(&registry)
        .unwrap();

    let root = std::fs::read_to_string(install_dir.path().join("my-package.ts")).unwrap();
    insta::assert_snapshot!(root, @r#"
    import * as Kit from "./kit";
    type uint32 = number;

    export class App {
        constructor (public id: uint32) {
        }
    }
    "#);
}

/// A namespace that both the registry and a plugin reference is imported once.
#[test]
fn a_plugin_s_reference_to_a_namespace_the_registry_references_is_imported_once() {
    #[derive(Facet)]
    struct App {
        presence: Presence,
    }

    let registry = reflect!(App).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("my-package", install_dir.path())
        .plugin(ReferencesPlugin {
            module: "my-package",
            types: vec![kit_presence(), kit_presence()],
        })
        .generate(&registry)
        .unwrap();

    let root = std::fs::read_to_string(install_dir.path().join("my-package.ts")).unwrap();
    assert_eq!(
        root.matches(r#"import * as Kit from "./kit";"#).count(),
        1,
        "{root}"
    );
}

/// A ROOT type a plugin names in a namespaced module is reached through the
/// root package's module, like one the registry references.
#[test]
fn a_plugin_s_reference_to_a_root_type_imports_the_root_package() {
    #[derive(Facet)]
    struct App {
        id: u32,
    }

    let registry = reflect!(App, Presence).unwrap();
    let install_dir = tempfile::tempdir().unwrap();
    Installer::new("my-package", install_dir.path())
        .plugin(ReferencesPlugin {
            module: "kit",
            types: vec![QualifiedTypeName::root("App".to_string())],
        })
        .generate(&registry)
        .unwrap();

    let kit = std::fs::read_to_string(install_dir.path().join("kit.ts")).unwrap();
    assert!(
        kit.starts_with("import * as MyPackage from \"./my-package\";\n"),
        "{kit}"
    );
}

/// A plugin naming a type the registry does not have is a bug in the plugin.
#[test]
fn a_plugin_s_reference_to_an_unregistered_type_is_rejected() {
    #[derive(Facet)]
    struct App {
        id: u32,
    }

    let registry = reflect!(App).unwrap();
    let error = Installer::new("my-package", tempfile::tempdir().unwrap().path())
        .plugin(ReferencesPlugin {
            module: "my-package",
            types: vec![kit_presence()],
        })
        .generate(&registry)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "plugin ReferencesPlugin { module: \"my-package\", types: [QualifiedTypeName { \
         namespace: Named(\"kit\"), name: \"Presence\" }] } declares that module \
         `my-package` references `kit::Presence`, which is not a type in the registry"
    );
}
