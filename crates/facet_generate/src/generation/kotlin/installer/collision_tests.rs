//! Tests for the Kotlin installer's namespace collision check (#153): every
//! rejected registry fails to compile with `kotlinc`, or loses a module, when
//! generated without the check, and every allowed one compiles.

use facet::Facet;

use crate as fg;
use crate::{
    Registry,
    generation::{Error, bincode::BincodePlugin, kotlin::installer::Installer},
    reflect,
};

/// The error generating `registry` fails with, after checking that nothing
/// was written.
fn rejection(package: &str, registry: &Registry) -> String {
    let dir = tempfile::tempdir().unwrap();
    let error = Installer::new(package, dir.path())
        .plugin(BincodePlugin)
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

/// "package conflicts with classifier com.x.Kit".
#[test]
fn rejects_a_namespace_spelled_like_a_root_type() {
    #[derive(Facet)]
    #[facet(fg::namespace = "Kit")]
    struct Row {
        label: String,
    }

    #[derive(Facet)]
    struct Kit {
        size: u32,
    }

    #[derive(Facet)]
    struct App {
        row: Row,
        kit: Kit,
    }

    assert_eq!(
        rejection("com.x", &reflect!(App).unwrap()),
        "Kotlin: namespace \"Kit\" becomes the package `com.x.Kit`, the same as type `Kit` in the \
         root namespace, the class `com.x.Kit`. Rename the type with \
         `#[facet(rename = \"...\")]` or choose a different namespace"
    );
}

/// Every type reference is qualified with the package, so a type named like
/// its first segment captures it: "unresolved reference 'Leaf'".
#[test]
fn rejects_a_root_type_named_like_the_root_package() {
    #[derive(Facet)]
    struct Leaf {
        n: u32,
    }

    #[derive(Facet)]
    struct Example {
        leaf: Leaf,
    }

    assert_eq!(
        rejection("Example", &reflect!(Example).unwrap()),
        "Kotlin: package `Example` refers to type `Leaf` in the root namespace as \
         `Example.Leaf`, whose first segment is `Example`, the same as type `Example` in the \
         root namespace. Rename the type with `#[facet(rename = \"...\")]` or choose a \
         different package name"
    );
}

/// The same inside a namespace's package, with a type of its own.
#[test]
fn rejects_a_namespaced_type_named_like_the_root_package() {
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
        "Kotlin: package `Example.shared` refers to type `Leaf` in the root namespace as \
         `Example.Leaf`, whose first segment is `Example`, the same as type `Example` in \
         namespace \"shared\". Rename the type with `#[facet(rename = \"...\")]` or choose a \
         different package name"
    );
}

/// `com/example/kv/Kv.kt` and `com/example/Kv/Kv.kt` are one file on a
/// case-insensitive file system: "unresolved reference 'Kv'".
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
        rejection("com.example", &reflect!(Root).unwrap()),
        "Kotlin: namespace \"kv\" is written to `com/example/kv/Kv.kt`, the same as namespace \
         \"Kv\", whose `com/example/Kv/Kv.kt` is the same file on a case-insensitive file \
         system. Choose a different namespace"
    );
}

/// The package keeps the namespace's case, `com.example.kv`, so it does not
/// conflict with the class `com.example.Kv`.
#[test]
fn allows_a_namespace_spelled_like_a_root_type_in_another_case() {
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

    generates("com.example", &reflect!(App).unwrap());
}

/// Packages keep the namespace's spelling, and a type is fine beside a
/// package named like it in another module.
#[test]
fn allows_namespaces_other_languages_reject() {
    #[derive(Facet)]
    #[facet(fg::namespace = "app")]
    struct Inner {
        x: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Kv {
        x: u32,
    }

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
    struct Outer {
        inner: Inner,
        kv: Kv,
        a: A,
        b: B,
    }

    generates("App", &reflect!(Outer).unwrap());
}

/// A namespaced package does not see the root package's classes, so a ROOT
/// type named like the root package does not capture its references.
#[test]
fn allows_a_root_type_named_like_the_root_package_that_the_root_module_does_not_qualify_with() {
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
        n: u32,
    }

    generates("Example", &reflect!(Row, Example).unwrap());
}
