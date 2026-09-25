//! Tests for the TypeScript installer's namespace collision check (#153):
//! every rejected registry fails to type-check, or loses a module, when
//! generated without the check, and every allowed one type-checks.

use facet::Facet;

use crate as fg;
use crate::{
    Registry,
    generation::{
        Error, ExternalPackage, PackageLocation, bincode::BincodePlugin, json::JsonPlugin,
        typescript::installer::Installer,
    },
    reflect,
};

/// The error generating `registry` fails with, after checking that nothing
/// was written.
fn rejection(package: &str, registry: &Registry) -> String {
    rejection_with(package, registry, &[])
}

/// [`rejection`], with `external_packages` provided by external packages.
fn rejection_with(
    package: &str,
    registry: &Registry,
    external_packages: &[ExternalPackage],
) -> String {
    let dir = tempfile::tempdir().unwrap();
    let error = Installer::new(package, dir.path())
        .plugin(BincodePlugin)
        .external_packages(external_packages)
        .generate(registry)
        .unwrap_err();
    assert!(
        std::fs::read_dir(dir.path()).unwrap().next().is_none(),
        "nothing is written"
    );
    let Error::Io(error) = error else {
        panic!("expected an I/O error, got {error:?}");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    error.to_string()
}

fn generates(package: &str, registry: &Registry) {
    let dir = tempfile::tempdir().unwrap();
    Installer::new(package, dir.path())
        .plugin(BincodePlugin)
        .generate(registry)
        .unwrap();
}

/// "TS2440: Import declaration conflicts with local declaration of 'Kv'".
#[test]
fn rejects_a_root_type_named_like_a_namespace() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Get {
        key: String,
    }

    #[derive(Facet)]
    struct Kv {
        size: u32,
    }

    #[derive(Facet)]
    struct App {
        get: Get,
        kv: Kv,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "TypeScript: namespace \"kv\" is imported as `Kv` in `Example.ts`, the same as type `Kv` \
         in the root namespace. Rename the type with `#[facet(rename = \"...\")]` or choose a \
         different namespace"
    );
}

/// A sibling namespace's import conflicts with a declaration the same way
/// (TS2440).
#[test]
fn rejects_a_type_named_like_a_namespace_its_module_imports() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Get {
        key: String,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "other")]
    struct Kv {
        get: Get,
    }

    #[derive(Facet)]
    struct App {
        kv: Kv,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "TypeScript: namespace \"kv\" is imported as `Kv` in `other.ts`, the same as type `Kv` \
         in namespace \"other\". Rename the type with `#[facet(rename = \"...\")]` or choose a \
         different namespace"
    );
}

/// A namespaced module imports the root package's module to reach ROOT
/// types, which a type of its own named like the package conflicts with
/// (TS2440).
#[test]
fn rejects_a_type_named_like_the_root_package() {
    #[derive(Facet)]
    #[facet(fg::namespace)]
    struct Leaf {
        n: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "shared")]
    struct Example {
        leaf: Leaf,
    }

    assert_eq!(
        rejection("Example", &reflect!(Example).unwrap()),
        "TypeScript: the root package \"Example\" is imported as `Example` in `shared.ts`, the \
         same as type `Example` in namespace \"shared\". Rename the type with \
         `#[facet(rename = \"...\")]` or choose a different package name"
    );
}

/// `import * as MyNs` is written once, for `my_ns`, so `MyNs.B` does not
/// exist (TS2694).
#[test]
fn rejects_namespaces_imported_under_the_same_name() {
    #[derive(Facet)]
    #[facet(fg::namespace = "my_ns")]
    struct A {
        x: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "MyNs")]
    struct B {
        y: u32,
    }

    #[derive(Facet)]
    struct Root {
        a: A,
        b: B,
    }

    assert_eq!(
        rejection("Example", &reflect!(Root).unwrap()),
        "TypeScript: namespace \"my_ns\" is imported as `MyNs` in `Example.ts`, the same as \
         namespace \"MyNs\". Choose a different namespace"
    );
}

/// `app.ts` and `App.ts` are one file on a case-insensitive file system, so
/// one module would be lost there.
#[test]
fn rejects_a_namespace_whose_file_differs_from_the_root_package_only_in_case() {
    #[derive(Facet)]
    #[facet(fg::namespace = "app")]
    struct Inner {
        x: u32,
    }

    #[derive(Facet)]
    struct Outer {
        inner: Inner,
    }

    assert_eq!(
        rejection("App", &reflect!(Outer).unwrap()),
        "TypeScript: namespace \"app\" is written to `app.ts`, the same as the root package \
         \"App\", whose `App.ts` is the same file on a case-insensitive file system. Choose a \
         different namespace or package name"
    );
}

/// "TS1261: Already included file name 'kv.ts' differs from file name
/// 'Kv.ts' only in casing".
#[test]
fn rejects_namespaces_whose_files_differ_only_in_case() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct A {
        x: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "Kv")]
    struct B {
        y: u32,
    }

    #[derive(Facet)]
    struct Root {
        a: A,
        b: B,
    }

    assert_eq!(
        rejection("Example", &reflect!(Root).unwrap()),
        "TypeScript: namespace \"kv\" is written to `kv.ts`, the same as namespace \"Kv\", whose \
         `Kv.ts` is the same file on a case-insensitive file system. Choose a different namespace"
    );
}

/// `deserializeMap` constructs the global, which the import shadows:
/// "TS2351: This expression is not constructable".
#[test]
fn rejects_a_namespace_named_like_a_global_the_module_constructs() {
    #[derive(Facet)]
    #[facet(fg::namespace = "map")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
        m: std::collections::HashMap<String, u32>,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "TypeScript: namespace \"map\" is imported as `Map` in `Example.ts`, the same as the \
         global `Map`, which `Example.ts` constructs, so `new Map(...)` would find the namespace \
         instead. Choose a different namespace"
    );
}

/// An enum's `deserialize` function throws `new Error(...)` (TS2351).
#[test]
fn rejects_a_namespace_named_like_a_global_an_enum_constructs() {
    #[derive(Facet)]
    #[facet(fg::namespace = "error")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Choice {
        A,
        B(u32),
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
        choice: Choice,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "TypeScript: namespace \"error\" is imported as `Error` in `Example.ts`, the same as the \
         global `Error`, which `Example.ts` constructs, so `new Error(...)` would find the namespace \
         instead. Choose a different namespace"
    );
}

/// The Bincode `Uuid` helper constructs a `Uint8Array` (TS2351).
#[test]
fn rejects_a_namespace_named_like_a_global_the_uuid_helper_constructs() {
    #[derive(Facet)]
    #[facet(fg::namespace = "uint8_array")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
        id: uuid::Uuid,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "TypeScript: namespace \"uint8_array\" is imported as `Uint8Array` in `Example.ts`, the \
         same as the global `Uint8Array`, which `Example.ts` constructs, so `new Uint8Array(...)` \
         would find the namespace instead. Choose a different namespace"
    );
}

/// `./serde` resolves to `serde.ts` before `serde/index.ts`: "TS2459:
/// Module '"./serde"' declares 'Serializer' locally, but it is not exported".
#[test]
fn rejects_a_namespace_written_over_the_serde_runtime() {
    #[derive(Facet)]
    #[facet(fg::namespace = "serde")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "TypeScript: namespace \"serde\" is written to `serde.ts`, the same as the runtime module \
         `./serde`, which the generated code imports `Serializer` and `Deserializer` from. Choose \
         a different namespace"
    );
}

/// The JSON plugin's code imports `./serde/json`, which `serde.ts` does not
/// shadow.
#[test]
fn allows_a_namespace_named_serde_beside_the_json_runtime() {
    #[derive(Facet)]
    #[facet(fg::namespace = "serde")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    let dir = tempfile::tempdir().unwrap();
    Installer::new("Example", dir.path())
        .plugin(JsonPlugin)
        .generate(&reflect!(App).unwrap())
        .unwrap();
    assert!(dir.path().join("serde.ts").exists());
    assert!(dir.path().join("serde/json.ts").exists());
}

/// A global written only as a type (`Map<K, V>`, `type bytes = Uint8Array`)
/// is still found through a namespace import of the same name, and one that
/// nothing constructs is not written at all.
#[test]
fn allows_a_namespace_named_like_a_global_the_module_does_not_construct() {
    #[derive(Facet)]
    #[facet(fg::namespace = "map")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "uint8_array")]
    struct Other {
        y: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
        other: Other,
        #[facet(fg::bytes)]
        bytes: Vec<u8>,
    }

    generates("Example", &reflect!(App).unwrap());
}

/// Without a plugin nothing constructs a global.
#[test]
fn allows_a_namespace_named_like_a_global_without_plugins() {
    #[derive(Facet)]
    #[facet(fg::namespace = "map")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
        m: std::collections::HashMap<String, u32>,
    }

    let dir = tempfile::tempdir().unwrap();
    Installer::new("Example", dir.path())
        .generate(&reflect!(App).unwrap())
        .unwrap();
}

/// The import is only written where the namespace is referenced, and the
/// root module, which imports `kv`, declares no `Kv`.
#[test]
fn allows_a_type_named_like_its_own_namespace() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Kv {
        x: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Get {
        key: String,
    }

    #[derive(Facet)]
    struct App {
        kv: Kv,
        get: Get,
    }

    generates("Example", &reflect!(App).unwrap());
}

/// `module::split` puts a namespace spelled exactly like the root package in
/// the root module, so it is not a module of its own.
#[test]
fn allows_a_namespace_spelled_like_the_root_package() {
    #[derive(Facet)]
    #[facet(fg::namespace = "Example")]
    struct Inner {
        x: u32,
    }

    #[derive(Facet)]
    struct Outer {
        inner: Inner,
    }

    generates("Example", &reflect!(Outer).unwrap());
}

/// A ROOT type named like the root package is fine: the root module never
/// imports itself, and the namespaced module declares no `Example`.
#[test]
fn allows_a_root_type_named_like_the_root_package() {
    #[derive(Facet)]
    #[facet(fg::namespace = "shared")]
    struct Row {
        label: String,
        owner: Leaf,
    }

    #[derive(Facet)]
    #[facet(fg::namespace)]
    struct Leaf {
        n: u32,
    }

    #[derive(Facet)]
    struct Example {
        row: Row,
    }

    generates("Example", &reflect!(Example).unwrap());
}

/// `module::split` merges namespace "shared" into the root module of package
/// `shared`, which would then be both generated and provided by the external
/// package for namespace "shared" (#186).
#[test]
fn rejects_a_root_package_named_like_an_external_namespace() {
    #[derive(Facet)]
    #[facet(fg::namespace = "shared")]
    struct Ext {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        e: Ext,
    }

    let external = [ExternalPackage {
        for_namespace: "shared".to_string(),
        module_name: None,
        location: PackageLocation::Path("../shared".to_string()),
        version: None,
    }];
    let expected = "TypeScript: the root package is \"shared\", the same as namespace \"shared\", \
                    which an external package provides, so it would be merged into the root \
                    module. Choose a different namespace or package name";

    assert_eq!(
        rejection_with("shared", &reflect!(App).unwrap(), &external),
        expected
    );
    // Whether or not the registry has types in that namespace.
    assert_eq!(
        rejection_with("shared", &reflect!(Ext).unwrap(), &external),
        expected
    );
}
