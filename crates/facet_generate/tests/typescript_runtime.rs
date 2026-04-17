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
