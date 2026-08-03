#![cfg(feature = "typescript")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
pub mod common;

use common::{Choice, Test};
use facet_generate::generation::{CodeGeneratorConfig, bincode::BincodePlugin, typescript};
use std::{fs::File, io::Write, process::Command, sync::Arc};
use tempfile::tempdir;

#[test]
fn test_typescript_runtime_bincode_uuid_roundtrip() {
    let registry = common::get_uuid_registry();
    let dir = tempdir().unwrap();
    let dir_path = dir.path();
    std::fs::create_dir_all(dir_path).unwrap();

    let mut installer = typescript::Installer::new("main", dir_path);
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();

    let source_path = dir_path.join("test.ts");
    let mut source = File::create(&source_path).unwrap();

    writeln!(
        source,
        r#"import {{ assertEquals }} from "https://deno.land/std@0.110.0/testing/asserts.ts";
import {{ BincodeDeserializer, BincodeSerializer }} from "./bincode/index.ts";
"#
    )
    .unwrap();

    let config = CodeGeneratorConfig::new("main".to_string());
    let generator = typescript::TypeScriptCodeGenerator::new(&config)
        .with_plugins(vec![Arc::new(BincodePlugin)]);
    generator.output(&mut source, &registry).unwrap();

    let reference = common::get_uuid_reference_bytes();
    let id_str = common::UUID_ID.to_string();
    let parent_id_str = common::UUID_PARENT_ID.to_string();

    writeln!(
        source,
        r#"
Deno.test("UUID bincode roundtrip", () => {{
  const expectedBytes = new Uint8Array([{bytes}]);
  const deserializer = new BincodeDeserializer(expectedBytes);
  const value: UuidData = UuidData.deserialize(deserializer);

  assertEquals(value.id, "{id}" as Uuid, "id should match");
  assertEquals(value.parent_id, "{parent_id}" as Uuid, "parent_id should match");

  const serializer = new BincodeSerializer();
  value.serialize(serializer);
  const output = serializer.getBytes();

  assertEquals(output, expectedBytes, "roundtrip bytes should match");
}});
"#,
        bytes = reference
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", "),
        id = id_str,
        parent_id = parent_id_str,
    )
    .unwrap();

    let status = Command::new("deno")
        .current_dir(dir_path)
        .arg("test")
        .arg("--sloppy-imports")
        .arg(&source_path)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_typescript_runtime_bincode_serialization() {
    let registry = common::get_simple_registry();
    let dir = tempdir().unwrap();
    let dir_path = dir.path();
    std::fs::create_dir_all(dir_path).unwrap();

    let mut installer = typescript::Installer::new("main", dir_path);
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();

    let source_path = dir_path.join("test.ts");
    let mut source = File::create(&source_path).unwrap();

    writeln!(
        source,
        r#"import {{ assertEquals }} from "https://deno.land/std@0.110.0/testing/asserts.ts";
import {{ BincodeDeserializer, BincodeSerializer }} from "./bincode/index.ts";
"#
    )
    .unwrap();

    let config = CodeGeneratorConfig::new("main".to_string());
    let generator = typescript::TypeScriptCodeGenerator::new(&config)
        .with_plugins(vec![Arc::new(BincodePlugin)]);
    generator.output(&mut source, &registry).unwrap();

    let reference = bincode::serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    })
    .unwrap();

    writeln!(
        source,
        r#"
Deno.test("bincode serialization matches deserialization", () => {{
  const expectedBytes = new Uint8Array([{0}]);
  const deserializer = new BincodeDeserializer(expectedBytes);
  const deserializedInstance: Test = Test.deserialize(deserializer);

  const expectedInstance: Test = new Test(
    [4, 6],
    [BigInt(-3), BigInt(5)],
    new ChoiceVariantC(7),
  );

  assertEquals(deserializedInstance, expectedInstance, "Object instances should match");

  const serializer = new BincodeSerializer();
  expectedInstance.serialize(serializer);
  const serializedBytes = serializer.getBytes();

  assertEquals(serializedBytes, expectedBytes, "bincode bytes should match");
}});
"#,
        reference
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", "),
    )
    .unwrap();

    let status = Command::new("deno")
        .current_dir(dir_path)
        .arg("test")
        .arg("--sloppy-imports")
        .arg(&source_path)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_typescript_runtime_i64_i128_low_limb_high_bit_roundtrip() {
    const LARGE_I64: i64 = 1_785_688_513_662;
    // Low limb (bits 0-63) has bit 63 set; the old signed-OR combine dropped
    // the high limb the same way it did for i64.
    const LARGE_I128: i128 = (1 << 64) | 0xF8C4_E09E_F8C4_E09E_u64 as i128;
    const NEGATIVE_I128: i128 = i128::MIN + 5;

    let registry = common::get_simple_registry();
    let dir = tempdir().unwrap();
    let dir_path = dir.path();
    std::fs::create_dir_all(dir_path).unwrap();

    let mut installer = typescript::Installer::new("main", dir_path);
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();

    let source_path = dir_path.join("test.ts");
    let mut source = File::create(&source_path).unwrap();

    writeln!(
        source,
        r#"import {{ assertEquals }} from "https://deno.land/std@0.110.0/testing/asserts.ts";
import {{ BincodeDeserializer, BincodeSerializer }} from "./bincode/index.ts";
"#
    )
    .unwrap();

    let config = CodeGeneratorConfig::new("main".to_string());
    let generator = typescript::TypeScriptCodeGenerator::new(&config)
        .with_plugins(vec![Arc::new(BincodePlugin)]);
    generator.output(&mut source, &registry).unwrap();

    let reference = bincode::serialize(&Test {
        a: vec![1],
        b: (LARGE_I64, 9),
        c: Choice::A,
    })
    .unwrap();

    writeln!(
        source,
        r#"
Deno.test("i64 with low-half bit 31 set round-trips", () => {{
  const expectedBytes = new Uint8Array([{bytes}]);
  const deserializer = new BincodeDeserializer(expectedBytes);
  const value: Test = Test.deserialize(deserializer);

  assertEquals(value.b[0], BigInt("{large}"), "i64 must keep high limb");
  assertEquals(value.b[1], BigInt(9));

  const serializer = new BincodeSerializer();
  value.serialize(serializer);
  assertEquals(serializer.getBytes(), expectedBytes, "bytes must round-trip");
}});
"#,
        bytes = reference
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", "),
        large = LARGE_I64,
    )
    .unwrap();

    let i128_reference = bincode::serialize(&(LARGE_I128, NEGATIVE_I128)).unwrap();

    writeln!(
        source,
        r#"
Deno.test("i128 with low-limb bit 63 set round-trips", () => {{
  const expectedBytes = new Uint8Array([{bytes}]);
  const deserializer = new BincodeDeserializer(expectedBytes);
  const large = deserializer.deserializeI128();
  const negative = deserializer.deserializeI128();

  assertEquals(large, BigInt("{large}"), "i128 must keep high limb");
  assertEquals(negative, BigInt("{negative}"), "negative i128 must round-trip");

  const serializer = new BincodeSerializer();
  serializer.serializeI128(large);
  serializer.serializeI128(negative);
  assertEquals(serializer.getBytes(), expectedBytes, "bytes must round-trip");
}});
"#,
        bytes = i128_reference
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", "),
        large = LARGE_I128,
        negative = NEGATIVE_I128,
    )
    .unwrap();

    let status = Command::new("deno")
        .current_dir(dir_path)
        .arg("test")
        .arg("--sloppy-imports")
        .arg(&source_path)
        .status()
        .unwrap();
    assert!(status.success());
}
