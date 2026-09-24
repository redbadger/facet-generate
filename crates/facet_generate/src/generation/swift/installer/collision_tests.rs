//! Tests for the Swift installer's namespace collision check (#153): every
//! rejected registry fails to build with `swift build` when generated without
//! the check, and every allowed one builds.

use facet::Facet;

use crate as fg;
use crate::{
    Registry,
    generation::{Error, bincode::BincodePlugin, swift::installer::Installer},
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

/// `import Kv` then `Kv.Get` finds the struct `Kv` first:
/// "'Get' is not a member type of struct 'Example.Kv'".
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
        "Swift: namespace \"kv\" becomes the module `Kv`, which qualifies its types in module \
         `Example`, the same as type `Kv` in the root namespace. Rename the type with \
         `#[facet(rename = \"...\")]` or choose a different namespace"
    );
}

/// Swift finds a type of an imported module before a module, so a type in
/// namespace `kv` named `Kv` hides its own module from every importer:
/// "'Get' is not a member type of struct 'Kv.Kv'".
#[test]
fn rejects_a_type_named_like_its_own_namespace_module() {
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

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "Swift: namespace \"kv\" becomes the module `Kv`, which qualifies its types in module \
         `Example`, the same as type `Kv` in namespace \"kv\". Rename the type with \
         `#[facet(rename = \"...\")]` or choose a different namespace"
    );
}

/// A type of another imported module hides the qualifier too:
/// "'Get' is not a member type of struct 'Other.Kv'".
#[test]
fn rejects_a_type_of_another_imported_module_named_like_a_namespace() {
    #[derive(Facet)]
    #[facet(fg::namespace = "other")]
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

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "Swift: namespace \"kv\" becomes the module `Kv`, which qualifies its types in module \
         `Example`, the same as type `Kv` in namespace \"other\". Rename the type with \
         `#[facet(rename = \"...\")]` or choose a different namespace"
    );
}

/// A namespaced module qualifies ROOT types with the root package's module,
/// which a ROOT type named like the package hides:
/// "'Leaf' is not a member type of struct 'Example.Example'".
#[test]
fn rejects_a_type_named_like_the_root_package() {
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

    assert_eq!(
        rejection("Example", &reflect!(Row, Example).unwrap()),
        "Swift: the root package \"Example\" becomes the module `Example`, which qualifies its \
         types in module `Shared`, the same as type `Example` in the root namespace. Rename the \
         type with `#[facet(rename = \"...\")]` or choose a different package name"
    );
}

/// Both would be written to `Sources/App/App.swift`, losing one module, and
/// the `App` target would depend on itself.
#[test]
fn rejects_a_namespace_that_becomes_the_root_package_module() {
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
        "Swift: namespace \"app\" is written to `Sources/App/App.swift`, the same as the root \
         package \"App\". Choose a different namespace or package name"
    );
}

/// Only one `Sources/Kv/Kv.swift` is written: "no type named 'B' in module 'Kv'".
#[test]
fn rejects_namespaces_that_become_the_same_module() {
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
        "Swift: namespace \"kv\" is written to `Sources/Kv/Kv.swift`, the same as namespace \
         \"Kv\". Choose a different namespace"
    );
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

/// A module named like a type that nothing in scope declares is fine, and so
/// is a namespace named like a builtin that no module qualifies with it.
#[test]
fn allows_distinct_namespaces_and_types() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Get {
        key: String,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "unit")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct Store {
        size: u32,
    }

    #[derive(Facet)]
    struct App {
        get: Get,
        thing: Thing,
        store: Store,
        nothing: (),
    }

    generates("Example", &reflect!(App).unwrap());
}

/// A type named like its own namespace is fine while no other module refers
/// to that namespace.
#[test]
fn allows_a_type_named_like_its_own_namespace_that_no_other_module_imports() {
    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Kv {
        x: u32,
    }

    generates("Example", &reflect!(Kv).unwrap());
}
