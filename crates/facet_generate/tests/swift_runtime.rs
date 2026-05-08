#![cfg(feature = "swift")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod common;

use common::{Choice, Test};
use facet_generate::generation::{CodeGeneratorConfig, bincode::BincodePlugin, swift};
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

// ---------------------------------------------------------------------------
// Helper: write the consumer app/Package.swift that imports a lib package
// ---------------------------------------------------------------------------

/// Write the consumer `app/Package.swift`.
///
/// SPM uses the **directory name** of a local path dependency as its package
/// identity (not the `name` field declared inside `Package.swift`).  The lib
/// directory is always named `"lib"`, so `package: "lib"` is the correct
/// identifier.  The *product* name (`lib_product_name`) is the name declared
/// in the library product of `lib/Package.swift`.
fn write_consumer_package_swift(app_dir: &Path, lib_product_name: &str) {
    let contents = format!(
        r#"// swift-tools-version:6.0

import PackageDescription

let package = Package(
    name: "TestApp",
    platforms: [.macOS(.v15)],
    dependencies: [
        .package(path: "../lib"),
    ],
    targets: [
        .executableTarget(
            name: "main",
            dependencies: [
                .product(name: "{lib_product_name}", package: "lib"),
            ]
        ),
    ]
)
"#
    );
    std::fs::write(app_dir.join("Package.swift"), contents).unwrap();
}

// ---------------------------------------------------------------------------
// Bincode runtime tests (refactored to use installer.generate())
// ---------------------------------------------------------------------------

#[test]
fn test_swift_bincode_runtime_on_simple_data() {
    let dir = tempfile::tempdir().unwrap();
    let lib_dir = dir.path().join("lib");
    let app_dir = dir.path().join("app");

    let config = CodeGeneratorConfig::new("Testing".to_string());
    let registry = common::get_simple_registry();
    swift::Installer::new(&config.module_name, &lib_dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let reference = bincode::serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    })
    .unwrap();

    std::fs::create_dir_all(app_dir.join("Sources/main")).unwrap();

    let mut main = File::create(app_dir.join("Sources/main/main.swift")).unwrap();
    writeln!(
        main,
        r#"
import Testing

var input : [UInt8] = [{bytes}]
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
        bytes = reference
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", "),
    )
    .unwrap();

    write_consumer_package_swift(&app_dir, "Testing");

    let status = Command::new("swift")
        .current_dir(&app_dir)
        .arg("run")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_swift_bincode_runtime_on_supported_types() {
    let dir = tempfile::tempdir().unwrap();
    let lib_dir = dir.path().join("lib");
    let app_dir = dir.path().join("app");

    let config = CodeGeneratorConfig::new("Testing".to_string());
    let registry = common::get_swift_registry();
    swift::Installer::new(&config.module_name, &lib_dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    std::fs::create_dir_all(app_dir.join("Sources/main")).unwrap();

    let positive_encodings = common::get_swift_positive_samples()
        .iter()
        .map(|bytes| quote_bytes(bytes))
        .collect::<Vec<_>>()
        .join(", ");

    let mut main = File::create(app_dir.join("Sources/main/main.swift")).unwrap();
    writeln!(
        main,
        r#"
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

    write_consumer_package_swift(&app_dir, "Testing");

    let status = Command::new("swift")
        .current_dir(&app_dir)
        .arg("run")
        .status()
        .unwrap();
    assert!(status.success());
}

// ---------------------------------------------------------------------------
// Helpers for MessagePack tests
// ---------------------------------------------------------------------------

/// Generate a correct `MessagePack` Swift library package into `lib_dir`.
///
/// Uses `SwiftCodeGenerator` directly instead of `Installer::generate()`,
/// because `install_module` unconditionally adds a `"Serde"` target
/// dependency for any plugin — including `MessagePackPlugin` which has no Serde
/// runtime — producing a broken `Package.swift`.  This helper writes
/// the source and a correct `Package.swift` by hand.
fn generate_msgpack_lib(
    lib_dir: &std::path::Path,
    registry: &facet_generate::Registry,
    package_name: &str,
) {
    use facet_generate::generation::{
        CodeGeneratorConfig,
        messagepack::MessagePackPlugin,
        plugin::EmitterPlugin,
        swift::{Swift as SwiftLang, SwiftCodeGenerator},
    };
    use std::sync::Arc;

    let config = CodeGeneratorConfig::new(package_name.to_string());

    // Write the generated Swift source (with Foundation prepended for Data).
    let sources_dir = lib_dir.join(format!("Sources/{package_name}"));
    std::fs::create_dir_all(&sources_dir).unwrap();
    let mut source = File::create(sources_dir.join(format!("{package_name}.swift"))).unwrap();
    writeln!(source, "import Foundation").unwrap();
    SwiftCodeGenerator::new(&config)
        .with_plugins(vec![
            Arc::new(MessagePackPlugin) as Arc<dyn EmitterPlugin<SwiftLang>>
        ])
        .output(&mut source, registry)
        .unwrap();

    // Write a Package.swift that correctly depends only on MessagePacker.
    let pkg = format!(
        r#"// swift-tools-version: 5.8
import PackageDescription
let package = Package(
    name: "{package_name}",
    products: [
        .library(name: "{package_name}", targets: ["{package_name}"]),
    ],
    dependencies: [
        .package(url: "https://github.com/hirotakan/MessagePacker.git", from: "0.4.7"),
    ],
    targets: [
        .target(
            name: "{package_name}",
            dependencies: ["MessagePacker"]
        ),
    ]
)
"#
    );
    std::fs::write(lib_dir.join("Package.swift"), pkg).unwrap();
}

// ---------------------------------------------------------------------------
// MessagePack runtime tests
//
// MessagePackEncoder uses Swift's Codable synthesis, which keys struct fields
// by their Swift property names (lowerCamelCase: fBool, fU8, …).
// rmp_serde::to_vec_named uses Rust field names (snake_case: f_bool, f_u8, …).
// These do NOT match, so we run a pure Swift round-trip instead of a
// cross-language comparison: construct values in Swift, serialize, deserialize,
// verify field values, re-serialize, compare bytes.
// ---------------------------------------------------------------------------

#[test]
fn test_swift_msgpack_runtime_on_simple_data() {
    use common::MsgPackPrimitiveTypes;
    use facet_generate::reflect;

    let dir = tempfile::tempdir().unwrap();
    let lib_dir = dir.path().join("lib");
    let app_dir = dir.path().join("app");

    let registry = reflect!(MsgPackPrimitiveTypes).unwrap();
    generate_msgpack_lib(&lib_dir, &registry, "Testing");

    std::fs::create_dir_all(app_dir.join("Sources/main")).unwrap();

    let mut main = File::create(app_dir.join("Sources/main/main.swift")).unwrap();
    writeln!(
        main,
        r#"
import Foundation
import Testing

// Construct a value directly in Swift (field names are lowerCamelCase).
let original = MsgPackPrimitiveTypes(
    fBool: false,
    fU8: 6,
    fU16: 5,
    fU32: 4,
    fU64: 3,
    fI8: 1,
    fI16: 0,
    fI32: -1,
    fI64: -2,
    fF32: Float(0.5),
    fF64: Double(-1.25)
)

// Serialize to MessagePack bytes.
let encoded = try original.msgPackSerialize()

// Deserialize back.
let decoded = try MsgPackPrimitiveTypes.msgPackDeserialize(input: encoded)

// Verify every field.
assert(decoded.fBool == false,  "fBool mismatch: \(decoded.fBool)")
assert(decoded.fU8  == 6,       "fU8 mismatch: \(decoded.fU8)")
assert(decoded.fU16 == 5,       "fU16 mismatch: \(decoded.fU16)")
assert(decoded.fU32 == 4,       "fU32 mismatch: \(decoded.fU32)")
assert(decoded.fU64 == 3,       "fU64 mismatch: \(decoded.fU64)")
assert(decoded.fI8  == 1,       "fI8 mismatch: \(decoded.fI8)")
assert(decoded.fI16 == 0,       "fI16 mismatch: \(decoded.fI16)")
assert(decoded.fI32 == -1,      "fI32 mismatch: \(decoded.fI32)")
assert(decoded.fI64 == -2,      "fI64 mismatch: \(decoded.fI64)")
assert(decoded.fF32 == Float(0.5),    "fF32 mismatch: \(decoded.fF32)")
assert(decoded.fF64 == Double(-1.25), "fF64 mismatch: \(decoded.fF64)")

// Re-serialize and verify the bytes are identical (lossless round-trip).
let reEncoded = try decoded.msgPackSerialize()
assert(Array(encoded) == Array(reEncoded), "Round-trip failed: \(Array(encoded)) != \(Array(reEncoded))")

print("MessagePack simple data roundtrip: PASSED")
"#,
    )
    .unwrap();

    write_consumer_package_swift(&app_dir, "Testing");

    let status = Command::new("swift")
        .current_dir(&app_dir)
        .arg("run")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_swift_msgpack_runtime_on_supported_types() {
    use common::MsgPackPrimitiveTypes;
    use facet_generate::reflect;

    let dir = tempfile::tempdir().unwrap();
    let lib_dir = dir.path().join("lib");
    let app_dir = dir.path().join("app");

    let registry = reflect!(MsgPackPrimitiveTypes).unwrap();
    generate_msgpack_lib(&lib_dir, &registry, "Testing");

    std::fs::create_dir_all(app_dir.join("Sources/main")).unwrap();

    let mut main = File::create(app_dir.join("Sources/main/main.swift")).unwrap();
    writeln!(
        main,
        r#"
import Foundation
import Testing

// A set of values that exercises typical, boundary, and zero cases.
let testCases: [MsgPackPrimitiveTypes] = [
    // Typical values (matching get_msgpack_sample_values first entry).
    MsgPackPrimitiveTypes(
        fBool: false, fU8: 6, fU16: 5, fU32: 4, fU64: 3,
        fI8: 1, fI16: 0, fI32: -1, fI64: -2,
        fF32: Float(0.5), fF64: Double(-1.25)
    ),
    // Boundary values — max unsigned, min signed.
    MsgPackPrimitiveTypes(
        fBool: true, fU8: UInt8.max, fU16: UInt16.max, fU32: UInt32.max, fU64: UInt64.max,
        fI8: Int8.min, fI16: Int16.min, fI32: Int32.min, fI64: Int64.min,
        fF32: Float(-1.25), fF64: Double(0.5)
    ),
    // Zero / false defaults.
    MsgPackPrimitiveTypes(
        fBool: false, fU8: 0, fU16: 0, fU32: 0, fU64: 0,
        fI8: 0, fI16: 0, fI32: 0, fI64: 0,
        fF32: Float(0.0), fF64: Double(0.0)
    ),
]

for original in testCases {{
    let encoded   = try original.msgPackSerialize()
    let decoded   = try MsgPackPrimitiveTypes.msgPackDeserialize(input: encoded)
    let reEncoded = try decoded.msgPackSerialize()
    assert(
        Array(encoded) == Array(reEncoded),
        "Round-trip failed for value with fBool=\(original.fBool) fU8=\(original.fU8)"
    )
}}

print("MessagePack supported types roundtrip: PASSED")
"#,
    )
    .unwrap();

    write_consumer_package_swift(&app_dir, "Testing");

    let status = Command::new("swift")
        .current_dir(&app_dir)
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
