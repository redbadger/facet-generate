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

/// Every module imports `Swift`, so a standard-library type hides the module
/// even where nothing is a `String`: "'Thing' is not a member type of struct
/// 'Swift.String'".
#[test]
fn rejects_a_namespace_named_like_a_standard_library_type() {
    #[derive(Facet)]
    #[facet(fg::namespace = "string")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "Swift: namespace \"string\" becomes the module `String`, which qualifies its types in \
         module `Example`, the same as the standard-library type `String`, which Swift finds \
         instead of the module. Choose a different namespace"
    );
}

/// A standard-library type that the generated code never writes hides the
/// module too: "'Thing' is not a member type of protocol 'Swift.Error'".
#[test]
fn rejects_a_namespace_named_like_swift_error() {
    #[derive(Facet)]
    #[facet(fg::namespace = "error")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "Swift: namespace \"error\" becomes the module `Error`, which qualifies its types in \
         module `Example`, the same as the standard-library type `Error`, which Swift finds \
         instead of the module. Choose a different namespace"
    );
}

/// `Result` too: "'Thing' is not a member type of generic enum
/// 'Swift.Result'".
#[test]
fn rejects_a_namespace_named_like_swift_result() {
    #[derive(Facet)]
    #[facet(fg::namespace = "result")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "Swift: namespace \"result\" becomes the module `Result`, which qualifies its types in \
         module `Example`, the same as the standard-library type `Result`, which Swift finds \
         instead of the module. Choose a different namespace"
    );
}

/// The Serde runtime imports `Foundation`, which imports `System`: "module
/// dependency cycle: 'System (Source Target) -> Serde.swiftmodule ->
/// Foundation.swiftmodule -> System.swiftmodule'".
#[test]
fn rejects_a_namespace_named_like_a_module_foundation_imports() {
    #[derive(Facet)]
    #[facet(fg::namespace = "system")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "Swift: namespace \"system\" becomes the module `System`, the same as the SDK module \
         `System`, which `Foundation` imports, so the target would depend on itself. Choose a \
         different namespace"
    );
}

/// "module dependency cycle: 'Foundation (Source Target) -> Serde.swiftmodule
/// -> Foundation.swiftmodule'".
#[test]
fn rejects_a_namespace_named_like_foundation() {
    #[derive(Facet)]
    #[facet(fg::namespace = "foundation")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "Swift: namespace \"foundation\" becomes the module `Foundation`, the same as the SDK \
         module `Foundation`, which the generated code imports, so the target would depend on itself. \
         Choose a different namespace"
    );
}

/// The compiler refuses the target: "module name "Swift" is reserved for the
/// standard library".
#[test]
fn rejects_a_namespace_named_like_the_standard_library_module() {
    #[derive(Facet)]
    #[facet(fg::namespace = "swift")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    assert_eq!(
        rejection("Example", &reflect!(App).unwrap()),
        "Swift: namespace \"swift\" becomes the module `Swift`, the same as the standard \
         library's module `Swift`. Choose a different namespace"
    );
}

/// Both would be the `Serde` target, so the installer found a cycle from
/// `Serde` to itself instead.
#[test]
fn rejects_a_namespace_named_like_the_serde_runtime() {
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
        "Swift: namespace \"serde\" becomes the module `Serde`, the same as the runtime module \
         `Serde`, which the generated code imports. Choose a different namespace"
    );
}

/// A module named like an SDK module is fine when nothing imports
/// `Foundation`, as without a plugin.
#[test]
fn allows_a_namespace_named_like_an_sdk_module_without_foundation() {
    #[derive(Facet)]
    #[facet(fg::namespace = "system")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct App {
        thing: Thing,
    }

    let dir = tempfile::tempdir().unwrap();
    Installer::new("Example", dir.path())
        .generate(&reflect!(App).unwrap())
        .unwrap();
}

/// A module named like a Foundation type that the generated code spells
/// otherwise (`Uuid`, not `UUID`) is fine, and so is one named like a
/// runtime target that the generated code never imports (`Bincode`).
#[test]
fn allows_namespaces_near_builtin_names() {
    #[derive(Facet)]
    #[facet(fg::namespace = "uuid")]
    struct Id {
        x: u32,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "bincode")]
    struct Codec {
        x: u32,
    }

    #[derive(Facet)]
    struct Store {
        id: Id,
        codec: Codec,
        uuid: uuid::Uuid,
    }

    generates("Example", &reflect!(Store).unwrap());
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
