#![cfg(feature = "swift")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod common;

use common::{Choice, Test};
use facet_generate::generation::{
    CodeGeneratorConfig, SourceInstaller, bincode::BincodePlugin, json::JsonPlugin, swift,
};
use std::{fs::File, io::Write as _, path::Path, process::Command};

#[test]
fn test_swift_runtime_autotests() {
    let runtime_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/swift");

    let status = Command::new("swift")
        .current_dir(runtime_path.to_str().unwrap())
        .arg("test")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_swift_bincode_runtime_on_simple_data() {
    let dir = tempfile::tempdir().unwrap();
    let config = CodeGeneratorConfig::new("Testing".to_string());
    let registry = common::get_simple_registry();
    let mut installer =
        swift::Installer::new(&config.module_name, dir.path()).plugin(BincodePlugin);
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();

    let reference = bincode::serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    })
    .unwrap();

    std::fs::create_dir_all(dir.path().join("Sources/main")).unwrap();
    let main_path = dir.path().join("Sources/main/main.swift");
    let mut main = File::create(main_path).unwrap();
    writeln!(
        main,
        r#"
import Serde
import Testing

var input : [UInt8] = [{0}]
let value = try Test.bincodeDeserialize(input: input)

let value2 = Test.init(
    a: [4, 6],
    b: (-3, 5),
    c: Choice.c(x: 7)
)
assert(value == value2, "value != value2")

let output = try value2.bincodeSerialize()
assert(input == output, "input != output")

input += [0]
do {{
    let _ = try Test.bincodeDeserialize(input: input)
    assertionFailure("Was expecting an error")
}}
catch {{}}

do {{
    let input2 : [UInt8] = [0, 1]
    let _ = try Test.bincodeDeserialize(input: input2)
    assertionFailure("Was expecting an error")
}}
catch {{}}
"#,
        reference
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", "),
    )
    .unwrap();

    let mut file = File::create(dir.path().join("Package.swift")).unwrap();
    write!(
        file,
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
            dependencies: ["Serde", "Testing"]
        ),
    ]
)
"#
    )
    .unwrap();

    let status = Command::new("swift")
        .current_dir(dir.path())
        .arg("run")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_swift_bincode_runtime_on_supported_types() {
    let dir = tempfile::tempdir().unwrap();
    let config = CodeGeneratorConfig::new("Testing".to_string());
    let registry = common::get_swift_registry();
    let mut installer =
        swift::Installer::new(&config.module_name, dir.path()).plugin(BincodePlugin);
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();

    std::fs::create_dir_all(dir.path().join("Sources/main")).unwrap();
    let main_path = dir.path().join("Sources/main/main.swift");
    let mut main = File::create(main_path).unwrap();

    let positive_encodings = common::get_swift_positive_samples()
        .iter()
        .map(|bytes| quote_bytes(bytes))
        .collect::<Vec<_>>()
        .join(", ");

    writeln!(
        main,
        r#"
import Serde
import Testing

let positive_inputs : [[UInt8]] = [{positive_encodings}]

for input in positive_inputs {{
    let value = try SwiftSerdeData.bincodeDeserialize(input: input)
    let output = try value.bincodeSerialize()
    assert(input == output, "input != output:\n  \(input)\n  \(output)")

    // Test self-equality by comparing serialized bytes.
    let value2 = try SwiftSerdeData.bincodeDeserialize(input: input)
    let output2 = try value2.bincodeSerialize()
    assert(input == output2, "Two deserializations of same input should re-serialize identically: \(input)")

    // Test simple mutations of the input.
    for i in 0..<min(40, input.count) {{
        var input3 = input
        input3[i] ^= 0x80
        if let value3 = try? SwiftSerdeData.bincodeDeserialize(input: input3) {{
            let output3 = try value3.bincodeSerialize()
            assert(output3 != input, "Modified input should round-trip to different bytes:\n  \(input)\n  \(input3)")
        }}
    }}

}}
"#,
    )
    .unwrap();

    let mut file = File::create(dir.path().join("Package.swift")).unwrap();
    write!(
        file,
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
            dependencies: ["Serde", "Testing"]
        ),
    ]
)
"#
    )
    .unwrap();

    let status = Command::new("swift")
        .current_dir(dir.path())
        .arg("run")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_swift_bincode_runtime_on_uuid_data() {
    let dir = tempfile::tempdir().unwrap();
    let config = CodeGeneratorConfig::new("Testing".to_string());
    let registry = common::get_uuid_registry();
    let mut installer =
        swift::Installer::new(&config.module_name, dir.path()).plugin(BincodePlugin);
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();

    let reference = common::get_uuid_reference_bytes();
    let id_str = common::UUID_ID.to_string();
    let parent_id_str = common::UUID_PARENT_ID.to_string();

    std::fs::create_dir_all(dir.path().join("Sources/main")).unwrap();
    let main_path = dir.path().join("Sources/main/main.swift");
    let mut main = File::create(main_path).unwrap();
    writeln!(
        main,
        r#"
import Foundation
import Serde
import Testing

let input: [UInt8] = [{bytes}]
let value = try UuidData.bincodeDeserialize(input: input)

let expectedId = UUID(uuidString: "{id}")!
let expectedParentId = UUID(uuidString: "{parent_id}")!
assert(value.id == expectedId, "id mismatch: \(value.id)")
assert(value.parentId == expectedParentId, "parentId mismatch: \(String(describing: value.parentId))")

let output = try value.bincodeSerialize()
assert(input == output, "roundtrip failed: \(input) != \(output)")

print("UUID roundtrip: PASSED")
"#,
        bytes = reference.iter().map(|x| format!("{x}")).collect::<Vec<_>>().join(", "),
        id = id_str,
        parent_id = parent_id_str,
    )
    .unwrap();

    let mut file = File::create(dir.path().join("Package.swift")).unwrap();
    write!(
        file,
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
            dependencies: ["Serde", "Testing"]
        ),
    ]
)
"#
    )
    .unwrap();

    let status = Command::new("swift")
        .current_dir(dir.path())
        .arg("run")
        .status()
        .unwrap();
    assert!(status.success());
}

fn quote_bytes(bytes: &[u8]) -> String {
    format!(
        "[{}]",
        bytes
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Round-trips a `kv::Entry`, which holds ROOT types (a struct, unit and data
/// enums, and an enum sharing its name with a `kv` struct), through the `Kv`
/// target, which names them from the root package's target (`Example.Shared`).
///
/// Only `kv` references ROOT: a ROOT type holding `kv::Entry` would make the
/// two targets depend on each other, which the installer rejects.
#[test]
fn test_swift_bincode_runtime_from_namespace_to_root() {
    use common::across_namespaces::to_root::{Level, Outcome, Presence, Shared, kv};

    let dir = tempfile::tempdir().unwrap();
    swift::Installer::new("Example", dir.path())
        .plugin(BincodePlugin)
        .generate(&common::across_namespaces::to_root::get_namespace_registry())
        .unwrap();

    let reference = bincode::serialize(&kv::Entry {
        shared: Shared { id: 7 },
        level: Level::High,
        outcome: Outcome::Score(42),
        status: Presence::Offline,
        local: kv::Presence { since: 9 },
    })
    .unwrap();

    std::fs::create_dir_all(dir.path().join("Sources/main")).unwrap();
    let mut main = File::create(dir.path().join("Sources/main/main.swift")).unwrap();
    writeln!(
        main,
        r#"
import Example
import Kv

let input: [UInt8] = [{bytes}]
let value = try Kv.Entry.bincodeDeserialize(input: input)

let expected = Kv.Entry(
    shared: Example.Shared(id: 7),
    level: Example.Level.high,
    outcome: Example.Outcome.score(42),
    status: Example.Presence.offline,
    local: Kv.Presence(since: 9)
)
assert(value == expected, "value mismatch: \(value)")

let output = try value.bincodeSerialize()
assert(input == output, "roundtrip failed: \(input) != \(output)")

print("Namespace to root roundtrip: PASSED")
"#,
        bytes = reference
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", "),
    )
    .unwrap();

    // The generated manifest, plus an executable target to run.
    let mut file = File::create(dir.path().join("Package.swift")).unwrap();
    write!(
        file,
        r#"// swift-tools-version:6.0

import PackageDescription

let package = Package(
    name: "Example",
    platforms: [.macOS(.v15)],
    targets: [
        .target(
            name: "Serde",
            dependencies: []),
        .target(
            name: "Example",
            dependencies: ["Serde"]),
        .target(
            name: "Kv",
            dependencies: ["Example", "Serde"]),
        .target(
            name: "main",
            dependencies: ["Example", "Kv"]
        ),
    ]
)
"#
    )
    .unwrap();

    let status = Command::new("swift")
        .current_dir(dir.path())
        .arg("run")
        .status()
        .unwrap();
    assert!(status.success());
}

/// `char`s of one to four UTF-8 bytes, in a sequence, an option and a map,
/// which Swift declares as `Character` (#213).
#[allow(clippy::unsafe_derive_deserialize)]
mod char_fixture {
    use std::collections::BTreeMap;

    use facet::Facet;
    use serde::{Deserialize, Serialize};

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct CharData {
        pub ascii: char,
        pub two_bytes: char,
        pub three_bytes: char,
        pub four_bytes: char,
        pub chars: Vec<char>,
        pub maybe_char: Option<char>,
        pub no_char: Option<char>,
        pub by_char: BTreeMap<char, char>,
        pub letter: Letter,
    }

    /// Nothing but a `char`, so that a value that isn't one is rejected by
    /// the `char` and not by what follows it.
    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Letter(pub char);

    /// The value the Swift side also builds, field for field.
    pub fn sample() -> CharData {
        CharData {
            ascii: 'a',
            two_bytes: 'é',
            three_bytes: '€',
            four_bytes: '🦀',
            chars: vec!['z', 'ß', '✓', '😀'],
            maybe_char: Some('🦀'),
            no_char: None,
            by_char: BTreeMap::from([('k', '🦀')]),
            letter: Letter('Ω'),
        }
    }

    /// The same value in Swift, as the generated types spell it, and
    /// `Character`s that are more than one Unicode scalar value: a letter and
    /// a combining accent, a flag, a line break and a skin-toned emoji.
    pub const SWIFT_SAMPLE: &str = r#"
let sample = CharData(
    ascii: "a",
    twoBytes: "é",
    threeBytes: "€",
    fourBytes: "🦀",
    chars: ["z", "ß", "✓", "😀"],
    maybeChar: "🦀",
    noChar: nil,
    byChar: ["k": "🦀"],
    letter: Letter(value: "Ω")
)
let notChars: [Character] = ["e\u{301}", "🇬🇧", "\r\n", "👍🏽"]
"#;
}

/// Round-trips `char`s of one to four UTF-8 bytes between Rust's `bincode`
/// and the generated Swift: Swift decodes Rust's bytes into the value it
/// builds itself and encodes both back into those bytes. A `Character` that
/// isn't exactly one Unicode scalar value isn't serialized, and bytes that
/// aren't the UTF-8 encoding of one aren't deserialized (#213).
#[test]
fn test_swift_bincode_runtime_on_chars() {
    use char_fixture::{CharData, SWIFT_SAMPLE, sample};

    // Rust's `bincode` writes a `char` as its UTF-8 bytes, with no length.
    assert_eq!(bincode::serialize(&'a').unwrap(), b"a");
    assert_eq!(bincode::serialize(&'🦀').unwrap(), "🦀".as_bytes());

    let dir = tempfile::tempdir().unwrap();
    swift::Installer::new("Example", dir.path())
        .plugin(BincodePlugin)
        .generate(&facet_generate::reflect!(CharData).unwrap())
        .unwrap();

    let reference = bincode::serialize(&sample()).unwrap();

    run_swift_main(
        dir.path(),
        &format!(
            r#"
import Serde
import Example
{SWIFT_SAMPLE}
let input: [UInt8] = {input}
let value = try CharData.bincodeDeserialize(input: input)
precondition(value == sample, "decoded mismatch:\n  \(value)\n  \(sample)")
for output in [try value.bincodeSerialize(), try sample.bincodeSerialize()] {{
    precondition(output == input, "roundtrip failed:\n  \(input)\n  \(output)")
}}

for bad in notChars {{
    do {{
        _ = try Letter(value: bad).bincodeSerialize()
        fatalError("serialized a bad char: \(bad.unicodeScalars.map {{ $0.value }})")
    }} catch is SerializationError {{}}
}}

// Bytes that aren't the UTF-8 encoding of one: a continuation byte first, a
// byte UTF-8 never uses, a lead byte followed by something other than a
// continuation or by too few of them, overlong encodings, a surrogate, and a
// code point past U+10FFFF.
let notUtf8: [[UInt8]] = [
    [0x80],
    [0xff],
    [0xc3, 0x41],
    [0xf0, 0x9f],
    [0xc0, 0x80],
    [0xe0, 0x80, 0x80],
    [0xf0, 0x80, 0x80, 0x80],
    [0xed, 0xa0, 0x80],
    [0xf4, 0x90, 0x80, 0x80],
]
for bad in notUtf8 {{
    do {{
        _ = try Letter.bincodeDeserialize(input: bad)
        fatalError("deserialized a bad char: \(bad)")
    }} catch is DeserializationError {{}}
}}

print("Chars roundtrip: PASSED")
"#,
            input = quote_bytes(&reference),
        ),
    );
}

/// Types exercising every shape the JSON plugin encodes, whose JSON is
/// `serde_json`'s: the Swift side must read it and write JSON that reads
/// back to the same Rust value.
#[allow(
    clippy::unsafe_derive_deserialize,
    clippy::struct_field_names,
    clippy::zero_sized_map_values,
    clippy::option_option
)]
mod json_fixture {
    use std::collections::{BTreeMap, BTreeSet};

    use facet::Facet;
    use facet_generate as fg;
    use serde::{Deserialize, Serialize};

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq)]
    pub struct JsonData {
        pub big: Big,
        pub floats: Floats,
        pub text: Text,
        #[facet(fg::bytes)]
        pub bytes: Vec<u8>,
        pub nested: Option<Option<u32>>,
        pub maybe_chars: Vec<Option<char>>,
        pub maps: Maps,
        pub choices: Vec<Choice>,
        pub levels: Vec<Level>,
        pub internal: Vec<Internal>,
        pub adjacent: Vec<Adjacent>,
        pub tree: Tree,
        pub list: List,
        pub renamed: Renamed,
        pub unit: (),
        pub units: Vec<()>,
        pub unit_struct: UnitStruct,
        pub newtype: NewType,
        pub tuple_struct: TupleStruct,
        pub tuple: (u32, char, Option<String>),
        pub nested_tuples: Vec<(i8, (bool, String))>,
        pub tuple_map: BTreeMap<String, (u8, u8)>,
        pub unit_values: BTreeMap<String, ()>,
        pub hollow: Vec<Hollow>,
        pub array: [u16; 3],
        pub set: BTreeSet<u32>,
        pub id: uuid::Uuid,
        pub ids: Vec<uuid::Uuid>,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Big {
        pub u128_max: u128,
        pub i128_min: i128,
        pub i128_max: i128,
        pub u128_small: u128,
        pub i128_negative: i128,
        pub u64_max: u64,
        pub i64_min: i64,
        pub i64_max: i64,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq)]
    pub struct Floats {
        pub tenth: f32,
        pub pi: f64,
        pub whole: f64,
        pub tiny: f64,
        pub negative: f32,
        pub optional: Option<f64>,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Text {
        pub ascii: char,
        pub accented: char,
        pub crab: char,
        pub escaped: String,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Maps {
        pub by_string: BTreeMap<String, u32>,
        pub by_int: BTreeMap<u32, String>,
        pub by_negative: BTreeMap<i64, bool>,
        pub by_big: BTreeMap<u128, u8>,
        pub by_bool: BTreeMap<bool, u8>,
        pub by_char: BTreeMap<char, Vec<Option<char>>>,
        pub by_level: BTreeMap<Level, u8>,
        pub by_uuid: BTreeMap<uuid::Uuid, u8>,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum Choice {
        Unit,
        NewType(String),
        Char(char),
        Tuple(u8, char),
        Struct {
            a: u32,
            #[facet(rename = "bee")]
            #[serde(rename = "bee")]
            b: Option<String>,
        },
        #[facet(rename = "renamed-variant")]
        #[serde(rename = "renamed-variant")]
        Renamed(i16),
        Nested(Box<Choice>),
        Big(u128),
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(C)]
    pub enum Level {
        Low,
        #[facet(rename = "HIGH")]
        #[serde(rename = "HIGH")]
        High,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    #[facet(tag = "type")]
    #[serde(tag = "type")]
    pub enum Internal {
        Unit,
        Wrapped(Point),
        Struct { x: i32, c: char },
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    #[facet(tag = "t", content = "c")]
    #[serde(tag = "t", content = "c")]
    pub enum Adjacent {
        Unit,
        NewType(Option<u8>),
        Tuple(u8, String),
        Struct { name: String },
    }

    /// Recursive through an optional field, so Swift holds it `@Indirect`.
    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Tree {
        pub value: u32,
        pub left: Option<Box<Tree>>,
        pub right: Option<Box<Tree>>,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum List {
        Nil,
        Cons(u32, Box<List>),
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[facet(rename_all = "camelCase")]
    #[serde(rename_all = "camelCase")]
    pub struct Renamed {
        pub snake_case_field: u8,
        #[facet(rename = "explicit")]
        #[serde(rename = "explicit")]
        pub other_field: u8,
        pub r#default: bool,
    }

    /// Holds `()` in a variant, so it is not `Equatable` in Swift.
    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum Hollow {
        Nothing(()),
        Something(u8),
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct UnitStruct;

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct NewType(pub String);

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct TupleStruct(pub u8, pub i16);

    pub const ID: uuid::Uuid = uuid::uuid!("550e8400-e29b-41d4-a716-446655440000");

    fn tree(value: u32, left: Option<Tree>, right: Option<Tree>) -> Tree {
        Tree {
            value,
            left: left.map(Box::new),
            right: right.map(Box::new),
        }
    }

    /// The value the Swift side also builds, field for field.
    #[allow(clippy::too_many_lines)]
    pub fn sample() -> JsonData {
        JsonData {
            big: Big {
                u128_max: u128::MAX,
                i128_min: i128::MIN,
                i128_max: i128::MAX,
                u128_small: 1000,
                i128_negative: -42,
                u64_max: u64::MAX,
                i64_min: i64::MIN,
                i64_max: i64::MAX,
            },
            floats: Floats {
                tenth: 0.1,
                pi: std::f64::consts::PI,
                whole: 3.0,
                tiny: 1e-300,
                negative: -2.5,
                optional: None,
            },
            text: Text {
                ascii: 'a',
                accented: 'é',
                crab: '🦀',
                escaped: "quote \" backslash \\ slash / tab \t newline \n unicode ✓".to_string(),
            },
            bytes: vec![0, 1, 127, 128, 255],
            nested: Some(Some(5)),
            maybe_chars: vec![Some('x'), None],
            maps: Maps {
                by_string: BTreeMap::from([("one".to_string(), 1), ("two".to_string(), 2)]),
                by_int: BTreeMap::from([(1, "one".to_string()), (20, "twenty".to_string())]),
                by_negative: BTreeMap::from([(-5, true), (i64::MAX, false)]),
                by_big: BTreeMap::from([(u128::MAX, 1)]),
                by_bool: BTreeMap::from([(true, 1), (false, 0)]),
                by_char: BTreeMap::from([('k', vec![Some('v'), None])]),
                by_level: BTreeMap::from([(Level::Low, 1), (Level::High, 2)]),
                by_uuid: BTreeMap::from([(ID, 7)]),
            },
            choices: vec![
                Choice::Unit,
                Choice::NewType("new".to_string()),
                Choice::Char('c'),
                Choice::Tuple(1, 't'),
                Choice::Struct {
                    a: 1,
                    b: Some("b".to_string()),
                },
                Choice::Struct { a: 2, b: None },
                Choice::Renamed(-1),
                Choice::Nested(Box::new(Choice::Nested(Box::new(Choice::Unit)))),
                Choice::Big(u128::MAX),
            ],
            levels: vec![Level::Low, Level::High],
            internal: vec![
                Internal::Unit,
                Internal::Wrapped(Point { x: 1, y: -1 }),
                Internal::Struct { x: 3, c: 'z' },
            ],
            adjacent: vec![
                Adjacent::Unit,
                Adjacent::NewType(Some(9)),
                Adjacent::NewType(None),
                Adjacent::Tuple(4, "four".to_string()),
                Adjacent::Struct {
                    name: "adj".to_string(),
                },
            ],
            tree: tree(
                1,
                Some(tree(2, None, None)),
                Some(tree(3, Some(tree(4, None, None)), None)),
            ),
            list: List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil)))),
            renamed: Renamed {
                snake_case_field: 1,
                other_field: 2,
                r#default: true,
            },
            unit: (),
            units: vec![(), ()],
            unit_struct: UnitStruct,
            newtype: NewType("wrapped".to_string()),
            tuple_struct: TupleStruct(7, -7),
            tuple: (1, 'q', None),
            nested_tuples: vec![(-1, (true, "yes".to_string()))],
            tuple_map: BTreeMap::from([("pair".to_string(), (1, 2))]),
            unit_values: BTreeMap::from([("a".to_string(), ()), ("b".to_string(), ())]),
            hollow: vec![Hollow::Nothing(()), Hollow::Something(3)],
            array: [1, 2, 3],
            set: BTreeSet::from([3, 1, 2]),
            id: ID,
            ids: vec![ID],
        }
    }

    /// The same value in Swift, as the generated types spell it.
    pub const SWIFT_SAMPLE: &str = r#"
func tree(_ value: UInt32, _ left: Tree?, _ right: Tree?) -> Tree {
    return Tree(value: value, left: left, right: right)
}
let id = UUID(uuidString: "550e8400-e29b-41d4-a716-446655440000")!
let sample = JsonData(
    big: Big(
        u128Max: UInt128(high: UInt64.max, low: UInt64.max),
        i128Min: Int128(high: Int64.min, low: 0),
        i128Max: Int128(high: Int64.max, low: UInt64.max),
        u128Small: UInt128(high: 0, low: 1000),
        i128Negative: Int128(high: -1, low: UInt64.max - 41),
        u64Max: UInt64.max,
        i64Min: Int64.min,
        i64Max: Int64.max
    ),
    floats: Floats(tenth: 0.1, pi: Double.pi, whole: 3.0, tiny: 1e-300, negative: -2.5, optional: nil),
    text: Text(ascii: "a", accented: "é", crab: "🦀", escaped: "quote \" backslash \\ slash / tab \t newline \n unicode ✓"),
    bytes: [0, 1, 127, 128, 255],
    nested: .some(.some(5)),
    maybeChars: ["x", nil],
    maps: Maps(
        byString: ["one": 1, "two": 2],
        byInt: [1: "one", 20: "twenty"],
        byNegative: [-5: true, Int64.max: false],
        byBig: [UInt128(high: UInt64.max, low: UInt64.max): 1],
        byBool: [true: 1, false: 0],
        byChar: ["k": ["v", nil]],
        byLevel: [.low: 1, .high: 2],
        byUuid: [id: 7]
    ),
    choices: [
        .unit,
        .newType("new"),
        .char("c"),
        .tuple(1, "t"),
        .struct(a: 1, bee: "b"),
        .struct(a: 2, bee: nil),
        .renamedVariant(-1),
        .nested(.nested(.unit)),
        .big(UInt128(high: UInt64.max, low: UInt64.max)),
    ],
    levels: [.low, .high],
    internal: [.unit, .wrapped(Point(x: 1, y: -1)), .struct(x: 3, c: "z")],
    adjacent: [.unit, .newType(9), .newType(nil), .tuple(4, "four"), .struct(name: "adj")],
    tree: tree(1, tree(2, nil, nil), tree(3, tree(4, nil, nil), nil)),
    list: .cons(1, .cons(2, .nil)),
    renamed: Renamed(snakeCaseField: 1, explicit: 2, default: true),
    unit: (),
    units: [(), ()],
    unitStruct: UnitStruct(),
    newtype: NewType(value: "wrapped"),
    tupleStruct: TupleStruct(field0: 7, field1: -7),
    tuple: (1, "q", nil),
    nestedTuples: [(-1, (true, "yes"))],
    tupleMap: ["pair": (1, 2)],
    unitValues: ["a": (), "b": ()],
    hollow: [.nothing(()), .something(3)],
    array: [1, 2, 3],
    set: [3, 1, 2],
    id: id,
    ids: [id]
)
"#;
}

/// Writes `main_swift` as the `main` target of the package the installer
/// generated in `dir`, runs it, and returns what it printed on lines starting
/// `JSON:`, without that prefix.
fn run_swift_main(dir: &Path, main_swift: &str) -> Vec<String> {
    std::fs::create_dir_all(dir.join("Sources/main")).unwrap();
    std::fs::write(dir.join("Sources/main/main.swift"), main_swift).unwrap();

    // Add an executable target, depending on every library target, to the
    // generated manifest.
    let manifest_path = dir.join("Package.swift");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let libraries: Vec<String> = std::fs::read_dir(dir.join("Sources"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name != "main")
        .map(|name| format!(r#""{name}""#))
        .collect();
    let main_target = format!(
        "targets: [\n        .executableTarget(name: \"main\", dependencies: [{}]),",
        libraries.join(", ")
    );
    // The package's `targets:` comes after the library product's.
    let at = manifest.rfind("targets: [").unwrap();
    let manifest = format!(
        "{}{main_target}{}",
        &manifest[..at],
        &manifest[at + "targets: [".len()..]
    );
    std::fs::write(&manifest_path, manifest).unwrap();

    let output = Command::new("swift")
        .current_dir(dir)
        .args(["run", "--disable-index-store", "main"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "swift run failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("JSON:").map(str::to_string))
        .collect()
}

/// Round-trips JSON between `serde_json` and the generated Swift, both ways:
/// Swift decodes what Rust wrote into the value it builds itself, and Rust
/// decodes what Swift wrote — re-encoding the decoded value, and encoding the
/// value it built — back into the original.
#[test]
fn test_swift_json_runtime_round_trips_with_serde_json() {
    use json_fixture::{JsonData, SWIFT_SAMPLE, sample};

    let dir = tempfile::tempdir().unwrap();
    swift::Installer::new("Example", dir.path())
        .plugin(JsonPlugin)
        .generate(&facet_generate::reflect!(JsonData).unwrap())
        .unwrap();

    let reference = serde_json::to_vec(&sample()).unwrap();

    let outputs = run_swift_main(
        dir.path(),
        &format!(
            r#"
import Foundation
import Serde
import Example
{SWIFT_SAMPLE}
let input: [UInt8] = {input}
let value = try JsonData.jsonDeserialize(input: input)
// `JsonData` holds `()` and tuples, so it is not `Equatable`: compare its
// fields.
func check(_ same: Bool, _ field: String) {{
    if !same {{ fatalError("decoded \(field) mismatch:\n  \(value)\n  \(sample)") }}
}}
check(value.big == sample.big, "big")
check(value.floats == sample.floats, "floats")
check(value.text == sample.text, "text")
check(value.bytes == sample.bytes, "bytes")
check(value.nested == sample.nested, "nested")
check(value.maybeChars == sample.maybeChars, "maybeChars")
check(value.maps == sample.maps, "maps")
check(value.choices == sample.choices, "choices")
check(value.levels == sample.levels, "levels")
check(value.internal == sample.internal, "internal")
check(value.adjacent == sample.adjacent, "adjacent")
check(value.tree == sample.tree, "tree")
check(value.list == sample.list, "list")
check(value.renamed == sample.renamed, "renamed")
check(value.units.count == 2, "units")
check(value.unitStruct == sample.unitStruct, "unitStruct")
// The registry records `struct EmptyStruct {{}}`, which Rust writes as `{{}}`,
// as a unit struct.
check((try? UnitStruct.jsonDeserialize(input: Array("{{}}".utf8))) != nil, "unit struct from {{}}")
check(value.newtype == sample.newtype, "newtype")
check(value.tupleStruct == sample.tupleStruct, "tupleStruct")
check(value.tuple == sample.tuple, "tuple")
check(value.nestedTuples.count == 1 && value.nestedTuples[0].0 == -1 && value.nestedTuples[0].1 == (true, "yes"), "nestedTuples")
check(value.tupleMap.count == 1 && value.tupleMap["pair"]! == (1, 2), "tupleMap")
check(value.unitValues.keys.sorted() == ["a", "b"], "unitValues")
if case .nothing = value.hollow[0], case .something(3) = value.hollow[1], value.hollow.count == 2 {{}} else {{
    check(false, "hollow")
}}
check(value.array == sample.array, "array")
check(value.set == sample.set, "set")
check(value.id == sample.id && value.ids == sample.ids, "ids")

print("JSON:" + String(decoding: try value.jsonSerialize(), as: UTF8.self))
print("JSON:" + String(decoding: try sample.jsonSerialize(), as: UTF8.self))

// Malformed and truncated input is rejected.
for bad in ["", "{{}}", "[]", String(decoding: input.dropLast(), as: UTF8.self)] {{
    if (try? JsonData.jsonDeserialize(input: Array(bad.utf8))) != nil {{
        fatalError("accepted bad input: \(bad)")
    }}
}}
"#,
            input = quote_bytes(&reference),
        ),
    );

    assert_eq!(outputs.len(), 2, "{outputs:?}");
    for output in &outputs {
        let value: JsonData = serde_json::from_str(output)
            .unwrap_or_else(|e| panic!("Rust could not read Swift's JSON: {e}\n{output}"));
        assert_eq!(value, sample(), "{output}");
    }
}

/// Round-trips `char`s of one to four UTF-8 bytes through JSON between
/// `serde_json` and the generated Swift, both ways, as values and as object
/// keys. A `Character` or JSON string that isn't exactly one Unicode scalar
/// value is neither written nor read, as `serde_json` reads none (#213).
#[test]
fn test_swift_json_runtime_on_chars() {
    use char_fixture::{CharData, SWIFT_SAMPLE, sample};

    let dir = tempfile::tempdir().unwrap();
    swift::Installer::new("Example", dir.path())
        .plugin(JsonPlugin)
        .generate(&facet_generate::reflect!(CharData).unwrap())
        .unwrap();

    let reference = serde_json::to_vec(&sample()).unwrap();
    // `serde_json` rejects a string of two scalar values for a `char`.
    assert!(serde_json::from_str::<char_fixture::Letter>(r#""e\u0301""#).is_err());

    let outputs = run_swift_main(
        dir.path(),
        &format!(
            r#"
import Foundation
import Serde
import Example
{SWIFT_SAMPLE}
let input: [UInt8] = {input}
let value = try CharData.jsonDeserialize(input: input)
precondition(value == sample, "decoded mismatch:\n  \(value)\n  \(sample)")

print("JSON:" + String(decoding: try value.jsonSerialize(), as: UTF8.self))
print("JSON:" + String(decoding: try sample.jsonSerialize(), as: UTF8.self))

for bad in notChars {{
    if (try? Letter(value: bad).jsonSerialize()) != nil {{
        fatalError("wrote a bad char: \(bad.unicodeScalars.map {{ $0.value }})")
    }}
    var badKey = sample
    badKey.byChar = [bad: "v"]
    if (try? badKey.jsonSerialize()) != nil {{
        fatalError("wrote a bad char key: \(bad.unicodeScalars.map {{ $0.value }})")
    }}
}}

// JSON strings that aren't one: none, two, a letter and a combining accent,
// and a flag.
let text = String(decoding: input, as: UTF8.self)
for bad in ["\"\"", "\"ab\"", "\"e\\u0301\"", "\"🇬🇧\""] {{
    if (try? Letter.jsonDeserialize(input: Array(bad.utf8))) != nil {{
        fatalError("read a bad char: \(bad)")
    }}
    let badKey = text.replacingOccurrences(of: "\"k\":", with: bad + ":")
    precondition(badKey != text, "no key to replace")
    if (try? CharData.jsonDeserialize(input: Array(badKey.utf8))) != nil {{
        fatalError("read a bad char key: \(bad)")
    }}
}}
// Nor is anything but a string.
for bad in ["null", "1", "[\"a\"]"] {{
    if (try? Letter.jsonDeserialize(input: Array(bad.utf8))) != nil {{
        fatalError("read a bad char: \(bad)")
    }}
}}
"#,
            input = quote_bytes(&reference),
        ),
    );

    assert_eq!(outputs.len(), 2, "{outputs:?}");
    for output in &outputs {
        let value: CharData = serde_json::from_str(output)
            .unwrap_or_else(|e| panic!("Rust could not read Swift's JSON: {e}\n{output}"));
        assert_eq!(value, sample(), "{output}");
    }
}

/// Round-trips a `kv::Entry`, which holds ROOT types, through JSON between
/// `serde_json` and the `Kv` target, which names them from the root package's
/// target (`Example.Shared`).
#[test]
fn test_swift_json_runtime_from_namespace_to_root() {
    use common::across_namespaces::to_root::{Level, Outcome, Presence, Shared, kv};

    let dir = tempfile::tempdir().unwrap();
    swift::Installer::new("Example", dir.path())
        .plugin(JsonPlugin)
        .generate(&common::across_namespaces::to_root::get_namespace_registry())
        .unwrap();

    let entries = [
        kv::Entry {
            shared: Shared { id: 7 },
            level: Level::High,
            outcome: Outcome::Score(42),
            status: Presence::Offline,
            local: kv::Presence { since: 9 },
        },
        kv::Entry {
            shared: Shared { id: 0 },
            level: Level::Low,
            outcome: Outcome::Missing,
            status: Presence::Online,
            local: kv::Presence { since: u64::MAX },
        },
    ];
    let references: Vec<Vec<u8>> = entries
        .iter()
        .map(|entry| serde_json::to_vec(entry).unwrap())
        .collect();

    let outputs = run_swift_main(
        dir.path(),
        &format!(
            r#"
import Example
import Kv

let inputs: [[UInt8]] = [{inputs}]
let expected = [
    Kv.Entry(
        shared: Example.Shared(id: 7),
        level: Example.Level.high,
        outcome: Example.Outcome.score(42),
        status: Example.Presence.offline,
        local: Kv.Presence(since: 9)
    ),
    Kv.Entry(
        shared: Example.Shared(id: 0),
        level: Example.Level.low,
        outcome: Example.Outcome.missing,
        status: Example.Presence.online,
        local: Kv.Presence(since: UInt64.max)
    ),
]
for (input, expected) in zip(inputs, expected) {{
    let value = try Kv.Entry.jsonDeserialize(input: input)
    assert(value == expected, "value mismatch: \(value)")
    print("JSON:" + String(decoding: try value.jsonSerialize(), as: UTF8.self))
    print("JSON:" + String(decoding: try expected.jsonSerialize(), as: UTF8.self))
}}
"#,
            inputs = references
                .iter()
                .map(|r| quote_bytes(r))
                .collect::<Vec<_>>()
                .join(", "),
        ),
    );

    assert_eq!(outputs.len(), 4, "{outputs:?}");
    for (output, reference) in outputs.iter().zip(references.iter().flat_map(|r| [r, r])) {
        let actual: serde_json::Value = serde_json::from_str(output).unwrap();
        let expected: serde_json::Value = serde_json::from_slice(reference).unwrap();
        assert_eq!(actual, expected, "{output}");
    }
}
