#![cfg(feature = "typescript")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
pub mod common;

use common::{Choice, Test};
use facet_generate::generation::{CodeGeneratorConfig, bincode::BincodePlugin, typescript};
use std::{fs::File, io::Write, process::Command, sync::Arc};
use tempfile::tempdir;

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
fn test_typescript_msgpack_runtime_self_roundtrip() {
    use common::MsgPackStruct;
    use facet_generate::generation::messagepack::MessagePackPlugin;
    use facet_generate::reflect;

    let registry = reflect!(MsgPackStruct).unwrap();
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // Generate TypeScript code with MessagePackPlugin
    typescript::Installer::new("testing", dir_path)
        .plugin(MessagePackPlugin)
        .generate(&registry)
        .unwrap();

    // Write deno.json import map so bare npm specifier resolves
    std::fs::write(
        dir_path.join("deno.json"),
        r#"{ "imports": { "@msgpack/msgpack": "npm:@msgpack/msgpack" } }"#,
    )
    .unwrap();

    // Write the Deno test file
    let test_path = dir_path.join("test_msgpack.ts");
    let mut test_file = File::create(&test_path).unwrap();
    writeln!(
        test_file,
        r#"import {{ assertEquals }} from "https://deno.land/std@0.110.0/testing/asserts.ts";
import {{ MsgPackStruct, msgPackEncode, msgPackDecode }} from "./testing.ts";

Deno.test("msgpack self-roundtrip for MsgPackStruct", () => {{
    // x is u32 (uint32 -> number), y is u64 (uint64 -> bigint). The generated
    // helpers pass `useBigInt64: true` to @msgpack/msgpack so BigInt values
    // round-trip as MessagePack int64 / uint64 without throwing.
    const original = new MsgPackStruct(42, 99n);

    // Encode the object as MessagePack bytes
    const encoded: Uint8Array = msgPackEncode(original);

    // Decode back to a plain JS object (not a class instance)
    const decoded = msgPackDecode<MsgPackStruct>(encoded);

    assertEquals(decoded.x, 42, "x should be 42");
    assertEquals(decoded.y, 99n, "y should be 99n");

    // Re-encode the decoded object and verify byte-level identity (structural roundtrip)
    const reEncoded: Uint8Array = msgPackEncode(decoded);
    assertEquals(encoded, reEncoded, "re-encoded bytes should match original encoding");
}});
"#
    )
    .unwrap();

    let status = Command::new("deno")
        .current_dir(dir_path)
        .arg("test")
        .arg("--sloppy-imports")
        // The installer writes a package.json alongside the generated .ts file;
        // Deno 2 sees the package.json and expects a local node_modules directory
        // to be populated via `deno install`.  Setting node-modules-dir=none
        // tells Deno to use its global npm cache instead, so the test works
        // without a prior `deno install` step.
        .arg("--node-modules-dir=none")
        .arg(&test_path)
        .status()
        .unwrap();
    assert!(status.success());
}
