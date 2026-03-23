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
        CodeGeneratorConfig, Encoding,
        swift::{SwiftCodeGenerator, normalize_path},
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
    ListWithMutualRecursion(SwiftList<Box<SwiftSerdeData>>),
    TreeWithMutualRecursion(Tree<Box<SwiftSerdeData>>),
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
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
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
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct SwiftUnitStruct;

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct SwiftNewTypeStruct(u64);

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct SwiftTupleStruct(u32, u64);

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct SwiftStruct {
    x: u32,
    y: u64,
}

#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
#[repr(C)]
#[allow(dead_code)]
pub enum SwiftList<T> {
    Empty,
    Node(T, Box<SwiftList<T>>),
}

#[allow(dead_code)]
#[derive(Facet, Debug, Serialize, Deserialize, PartialEq)]
pub struct SwiftSimpleList(pub Option<Box<SwiftSimpleList>>);

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
            r#"// swift-tools-version:5.3

import PackageDescription

let package = Package(
    name: "Testing",
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
            r#"// swift-tools-version:5.3

import PackageDescription

let package = Package(
    name: "Testing",
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
    let lang = Swift::new(Encoding::Bincode);
    let mut out = Vec::new();
    let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
    let result: std::io::Result<()> = (|| {
        for container in registry
            .iter()
            .map(|pair| Container::from(pair).with_registry(&registry))
        {
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
    let lang = Swift::new(Encoding::Bincode);
    let mut out = Vec::new();
    let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
    let result: std::io::Result<()> = (|| {
        for container in registry
            .iter()
            .map(|pair| Container::from(pair).with_registry(&registry))
        {
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
