#![cfg(feature = "typescript")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
pub mod common;

use common::{Choice, Test};
use facet_generate::{
    Registry,
    generation::{CodeGeneratorConfig, bincode::BincodePlugin, typescript},
};
use std::{fs::File, io::Write, path::PathBuf, process::Command, sync::Arc};
use tempfile::{TempDir, tempdir};

/// A throwaway TypeScript project with the serde and bincode runtimes
/// installed, and a `test.ts` primed with the imports plus the types generated
/// from `registry`.
///
/// Append `Deno.test` blocks with [`TsProject::write_test`], then execute them
/// with [`TsProject::run`].
struct TsProject {
    dir: TempDir,
    source_path: PathBuf,
    source: File,
}

impl TsProject {
    fn new(registry: &Registry) -> Self {
        let dir = tempdir().unwrap();

        let mut installer = typescript::Installer::new("main", dir.path());
        installer.install_serde_runtime().unwrap();
        installer.install_bincode_runtime().unwrap();

        let source_path = dir.path().join("test.ts");
        let mut source = File::create(&source_path).unwrap();

        writeln!(
            source,
            r#"import {{ assertEquals, assertThrows }} from "https://deno.land/std@0.110.0/testing/asserts.ts";
import {{ BincodeDeserializer, BincodeSerializer }} from "./bincode/index.ts";
"#
        )
        .unwrap();

        let config = CodeGeneratorConfig::new("main".to_string());
        let generator = typescript::TypeScriptCodeGenerator::new(&config)
            .with_plugins(vec![Arc::new(BincodePlugin)]);
        generator.output(&mut source, registry).unwrap();

        Self {
            dir,
            source_path,
            source,
        }
    }

    fn write_test(&mut self, body: &str) {
        writeln!(self.source, "{body}").unwrap();
    }

    fn run(self) {
        drop(self.source);

        let status = Command::new("deno")
            .current_dir(self.dir.path())
            .arg("test")
            .arg("--sloppy-imports")
            .arg(&self.source_path)
            .status()
            .unwrap();
        assert!(status.success());
    }
}

/// Pairs each value with its bincode encoding, for [`scalar_roundtrip_test`].
macro_rules! wire_cases {
    ($($value:expr),* $(,)?) => {
        vec![$((($value).to_string(), bincode::serialize(&$value).unwrap())),*]
    };
}

/// Builds a Deno test that round-trips every `(value, bytes)` pair through the
/// runtime's `deserialize{method}` and `serialize{method}`.
///
/// The expected bytes come from bincode on the Rust side, so the TypeScript
/// runtime is checked against the wire format rather than against itself.
fn scalar_roundtrip_test(test_name: &str, method: &str, cases: &[(String, Vec<u8>)]) -> String {
    let rows = cases
        .iter()
        .map(|(value, bytes)| {
            let bytes = bytes
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("    {{ value: BigInt(\"{value}\"), bytes: new Uint8Array([{bytes}]) }},")
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"
Deno.test("{test_name}", () => {{
  const cases = [
{rows}
  ];

  for (const {{ value, bytes }} of cases) {{
    const deserializer = new BincodeDeserializer(bytes);
    assertEquals(
      deserializer.deserialize{method}(),
      value,
      `deserialize{method}(${{value}})`,
    );

    const serializer = new BincodeSerializer();
    serializer.serialize{method}(value);
    assertEquals(
      serializer.getBytes(),
      bytes,
      `serialize{method}(${{value}})`,
    );
  }}
}});"#
    )
}

#[test]
fn test_typescript_runtime_bincode_uuid_roundtrip() {
    let mut project = TsProject::new(&common::get_uuid_registry());

    let reference = common::get_uuid_reference_bytes();

    project.write_test(&format!(
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
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        id = common::UUID_ID,
        parent_id = common::UUID_PARENT_ID,
    ));

    project.run();
}

#[test]
fn test_typescript_runtime_bincode_serialization() {
    let mut project = TsProject::new(&common::get_simple_registry());

    let reference = bincode::serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    })
    .unwrap();

    project.write_test(&format!(
        r#"
Deno.test("bincode serialization matches deserialization", () => {{
  const expectedBytes = new Uint8Array([{bytes}]);
  const deserializer = new BincodeDeserializer(expectedBytes);
  const deserializedInstance: Test = Test.deserialize(deserializer);

  const expectedChoice: Choice = choiceC(7);
  const expectedInstance: Test = new Test(
    [4, 6],
    [BigInt(-3), BigInt(5)],
    expectedChoice,
  );

  assertEquals(deserializedInstance, expectedInstance, "Object instances should match");

  const serializer = new BincodeSerializer();
  expectedInstance.serialize(serializer);
  const serializedBytes = serializer.getBytes();

  assertEquals(serializedBytes, expectedBytes, "bincode bytes should match");
}});
"#,
        bytes = reference
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", "),
    ));

    project.run();
}

#[test]
fn test_typescript_runtime_i64_i128_low_limb_high_bit_roundtrip() {
    const LARGE_I64: i64 = 1_785_688_513_662;
    // Low limb (bits 0-63) has bit 63 set; the old signed-OR combine dropped
    // the high limb the same way it did for i64.
    const LARGE_I128: i128 = (1 << 64) | 0xF8C4_E09E_F8C4_E09E_u64 as i128;
    const NEGATIVE_I128: i128 = i128::MIN + 5;

    let mut project = TsProject::new(&common::get_simple_registry());

    let reference = bincode::serialize(&Test {
        a: vec![1],
        b: (LARGE_I64, 9),
        c: Choice::A,
    })
    .unwrap();

    project.write_test(&format!(
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
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        large = LARGE_I64,
    ));

    let i128_reference = bincode::serialize(&(LARGE_I128, NEGATIVE_I128)).unwrap();

    project.write_test(&format!(
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
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        large = LARGE_I128,
        negative = NEGATIVE_I128,
    ));

    project.run();
}

/// Exercises the boundaries of every multi-limb integer the TypeScript runtime
/// handles: zero, ±1, the type extremes, and values that straddle the 32- and
/// 64-bit limb boundaries where sign extension used to corrupt the result.
#[test]
fn test_typescript_runtime_integer_edge_cases_roundtrip() {
    let mut project = TsProject::new(&common::get_simple_registry());

    let i64_cases = wire_cases![
        0_i64,
        1_i64,
        -1_i64,
        i64::MIN,
        i64::MAX,
        1_i64 << 31,
        -(1_i64 << 31),
        1_785_688_513_662_i64,
        -1_785_688_513_662_i64,
    ];
    project.write_test(&scalar_roundtrip_test(
        "i64 edge cases round-trip",
        "I64",
        &i64_cases,
    ));

    let u64_cases = wire_cases![
        0_u64,
        1_u64,
        u64::MAX,
        1_u64 << 31,
        (1_u64 << 32) - 1,
        1_u64 << 32,
        0xFFFF_FFFF_8000_0000_u64,
    ];
    project.write_test(&scalar_roundtrip_test(
        "u64 edge cases round-trip",
        "U64",
        &u64_cases,
    ));

    let i128_cases = wire_cases![
        0_i128,
        1_i128,
        -1_i128,
        i128::MIN,
        i128::MAX,
        i128::MIN + 5,
        1_i128 << 63,
        -(1_i128 << 64),
    ];
    project.write_test(&scalar_roundtrip_test(
        "i128 edge cases round-trip",
        "I128",
        &i128_cases,
    ));

    let u128_cases = wire_cases![
        0_u128,
        1_u128,
        u128::MAX,
        1_u128 << 63,
        1_u128 << 64,
        u128::from(u64::MAX),
        (1_u128 << 64) | 0xF8C4_E09E_F8C4_E09E_u128,
    ];
    project.write_test(&scalar_roundtrip_test(
        "u128 edge cases round-trip",
        "U128",
        &u128_cases,
    ));

    project.run();
}

/// Truncated input must fail loudly. `read()` used to hand back whatever the
/// buffer had left, so a short fixed-width field surfaced as an opaque
/// `RangeError` from `DataView` and a short length-prefixed field was silently
/// deserialized as a shorter value.
#[test]
fn test_typescript_runtime_truncated_input_throws() {
    let mut project = TsProject::new(&common::get_simple_registry());

    project.write_test(
        r#"
Deno.test("truncated input throws instead of yielding a short value", () => {
  // an i64 needs eight bytes; only three are available
  assertThrows(
    () => new BincodeDeserializer(new Uint8Array([1, 2, 3])).deserializeI64(),
    Error,
    "Unexpected end of input",
  );

  // a length prefix of five, with only two bytes of payload behind it
  const truncatedStr = new Uint8Array([5, 0, 0, 0, 0, 0, 0, 0, 0x68, 0x69]);
  assertThrows(
    () => new BincodeDeserializer(truncatedStr).deserializeStr(),
    Error,
    "Unexpected end of input",
  );

  // nothing at all to read
  assertThrows(
    () => new BincodeDeserializer(new Uint8Array([])).deserializeBool(),
    Error,
    "Unexpected end of input",
  );

  // reading exactly to the end must still succeed
  const exact = new BincodeDeserializer(
    new Uint8Array([9, 0, 0, 0, 0, 0, 0, 0]),
  );
  assertEquals(exact.deserializeI64(), BigInt(9));
});
"#,
    );

    project.run();
}
