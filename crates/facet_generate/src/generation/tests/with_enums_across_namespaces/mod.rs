use std::{fs, path::Path};

use expect_test::expect_file;
use facet::Facet;
use tempfile::tempdir;

use crate::{
    self as fg, Registry,
    generation::{
        bincode::BincodePlugin,
        csharp,
        json::JsonPlugin,
        kotlin, swift,
        tests::{check, read_files_and_create_expect_dirs},
        typescript,
    },
    reflect, source_dir,
};

/// Writes a registry's generated sources into a directory.
type Generate = fn(&Registry, &Path);

/// Generates `registry` with every installer and encoding whose output depends
/// on whether a referenced type is an enum, and checks each against the
/// snapshots in `snapshots/<layout>`.
///
/// The runtime sources are removed before comparing: they do not depend on the
/// registry, and the other fixtures already cover them.
fn check_all(layout: &str, registry: &Registry) {
    let snapshot_root = source_dir!().join("snapshots").join(layout);

    let targets: [(&str, &[&str], Generate); 7] = [
        ("csharp", &["Facet"], |registry, dir| {
            csharp::Installer::new("Example", dir)
                .plugin(BincodePlugin)
                .generate(registry)
                .unwrap();
        }),
        ("kotlin", &["com/novi"], |registry, dir| {
            kotlin::Installer::new("com.example", dir)
                .plugin(BincodePlugin)
                .generate(registry)
                .unwrap();
        }),
        ("kotlin_json", &["com/novi"], |registry, dir| {
            kotlin::Installer::new("com.example", dir)
                .plugin(JsonPlugin)
                .generate(registry)
                .unwrap();
        }),
        ("swift", &["Sources/Serde"], |registry, dir| {
            swift::Installer::new("Example", dir)
                .plugin(BincodePlugin)
                .generate(registry)
                .unwrap();
        }),
        ("swift_json", &["Sources/Serde"], |registry, dir| {
            swift::Installer::new("Example", dir)
                .plugin(JsonPlugin)
                .generate(registry)
                .unwrap();
        }),
        ("typescript", &["serde", "bincode"], |registry, dir| {
            typescript::Installer::new("example", dir)
                .plugin(BincodePlugin)
                .generate(registry)
                .unwrap();
        }),
        ("typescript_json", &["serde", "bincode"], |registry, dir| {
            typescript::Installer::new("example", dir)
                .plugin(JsonPlugin)
                .generate(registry)
                .unwrap();
        }),
    ];

    for (target, runtime_dirs, generate) in targets {
        let tmp_dir = tempdir().unwrap();
        let tmp_path = tmp_dir.path();
        generate(registry, tmp_path);
        for runtime_dir in runtime_dirs {
            let _ = fs::remove_dir_all(tmp_path.join(runtime_dir));
        }

        let snapshot_dir = snapshot_root.join(target);
        fs::create_dir_all(&snapshot_dir).unwrap();

        for (actual, expected) in read_files_and_create_expect_dirs(tmp_path, &snapshot_dir) {
            check(&actual, &expect_file!(&expected));
        }
    }
}

/// A ROOT type holding enums from namespace `kit`, and a `kit` type holding
/// the same enums.
///
/// TypeScript serializes an enum through the standalone functions beside it
/// and C# a unit-only enum through its `…Bincode` helper class, so both have
/// to know that a referenced type is an enum. They used to know only about
/// the enums of the module being generated, so a reference from ROOT treated
/// `Kit.Presence` as a class (`this.presence.serialize(…)`,
/// `Kit.Presence.deserialize(…)`), which did not compile. The `kit` type
/// guards the case that always worked.
///
/// The ROOT struct `Presence` shares its name with the `kit` enum, and must
/// still be serialized as a class.
#[test]
fn root_to_kit() {
    mod kit {
        use super::*;

        #[derive(Facet)]
        #[repr(C)]
        #[facet(fg::namespace = "kit")]
        #[allow(unused)]
        pub enum Presence {
            Online,
            Offline,
        }

        #[derive(Facet)]
        #[repr(C)]
        #[facet(fg::namespace = "kit")]
        #[allow(unused)]
        pub enum Shape {
            Circle(f64),
            Empty,
        }

        #[derive(Facet)]
        #[facet(fg::namespace = "kit")]
        pub struct Badge {
            pub presence: Presence,
            pub shape: Shape,
        }
    }

    #[derive(Facet)]
    struct Presence {
        since: u64,
    }

    #[derive(Facet)]
    struct Sighting {
        last_seen: Presence,
    }

    #[derive(Facet)]
    struct Card {
        presence: kit::Presence,
        shape: kit::Shape,
        shapes: Vec<Option<kit::Shape>>,
        badge: kit::Badge,
    }

    check_all("root_to_kit", &reflect!(Card, Sighting).unwrap());
}

/// A type in namespace `a` holding enums from namespace `b`.
///
/// C# qualifies the helper class the same way as the type, which is still
/// rooted at the wrong namespace (`Example.A.B`, #149).
#[test]
fn a_to_b() {
    #[derive(Facet)]
    #[repr(C)]
    #[facet(fg::namespace = "b")]
    #[allow(unused)]
    enum Status {
        Up,
        Down,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[facet(fg::namespace = "b")]
    #[allow(unused)]
    enum Signal {
        Level(u8),
        Silent,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "a")]
    struct Row {
        status: Status,
        signal: Signal,
    }

    #[derive(Facet)]
    struct App {
        row: Row,
    }

    check_all("a_to_b", &reflect!(App).unwrap());
}

/// A type in namespace `kv` holding enums pinned to ROOT.
///
/// TypeScript imports the root module as `Example` and reaches the enums and
/// their functions through it. The enums are serialized as enums in every
/// language, but the references themselves are still broken in the others:
/// #148 (Kotlin), #149 (C#) and #151 (Swift).
#[test]
fn namespace_to_root() {
    #[derive(Facet)]
    #[repr(C)]
    #[facet(fg::namespace)]
    #[allow(unused)]
    enum Level {
        Low,
        High,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[facet(fg::namespace)]
    #[allow(unused)]
    enum Outcome {
        Score(u32),
        Missing,
    }

    #[derive(Facet)]
    #[facet(fg::namespace = "kv")]
    struct Entry {
        level: Level,
        outcome: Outcome,
    }

    #[derive(Facet)]
    struct App {
        entry: Entry,
    }

    check_all("namespace_to_root", &reflect!(App).unwrap());
}

/// Builtin type names shadowed by a declaration in *another* module.
///
/// A C# namespace sees the types of the namespace it is nested in, so the
/// ROOT struct `Unit` shadows the runtime's `Unit` inside `Example.Kit` too.
/// A Swift module sees the types of the modules it imports, so the `kit`
/// struct `Set` shadows `Swift.Set` in the root module. Both modules have to
/// write the builtin fully qualified.
#[test]
fn builtins_shadowed_across_namespaces() {
    mod kit {
        use std::collections::BTreeSet;

        use super::*;

        #[derive(Facet)]
        #[facet(fg::namespace = "kit")]
        pub struct Set {
            pub value: u32,
        }

        #[derive(Facet)]
        #[facet(fg::namespace = "kit")]
        pub struct Tray {
            pub nothing: (),
            pub ids: BTreeSet<u32>,
        }
    }

    #[derive(Facet)]
    struct Unit {
        value: u32,
    }

    // `kit::Tray` is not a field: its `()` makes it not `Hashable` in Swift,
    // and Swift decides a ROOT type's conformance as if every type from
    // another module were `Hashable`, a separate bug.
    #[derive(Facet)]
    struct Shelf {
        set: kit::Set,
        ids: std::collections::BTreeSet<u32>,
        unit: Unit,
    }

    use kit::Tray;
    check_all(
        "builtins_shadowed_across_namespaces",
        &reflect!(Shelf, Tray).unwrap(),
    );
}
