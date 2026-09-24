//! Tests for the C# installer's namespace collision check (#153): every
//! rejected registry fails to build with `dotnet build`, or loses a module,
//! when generated without the check, and every allowed one builds.

use facet::Facet;

use crate as fg;
use crate::{
    Registry,
    generation::{Error, bincode::BincodePlugin, csharp::installer::Installer},
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

/// "CS0101: The namespace 'Example' already contains a definition for 'Kv'".
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
        "C#: namespace \"kv\" becomes `Example.Kv`, the same as type `Kv` in the root namespace. \
         Rename the type with `#[facet(rename = \"...\")]` or choose a different namespace"
    );
}

/// CS0101 again, for a namespace in `UpperCamelCase`.
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
        rejection("Example", &reflect!(App).unwrap()),
        "C#: namespace \"Kit\" becomes `Example.Kit`, the same as type `Kit` in the root \
         namespace. Rename the type with `#[facet(rename = \"...\")]` or choose a different \
         namespace"
    );
}

/// "CS0426: The type name 'Shared' does not exist in the type 'Example'".
#[test]
fn rejects_a_root_type_named_like_the_first_segment_of_a_qualified_reference() {
    #[derive(Facet)]
    #[facet(fg::namespace = "shared")]
    struct Row {
        label: String,
    }

    #[derive(Facet)]
    struct Example {
        row: Row,
    }

    assert_eq!(
        rejection("Example", &reflect!(Example).unwrap()),
        "C#: `Example` refers to type `Row` in namespace \"shared\" as `Example.Shared.Row`, \
         whose first segment is `Example`, the same as type `Example` in the root namespace. \
         Rename the type with `#[facet(rename = \"...\")]` or choose a different package name"
    );
}

/// A namespaced module sees the root package's types, and its own:
/// "CS0426: The type name 'Leaf' does not exist in the type 'Example'".
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
        "C#: `Example.Shared` refers to type `Leaf` in the root namespace as `Example.Leaf`, \
         whose first segment is `Example`, the same as type `Example` in namespace \"shared\". \
         Rename the type with `#[facet(rename = \"...\")]` or choose a different package name"
    );
}

/// `App.App` captures `App.App.Inner`: "CS0234: The type or namespace name
/// 'App' does not exist in the namespace 'App.App'".
#[test]
fn rejects_a_namespace_named_like_the_first_segment_of_the_root_package() {
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
        "C#: `App` refers to type `Inner` in namespace \"app\" as `App.App.Inner`, whose first \
         segment is `App`, the same as namespace \"app\", which becomes `App.App`. Choose a \
         different namespace or package name"
    );
}

/// "CS0118: 'Unit' is a namespace but is used like a type".
#[test]
fn rejects_a_namespace_named_like_a_builtin_the_registry_uses() {
    #[derive(Facet)]
    #[facet(fg::namespace = "unit")]
    struct Thing {
        x: u32,
    }

    #[derive(Facet)]
    struct Top {
        thing: Thing,
        nothing: (),
    }

    assert_eq!(
        rejection("Example", &reflect!(Top).unwrap()),
        "C#: namespace \"unit\" becomes `Example.Unit`, the same as the builtin `Unit`, which the \
         generated code writes unqualified, so every module in `Example` would find the \
         namespace instead. Choose a different namespace"
    );
}

/// `Example/kv/Kv.cs` and `Example/Kv/Kv.cs` are one file on a
/// case-insensitive file system: "CS0234: The type or namespace name 'B'
/// does not exist in the namespace 'Example.Kv'".
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
        "C#: namespace \"kv\" is written to `Example/kv/Kv.cs`, the same as namespace \"Kv\", \
         whose `Example/Kv/Kv.cs` is the same file on a case-insensitive file system. Choose a \
         different namespace"
    );
}

/// A namespace named like a builtin is fine while nothing writes the
/// builtin; so are a type named like its own namespace, two namespaces that
/// become one C# namespace from separate files, and a namespace spelled
/// exactly like the root package, which `module::split` merges into the root
/// module.
#[test]
fn allows_near_misses() {
    #[derive(Facet)]
    #[facet(fg::namespace = "unit")]
    struct Thing {
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
    #[facet(fg::namespace = "Example")]
    struct Inner {
        x: u32,
    }

    #[derive(Facet)]
    struct Top {
        thing: Thing,
        kv: Kv,
        a: A,
        b: B,
        inner: Inner,
    }

    generates("Example", &reflect!(Top).unwrap());
}

/// The root module of an undotted package writes ROOT types bare, so a ROOT
/// type named like the package captures nothing.
#[test]
fn allows_a_root_type_named_like_an_undotted_package_without_qualified_references() {
    #[derive(Facet)]
    struct Leaf {
        n: u32,
    }

    #[derive(Facet)]
    struct Example {
        leaf: Leaf,
    }

    generates("Example", &reflect!(Example).unwrap());
}
