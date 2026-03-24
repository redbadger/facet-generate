#![allow(dead_code, clippy::unsafe_derive_deserialize)]
#![cfg(feature = "swift")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod common;

use crate::common::Tree;

use facet::Facet;
use facet_generate as fg;
use facet_generate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Encoding, SourceInstaller,
        swift::{Installer as SwiftInstaller, SwiftCodeGenerator, normalize_path},
    },
    reflect,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io::Write, process::Command};
use tempfile::{TempDir, tempdir};

// ---------------------------------------------------------------------------
// Swift-compatible test fixture
//
// `common::SerdeData` includes `ComplexMap(BTreeMap<([u32; 2], [u8; 4]), ()>)`
// whose key is a native tuple — valid in Rust but not representable as a
// Swift `Dictionary` key (native tuples do not conform to `Hashable`).
// The types below cover the same surface area without that variant.
// ---------------------------------------------------------------------------

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
#[repr(C)]
#[allow(dead_code)]
pub enum SwiftSerdeData {
    PrimitiveTypes(SwiftPrimitiveTypes),
    OtherTypes(SwiftOtherTypes),
    UnitVariant,
    NewTypeVariant(String),
    TupleVariant(u32, u64),
    StructVariant {
        f0: SwiftUnitStruct,
        f1: SwiftNewTypeStruct,
        f2: SwiftTupleStruct,
        f3: SwiftStruct,
    },
    ListWithMutualRecursion(SwiftList<Box<Self>>),
    TreeWithMutualRecursion(Tree<Box<Self>>),
    TupleArray([u32; 3]),
    UnitVector(Vec<()>),
    SimpleList(SwiftSimpleList),
    EmptyStructVariant {},
}

#[allow(dead_code, clippy::struct_field_names)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct SwiftPrimitiveTypes {
    f_bool: bool,
    f_u8: u8,
    f_u16: u16,
    f_u32: u32,
    f_u64: u64,
    f_u128: u128,
    f_i8: i8,
    f_i16: i16,
    f_i32: i32,
    f_i64: i64,
    f_i128: i128,
    f_f32: Option<f32>,
    f_f64: Option<f64>,
    f_char: Option<char>,
}

#[allow(dead_code, clippy::struct_field_names)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftOtherTypes {
    f_string: String,
    #[facet(fg::bytes)]
    f_bytes: Vec<u8>,
    f_option: Option<SwiftStruct>,
    f_unit: (),
    f_seq: Vec<SwiftStruct>,
    f_opt_seq: Option<Vec<i32>>,
    f_tuple: (u8, u16),
    f_stringmap: BTreeMap<String, u32>,
    f_nested_seq: Vec<Vec<SwiftStruct>>,
}

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftUnitStruct;

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftNewTypeStruct(u64);

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftTupleStruct(u32, u64);

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftStruct {
    x: u32,
    y: u64,
}

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(C)]
#[allow(dead_code)]
pub enum SwiftList<T> {
    Empty,
    Node(T, Box<Self>),
}

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SwiftSimpleList(pub Option<Box<Self>>);

/// Registry used for Swift compilation tests.
///
/// Excludes `ComplexMap(BTreeMap<([u32; 2], [u8; 4]), ()>)` from the common
/// `SerdeData` fixture because native Swift tuples are not `Hashable` and
/// cannot be used as `Dictionary` keys.
fn get_swift_registry() -> Registry {
    reflect!(SwiftSerdeData).unwrap()
}

#[derive(Serialize, Deserialize)]
struct Test {
    a: Vec<u32>,
}

fn test_that_swift_code_compiles_with_config(
    config: &CodeGeneratorConfig,
) -> (TempDir, std::path::PathBuf) {
    test_that_swift_code_compiles_with_config_and_registry(config, &get_swift_registry())
}

fn test_that_swift_code_compiles_with_config_and_registry(
    config: &CodeGeneratorConfig,
    registry: &Registry,
) -> (TempDir, std::path::PathBuf) {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("Sources/Testing")).unwrap_or(());
    let serde_package_path = std::env::current_dir().unwrap().join("runtime/swift");
    let mut file = File::create(dir.path().join("Package.swift")).unwrap();

    if config.has_encoding() {
        write!(
            file,
            r#"// swift-tools-version:6.0

import PackageDescription

let package = Package(
    name: "Testing",
    platforms: [.macOS(.v15)],
    products: [
        .library(
            name: "Testing",
            targets: ["Testing"]),
    ],
    dependencies: [
        .package(name: "Serde", path: "{}"),
    ],
    targets: [
        .target(
            name: "Testing",
            dependencies: ["Serde"]),
    ]
)
"#,
            normalize_path(serde_package_path.to_str().unwrap())
        )
        .unwrap();
    } else {
        write!(
            file,
            r#"// swift-tools-version:6.0

import PackageDescription

let package = Package(
    name: "Testing",
    platforms: [.macOS(.v15)],
    products: [
        .library(
            name: "Testing",
            targets: ["Testing"]),
    ],
    targets: [
        .target(
            name: "Testing",
            dependencies: []),
    ]
)
"#
        )
        .unwrap();
    }

    let source_path = dir.path().join("Sources/Testing/Testing.swift");
    let mut source = File::create(&source_path).unwrap();

    let generator = SwiftCodeGenerator::new(config);
    generator.output(&mut source, registry).unwrap();

    // Disable the index store: it's not needed for compilation checks, and on
    // Windows parallel builds race over index-store .pcm files because Windows
    // enforces mandatory file locks.
    let status = Command::new("swift")
        .current_dir(dir.path())
        .args(["build", "--disable-index-store"])
        .status()
        .unwrap();
    assert!(status.success());

    (dir, source_path)
}

// ---------------------------------------------------------------------------
// Error-case tests: non-Hashable types used as Set elements / Map keys
// ---------------------------------------------------------------------------

#[test]
fn set_of_tuple_errors() {
    #[derive(Facet)]
    struct MyStruct {
        items: std::collections::HashSet<(String, i32)>,
    }

    use facet_generate::generation::swift::Swift;
    use facet_generate::generation::{
        Container,
        indent::{IndentConfig, IndentedWriter},
    };

    let registry = reflect!(MyStruct).unwrap();
    let config = facet_generate::generation::CodeGeneratorConfig::new(String::new())
        .with_encoding(Encoding::Bincode);
    let lang = Swift::new(&config, &registry);
    let mut out = Vec::new();
    let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
    let result: std::io::Result<()> = (|| {
        for container in registry.iter().map(Container::from) {
            use facet_generate::generation::Emitter as _;
            container.write(&mut w, &lang)?;
        }
        Ok(())
    })();
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Hashable"),
        "expected Hashable error, got: {msg}"
    );
}

#[test]
fn map_with_tuple_key_errors() {
    #[derive(Facet)]
    struct MyStruct {
        data: std::collections::HashMap<(String, i32), bool>,
    }

    use facet_generate::generation::swift::Swift;
    use facet_generate::generation::{
        Container,
        indent::{IndentConfig, IndentedWriter},
    };

    let registry = reflect!(MyStruct).unwrap();
    let config = facet_generate::generation::CodeGeneratorConfig::new(String::new())
        .with_encoding(Encoding::Bincode);
    let lang = Swift::new(&config, &registry);
    let mut out = Vec::new();
    let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
    let result: std::io::Result<()> = (|| {
        for container in registry.iter().map(Container::from) {
            use facet_generate::generation::Emitter as _;
            container.write(&mut w, &lang)?;
        }
        Ok(())
    })();
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Hashable"),
        "expected Hashable error, got: {msg}"
    );
}

#[test]
fn test_that_swift_code_compiles() {
    let config = CodeGeneratorConfig::new("Testing".to_string()).with_encoding(Encoding::Bincode);
    test_that_swift_code_compiles_with_config(&config);
}

#[test]
fn test_that_swift_code_compiles_without_serialization() {
    #[derive(Facet)]
    struct Child {
        name: String,
        age: u32,
        tags: Vec<String>,
        nickname: Option<String>,
    }

    #[derive(Facet)]
    #[repr(C)]
    #[allow(unused)]
    enum Parent {
        Single(Child),
        Pair { left: Child, right: Child },
        Empty,
    }

    let registry = reflect!(Parent).unwrap();
    let config = CodeGeneratorConfig::new("Testing".to_string());
    test_that_swift_code_compiles_with_config_and_registry(&config, &registry);
}

#[test]
fn test_that_swift_code_compiles_with_bincode() {
    let config = CodeGeneratorConfig::new("Testing".to_string()).with_encoding(Encoding::Bincode);
    test_that_swift_code_compiles_with_config(&config);
}

// ---------------------------------------------------------------------------
// Conformance compile-and-run tests
//
// Each test generates Swift code with Bincode encoding, adds a `main.swift`
// that exercises the declared conformance (`: Hashable` or `: Equatable`),
// and asserts that `swift run` succeeds — proving both that the conformance
// synthesizes correctly and that it behaves correctly at runtime.
// ---------------------------------------------------------------------------

/// Build and run a Swift package that exercises a generated conformance.
///
/// * Installs the Serde runtime and generated code for `registry` into a
///   temp directory.
/// * Writes `main_swift` as `Sources/main/main.swift`.
/// * Writes a `Package.swift` with `Serde`, `Testing`, and `main` targets.
/// * Asserts that `swift run` exits successfully.
fn assert_swift_conformance_runs(registry: &Registry, main_swift: &str) {
    let dir = tempdir().unwrap();
    let config = CodeGeneratorConfig::new("Testing".to_string()).with_encoding(Encoding::Bincode);

    let mut installer = SwiftInstaller::new(&config.module_name, dir.path());
    installer.install_module(&config, registry).unwrap();
    installer.install_serde_runtime().unwrap();

    std::fs::create_dir_all(dir.path().join("Sources/main")).unwrap();
    let mut main = File::create(dir.path().join("Sources/main/main.swift")).unwrap();
    main.write_all(main_swift.as_bytes()).unwrap();

    // Write a Package.swift that exposes Serde and Testing as library
    // targets and main as an executable.
    let mut pkg = File::create(dir.path().join("Package.swift")).unwrap();
    write!(
        pkg,
        r#"// swift-tools-version:6.0
import PackageDescription

let package = Package(
    name: "Testing",
    platforms: [.macOS(.v15)],
    targets: [
        .target(
            name: "Serde",
            dependencies: []),
        .target(
            name: "Testing",
            dependencies: ["Serde"]),
        .target(
            name: "main",
            dependencies: ["Serde", "Testing"]),
    ]
)
"#
    )
    .unwrap();

    let status = Command::new("swift")
        .current_dir(dir.path())
        .args(["run", "--disable-index-store"])
        .status()
        .unwrap();
    assert!(status.success());
}

/// A struct with only primitive (non-`Void`) fields gets `: Hashable` and can
/// therefore be inserted into a `Set` and used as a `Dictionary` key.
///
/// Note: `Void` (`()`) does **not** conform to `Hashable` or `Equatable` in
/// Swift, so structs with a `unit: ()` field receive no protocol conformance.
/// This test uses only genuinely `Hashable` field types to prove the end-to-end
/// path: field-type analysis → `: Hashable` declaration → Swift synthesis →
/// runtime use in `Set` and `Dictionary`.
#[test]
fn test_hashable_conformance_simple_struct() {
    #[derive(Facet)]
    struct SimpleStruct {
        name: String,
        value: i32,
    }

    let registry = reflect!(SimpleStruct).unwrap();
    assert_swift_conformance_runs(
        &registry,
        r#"
import Serde
import Testing

let a = SimpleStruct(name: "hello", value: 42)
let b = SimpleStruct(name: "hello", value: 42)
let c = SimpleStruct(name: "world", value: 0)

// Hashable: usable as a Set element.
var s: Set<SimpleStruct> = []
s.insert(a)
assert(s.contains(b), "identical SimpleStruct values should be found in Set")
assert(!s.contains(c), "different SimpleStruct value should not be in Set")

// Hashable: usable as a Dictionary key.
let d: [SimpleStruct: String] = [a: "value"]
assert(d[b] == "value", "SimpleStruct should be usable as Dictionary key")
"#,
    );
}

/// A struct whose fields are all `Hashable` except for a `[K: V]` Dictionary
/// field should receive `: Equatable` via **auto-synthesis** — `Dictionary`
/// is `Equatable` but not `Hashable` in Swift.
///
/// Proved by comparing instances with `==` and `!=`.
#[test]
fn test_equatable_auto_synthesis_with_dict_field() {
    #[derive(Facet)]
    struct DictField {
        data: BTreeMap<String, i32>,
    }

    let registry = reflect!(DictField).unwrap();
    assert_swift_conformance_runs(
        &registry,
        r#"
import Serde
import Testing

let a = DictField(data: ["key": 1])
let b = DictField(data: ["key": 1])
let c = DictField(data: ["other": 2])

assert(a == b, "DictField with equal contents should compare equal")
assert(a != c, "DictField with different contents should compare unequal")
"#,
    );
}

/// A struct with a tuple field gets `: Equatable` with a **manual**
/// `static func ==`, because native Swift tuples do not conform to the
/// `Equatable` protocol (though `==` works on them as a built-in operator).
///
/// Proved by comparing instances with `==` and `!=`.
#[test]
fn test_equatable_manual_eq_with_tuple_field() {
    #[derive(Facet)]
    struct TupleField {
        pair: (String, i32),
    }

    let registry = reflect!(TupleField).unwrap();
    assert_swift_conformance_runs(
        &registry,
        r#"
import Serde
import Testing

let a = TupleField(pair: ("hello", 42))
let b = TupleField(pair: ("hello", 42))
let c = TupleField(pair: ("world", 0))

assert(a == b, "TupleField with equal tuple should compare equal")
assert(a != c, "TupleField with different tuple should compare unequal")
"#,
    );
}

/// A struct with a deeply nested `Option<BTreeMap<String, Vec<bool>>>` field
/// gets `: Equatable` via **auto-synthesis**, proving that the Swift compiler
/// can synthesize `Equatable` through arbitrarily nested `Dictionary`
/// containers (`[K: V]: Equatable when K: Equatable, V: Equatable`).
///
/// Proved by comparing `nil`, matching, and differing instances.
#[test]
fn test_equatable_auto_synthesis_with_nested_generics() {
    #[derive(Facet)]
    struct NestedDict {
        data: Option<BTreeMap<String, Vec<bool>>>,
    }

    let registry = reflect!(NestedDict).unwrap();
    assert_swift_conformance_runs(
        &registry,
        r#"
import Serde
import Testing

let empty  = NestedDict(data: nil)
let some1  = NestedDict(data: ["key": [true, false]])
let some2  = NestedDict(data: ["key": [true, false]])
let some3  = NestedDict(data: ["other": []])

assert(empty == empty, "nil nested-dict should equal itself")
assert(some1 == some2, "nested-dicts with equal contents should compare equal")
assert(some1 != some3, "nested-dicts with different contents should compare unequal")
assert(empty != some1, "nil should not equal Some(_)")
"#,
    );
}

#[test]
fn test_swift_code_with_external_definitions() {
    #[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
    #[repr(C)]
    pub enum TestData {
        Tree(#[facet(fg::namespace = "foo")] Tree<Box<SwiftSerdeData>>),
        Data(SwiftSerdeData),
    }

    let registry = reflect!(TestData).unwrap();
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("Testing.swift");
    let mut source = File::create(&source_path).unwrap();

    // Pretend that "Tree" is external.
    let mut definitions = BTreeMap::new();
    definitions.insert("foo".to_string(), vec!["Tree".to_string()]);
    let config =
        CodeGeneratorConfig::new("Testing".to_string()).with_external_definitions(definitions);
    let generator = SwiftCodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();

    // References were updated.
    let content = std::fs::read_to_string(source_path).unwrap();
    assert!(content.contains("Foo.Tree"));
}

#[test]
fn test_that_swift_code_follow_case_convention() {
    let config = CodeGeneratorConfig::new("Testing".to_string()).with_encoding(Encoding::Bincode);

    let (_dir, source_path) = test_that_swift_code_compiles_with_config(&config);
    // Case convention were correctly followed.
    let content = std::fs::read_to_string(source_path).unwrap();

    // Enum variants are `lowerCamelCase`.
    assert!(content.contains(r"case primitiveTypes"));
    assert!(!content.contains(r"case PrimitiveTypes"));
    assert!(!content.contains(r"case primitive_types"));
    // Field names are `lowerCamelCase`.
    assert!(content.contains(r"public var fBool: Bool"));
    assert!(!content.contains(r"public var f_bool: Bool"));
}
