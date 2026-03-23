#![cfg(feature = "swift")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod common;

use common::{Choice, Test};
use facet_generate::generation::{CodeGeneratorConfig, Encoding, SourceInstaller, swift};
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
    let config = CodeGeneratorConfig::new("Testing".to_string()).with_encoding(Encoding::Bincode);
    let registry = common::get_simple_registry();
    let mut installer = swift::Installer::new(&config.module_name, dir.path());
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
    let config = CodeGeneratorConfig::new("Testing".to_string()).with_encoding(Encoding::Bincode);
    let registry = common::get_swift_registry();
    let mut installer = swift::Installer::new(&config.module_name, dir.path());
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
