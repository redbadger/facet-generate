#![cfg(feature = "typescript")]
// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
pub mod common;

use common::{Choice, Test};
use facet_generate::{
    Registry,
    generation::{CodeGeneratorConfig, bincode::BincodePlugin, json::JsonPlugin, typescript},
};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
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

/// A field whose wire name isn't an identifier round-trips through bincode,
/// on a struct and on a struct variant (#197).
#[test]
fn test_typescript_runtime_bincode_non_identifier_field_names_roundtrip() {
    use facet::Facet;
    use serde::Serialize;

    #[derive(Facet, Serialize)]
    struct Renamed {
        plain: u8,
        #[facet(rename = "with-dash")]
        #[serde(rename = "with-dash")]
        dashed: u16,
        r#default: bool,
        choice: Choice,
    }

    #[derive(Facet, Serialize)]
    #[repr(C)]
    #[allow(unused)]
    enum Choice {
        Unit,
        Other {
            #[facet(rename = "with-dash")]
            #[serde(rename = "with-dash")]
            dashed: u32,
            r#default: bool,
        },
    }

    let mut project = TsProject::new(&facet_generate::reflect!(Renamed).unwrap());
    let bytes = to_byte_list(
        &bincode::serialize(&Renamed {
            plain: 1,
            dashed: 515,
            r#default: true,
            choice: Choice::Other {
                dashed: 70000,
                r#default: false,
            },
        })
        .unwrap(),
    );

    project.write_test(&format!(
        r#"
Deno.test("non-identifier field names round-trip through bincode", () => {{
  const expectedBytes = new Uint8Array([{bytes}]);
  const expected = new Renamed(1, 515, true, choiceOther(70000, false));

  const actual = Renamed.deserialize(new BincodeDeserializer(expectedBytes));
  assertEquals(actual, expected);
  assertEquals(actual["with-dash"], 515);

  const serializer = new BincodeSerializer();
  actual.serialize(serializer);
  assertEquals(serializer.getBytes(), expectedBytes);
}});
"#
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

/// Round-trips values across a namespaced module and the root one, which
/// import each other: the ROOT `App` holds a `kv::Entry`, which holds ROOT
/// types (a struct, unit and data enums, and an enum sharing its name with a
/// `kv` struct).
///
/// Each test file imports the two modules in a different order, so each one
/// is evaluated first once. Nothing in either runs at load time, only when
/// called, so neither order sees the other half-initialised.
#[test]
fn test_typescript_runtime_bincode_roundtrip_across_root_and_namespace() {
    use common::across_namespaces::to_root::{App, Level, Outcome, Presence, Shared, kv};

    let dir = tempdir().unwrap();
    typescript::Installer::new("example", dir.path())
        .plugin(BincodePlugin)
        .generate(&common::across_namespaces::to_root::get_registry())
        .unwrap();

    let entry = kv::Entry {
        shared: Shared { id: 7 },
        level: Level::High,
        outcome: Outcome::Score(42),
        status: Presence::Offline,
        local: kv::Presence { since: 9 },
    };
    let entry_bytes = to_byte_list(&bincode::serialize(&entry).unwrap());
    let app_bytes = to_byte_list(
        &bincode::serialize(&App {
            entry,
            shared: Shared { id: 3 },
        })
        .unwrap(),
    );

    let body = format!(
        r#"import {{ BincodeDeserializer, BincodeSerializer }} from "./bincode/index.ts";

Deno.test("ROOT and kv round-trip through each other", () => {{
  const expectedEntry = new Kv.Entry(
    new Example.Shared(7),
    Example.levelHigh(),
    Example.outcomeScore(42),
    Example.presenceOffline(),
    new Kv.Presence(BigInt(9)),
  );

  // kv -> ROOT: a namespaced type deserializing ROOT types
  const entryBytes = new Uint8Array([{entry_bytes}]);
  const entry = Kv.Entry.deserialize(new BincodeDeserializer(entryBytes));
  assertEquals(entry, expectedEntry);
  assert(entry.shared instanceof Example.Shared);
  assert(entry.local instanceof Kv.Presence);
  const entrySerializer = new BincodeSerializer();
  entry.serialize(entrySerializer);
  assertEquals(entrySerializer.getBytes(), entryBytes);

  // ROOT -> kv -> ROOT
  const appBytes = new Uint8Array([{app_bytes}]);
  const app = Example.App.deserialize(new BincodeDeserializer(appBytes));
  assertEquals(app, new Example.App(expectedEntry, new Example.Shared(3)));
  assert(app.entry instanceof Kv.Entry);
  const appSerializer = new BincodeSerializer();
  app.serialize(appSerializer);
  assertEquals(appSerializer.getBytes(), appBytes);
}});
"#
    );

    let asserts = r#"import { assert, assertEquals } from "https://deno.land/std@0.110.0/testing/asserts.ts";"#;
    let root_first = dir.path().join("root_first.test.ts");
    std::fs::write(
        &root_first,
        format!(
            "{asserts}\nimport * as Example from \"./example.ts\";\nimport * as Kv from \"./kv.ts\";\n{body}"
        ),
    )
    .unwrap();
    let kv_first = dir.path().join("kv_first.test.ts");
    std::fs::write(
        &kv_first,
        format!(
            "{asserts}\nimport * as Kv from \"./kv.ts\";\nimport * as Example from \"./example.ts\";\n{body}"
        ),
    )
    .unwrap();

    // One process per file, so that each order is the first evaluation of
    // the modules.
    for test_file in [root_first, kv_first] {
        let status = Command::new("deno")
            .current_dir(dir.path())
            .arg("test")
            .arg("--sloppy-imports")
            .arg(&test_file)
            .status()
            .unwrap();
        assert!(
            status.success(),
            "deno test failed for {}",
            test_file.display()
        );
    }
}

fn to_byte_list(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------

/// Types exercising every shape the JSON plugin encodes, whose JSON is
/// `serde_json`'s: the TypeScript side must read it and write JSON that reads
/// back to the same Rust value.
///
/// The Kotlin runtime test's fixture, but for `Option<Option<T>>`, which
/// TypeScript cannot hold (`T | null | null` is `T | null`), and with the
/// enum shadowing a builtin named `Map`, a global the module's map fields
/// use.
#[allow(
    clippy::unsafe_derive_deserialize,
    clippy::struct_field_names,
    clippy::zero_sized_map_values
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
        pub maybe: Option<u32>,
        pub maybe_chars: Vec<Option<char>>,
        pub maps: Maps,
        pub choices: Vec<Choice>,
        pub levels: Vec<Level>,
        pub modes: Vec<Mode>,
        pub internal: Vec<Internal>,
        pub adjacent: Vec<Adjacent>,
        pub tree: Tree,
        pub list: Map,
        pub renamed: Renamed,
        pub unit: (),
        pub units: Vec<()>,
        pub unit_struct: UnitStruct,
        pub newtype: NewType,
        pub tuple_struct: TupleStruct,
        pub pair: (u8, String),
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
        pub by_big: BTreeMap<u128, u8>,
        pub maybe_big: Option<i128>,
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
        #[facet(rename = "$ref")]
        #[serde(rename = "$ref")]
        pub reference: String,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Maps {
        pub by_string: BTreeMap<String, u32>,
        pub by_int: BTreeMap<u32, String>,
        pub by_negative: BTreeMap<i64, bool>,
        pub by_bool: BTreeMap<bool, u8>,
        pub by_char: BTreeMap<char, Vec<Option<char>>>,
        pub by_level: BTreeMap<Level, u8>,
        pub by_uuid: BTreeMap<uuid::Uuid, u8>,
        pub by_newtype: BTreeMap<NewType, u8>,
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
        #[facet(rename = "Other")]
        #[serde(rename = "Other")]
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

    /// Unit variants only, but internally tagged, so not a bare string.
    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    #[facet(tag = "kind")]
    #[serde(tag = "kind")]
    pub enum Mode {
        Fast,
        Slow,
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

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Tree {
        pub value: u32,
        pub left: Option<Box<Tree>>,
        pub right: Option<Box<Tree>>,
    }

    /// Shadows TypeScript's `Map`, which the module's map fields are.
    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum Map {
        Nil,
        Cons(u32, Box<Map>),
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[facet(rename_all = "camelCase")]
    #[serde(rename_all = "camelCase")]
    pub struct Renamed {
        pub snake_case_field: u8,
        #[facet(rename = "explicit")]
        #[serde(rename = "explicit")]
        pub other_field: u8,
        #[facet(rename = "with-dash")]
        #[serde(rename = "with-dash")]
        pub dashed: u8,
        pub r#default: bool,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum Hollow {
        Empty(()),
        Something(u8),
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct UnitStruct;

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

    /// The value the TypeScript side also builds, field for field.
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
                by_big: BTreeMap::from([(u128::MAX, 1)]),
                maybe_big: Some(i128::MIN),
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
                escaped: "quote \" backslash \\ slash / tab \t newline \n dollar $x unicode ✓"
                    .to_string(),
                reference: "#/defs".to_string(),
            },
            bytes: vec![0, 1, 127, 128, 255],
            maybe: Some(5),
            maybe_chars: vec![Some('x'), None],
            maps: Maps {
                by_string: BTreeMap::from([("one".to_string(), 1), ("two".to_string(), 2)]),
                by_int: BTreeMap::from([(1, "one".to_string()), (20, "twenty".to_string())]),
                by_negative: BTreeMap::from([(-5, true), (i64::MAX, false)]),
                by_bool: BTreeMap::from([(false, 0), (true, 1)]),
                by_char: BTreeMap::from([('k', vec![Some('v'), None])]),
                by_level: BTreeMap::from([(Level::Low, 1), (Level::High, 2)]),
                by_uuid: BTreeMap::from([(ID, 7)]),
                by_newtype: BTreeMap::from([(NewType("key".to_string()), 3)]),
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
            modes: vec![Mode::Fast, Mode::Slow],
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
            list: Map::Cons(1, Box::new(Map::Cons(2, Box::new(Map::Nil)))),
            renamed: Renamed {
                snake_case_field: 1,
                other_field: 2,
                dashed: 3,
                r#default: true,
            },
            unit: (),
            units: vec![(), ()],
            unit_struct: UnitStruct,
            newtype: NewType("wrapped".to_string()),
            tuple_struct: TupleStruct(7, -7),
            pair: (8, "eight".to_string()),
            tuple: (1, 'q', None),
            nested_tuples: vec![(-1, (true, "yes".to_string()))],
            tuple_map: BTreeMap::from([("pair".to_string(), (1, 2))]),
            unit_values: BTreeMap::from([("a".to_string(), ()), ("b".to_string(), ())]),
            hollow: vec![Hollow::Empty(()), Hollow::Something(3)],
            array: [1, 2, 3],
            set: BTreeSet::from([1, 2, 3]),
            id: ID,
            ids: vec![ID],
        }
    }

    /// The same value in TypeScript, as the generated types spell it.
    pub const TYPESCRIPT_SAMPLE: &str = r##"
const tree = (value: number, left: M.Tree | null, right: M.Tree | null) => new M.Tree(value, left, right);

const id = "550e8400-e29b-41d4-a716-446655440000" as M.Uuid;
const u128Max = 340282366920938463463374607431768211455n;
const i128Min = -170141183460469231731687303715884105728n;

const sample = new M.JsonData(
    new M.Big(
        u128Max,
        i128Min,
        170141183460469231731687303715884105727n,
        1000n,
        -42n,
        18446744073709551615n,
        -9223372036854775808n,
        9223372036854775807n,
        new Map([[u128Max, 1]]),
        i128Min,
    ),
    new M.Floats(0.1, Math.PI, 3.0, 1e-300, -2.5, null),
    new M.Text("a", "é", "🦀", "quote \" backslash \\ slash / tab \t newline \n dollar $x unicode ✓", "#/defs"),
    new Uint8Array([0, 1, 127, 128, 255]),
    5,
    ["x", null],
    new M.Maps(
        new Map([["one", 1], ["two", 2]]),
        new Map([[1, "one"], [20, "twenty"]]),
        new Map([[-5n, true], [9223372036854775807n, false]]),
        new Map([[false, 0], [true, 1]]),
        new Map([["k", ["v", null]]]),
        new Map([[M.levelLow(), 1], [M.levelHigh(), 2]]),
        new Map([[id, 7]]),
        new Map([[new M.NewType("key"), 3]]),
    ),
    [
        M.choiceUnit(),
        M.choiceNewType("new"),
        M.choiceChar("c"),
        M.choiceTuple(1, "t"),
        M.choiceStruct(1, "b"),
        M.choiceStruct(2, null),
        M.choiceOther(-1),
        M.choiceNested(M.choiceNested(M.choiceUnit())),
        M.choiceBig(u128Max),
    ],
    [M.levelLow(), M.levelHigh()],
    [M.modeFast(), M.modeSlow()],
    [M.internalUnit(), M.internalWrapped(new M.Point(1, -1)), M.internalStruct(3, "z")],
    [
        M.adjacentUnit(),
        M.adjacentNewType(9),
        M.adjacentNewType(null),
        M.adjacentTuple(4, "four"),
        M.adjacentStruct("adj"),
    ],
    tree(1, tree(2, null, null), tree(3, tree(4, null, null), null)),
    M.mapCons(1, M.mapCons(2, M.mapNil())),
    new M.Renamed(1, 2, 3, true),
    null,
    [null, null],
    new M.UnitStruct(),
    new M.NewType("wrapped"),
    new M.TupleStruct(7, -7),
    [8, "eight"],
    [1, "q", null],
    [[-1, [true, "yes"]]],
    new Map([["pair", [1, 2]]]),
    new Map([["a", null], ["b", null]]),
    [M.hollowEmpty(null), M.hollowSomething(3)],
    [[1], [2], [3]],
    [1, 2, 3],
    id,
    [id],
);
"##;
}

/// Type-checks the TypeScript the installer generated in `dir`, with
/// `main_ts` beside it as `main.ts` and `input.json` holding `input`, with
/// `deno check`, runs `main.ts` with `deno run`, and returns what it printed
/// on lines starting `JSON:`, without that prefix.
fn run_typescript_json_main(dir: &Path, main_ts: &str, input: &[u8]) -> Vec<String> {
    std::fs::write(dir.join("main.ts"), main_ts).unwrap();
    std::fs::write(dir.join("input.json"), input).unwrap();

    // The installer's `package.json` would have Deno look for `@types/node`
    // in `node_modules` for `node:assert`, instead of using its own.
    let status = Command::new("deno")
        .current_dir(dir)
        .env("DENO_NO_PACKAGE_JSON", "1")
        .arg("check")
        .arg("--sloppy-imports")
        .arg("main.ts")
        .status()
        .unwrap();
    assert!(status.success(), "deno check failed");

    let output = Command::new("deno")
        .current_dir(dir)
        .env("DENO_NO_PACKAGE_JSON", "1")
        .arg("run")
        .arg("--sloppy-imports")
        .arg("--allow-read")
        .arg("main.ts")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "deno run failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("JSON:").map(str::to_string))
        .collect()
}

/// The assertions, and the JSON Rust wrote.
const PRELUDE: &str = r#"import assert from "node:assert";
const input: string = Deno.readTextFileSync("input.json");
"#;

/// Round-trips JSON between `serde_json` and the generated TypeScript, both
/// ways: TypeScript decodes what Rust wrote into the value it builds itself,
/// and Rust decodes what TypeScript wrote — re-encoding the decoded value,
/// and encoding the value it built — back into the original.
///
/// TypeScript's JSON is compared with Rust's by value, not byte for byte:
/// JavaScript writes map keys that are array indices first, and spells some
/// floats differently.
#[test]
fn test_typescript_json_runtime_round_trips_with_serde_json() {
    use json_fixture::{JsonData, TYPESCRIPT_SAMPLE, sample};

    let dir = tempdir().unwrap();
    typescript::Installer::new("example", dir.path())
        .plugin(JsonPlugin)
        .generate(&facet_generate::reflect!(JsonData).unwrap())
        .unwrap();

    let reference = serde_json::to_vec(&sample()).unwrap();

    let outputs = run_typescript_json_main(
        dir.path(),
        &format!(
            r#"import * as M from "./example";
{PRELUDE}{TYPESCRIPT_SAMPLE}
const value = M.JsonData.jsonDeserialize(input);
assert.deepStrictEqual(value, sample, "decoded mismatch");
assert.deepStrictEqual(value.big.u64_max, 18446744073709551615n, "u64 max");
assert.deepStrictEqual(value.big.by_big.get(u128Max), 1, "128-bit key");

// The registry records `struct EmptyStruct {{}}`, which Rust writes as `{{}}`,
// as a unit struct, so that reads too.
assert.deepStrictEqual(M.UnitStruct.jsonDeserialize("{{}}"), new M.UnitStruct(), "unit struct from {{}}");

// Keys a type does not have are ignored, as Rust ignores them.
assert.deepStrictEqual(M.Point.jsonDeserialize('{{"x": 1, "extra": [true], "y": 2}}'), new M.Point(1, 2), "unknown key");

// A missing optional field reads as `null`, as Rust reads it as `None`.
assert.deepStrictEqual(
    M.Floats.jsonDeserialize('{{"tenth": 0.1, "pi": 1.5, "whole": 3.0, "tiny": 1e-300, "negative": -2.5}}'),
    new M.Floats(0.1, 1.5, 3, 1e-300, -2.5, null),
    "missing optional field",
);

// Without JSON.parse source text access, an integer beyond 2^53 cannot be
// written or read exactly, so it throws, while smaller ones still work.
const rawJSON = (JSON as any).rawJSON;
delete (JSON as any).rawJSON;
assert.throws(() => M.JsonData.jsonSerialize(sample), /source text access/);
assert.deepStrictEqual(M.Point.jsonSerialize(new M.Point(1, 2)), '{{"x":1,"y":2}}', "small values");
(JSON as any).rawJSON = rawJSON;
const parse = JSON.parse;
JSON.parse = (text: string, reviver?: (this: any, key: string, value: any) => any) =>
    parse(text, reviver && ((key, value) => reviver(key, value)));
assert.throws(() => M.JsonData.jsonDeserialize(input), /source text access/);
assert.deepStrictEqual(M.jsonDeserializeChoice('{{"Big": 7}}'), M.choiceBig(7n), "small u128");
JSON.parse = parse;

// A value converts to JSON and back without text in between.
assert.deepStrictEqual(M.JsonData.fromJson(M.JsonData.toJson(sample)), sample, "toJson / fromJson");

console.log("JSON:" + M.JsonData.jsonSerialize(value));
console.log("JSON:" + M.JsonData.jsonSerialize(sample));

// Malformed and truncated input is rejected.
for (const bad of ["", "{{}}", "[]", "null", input.slice(0, -1)]) {{
    assert.throws(() => M.JsonData.jsonDeserialize(bad), "accepted bad input: " + bad);
}}
// So is a variant the enum does not have, or one missing its payload.
for (const bad of ['"Missing"', '"NewType"', '{{"Unit": null, "Tuple": [1, "t"]}}', '{{"Tuple": [1]}}', '{{"Unit": 1}}']) {{
    assert.throws(() => M.jsonDeserializeChoice(bad), "accepted bad variant: " + bad);
}}
for (const bad of ['{{"kind": "Missing"}}', "{{}}", '"Fast"']) {{
    assert.throws(() => M.jsonDeserializeMode(bad), "accepted bad internally tagged variant: " + bad);
}}
for (const bad of ['{{"t": "Missing"}}', '{{"c": 1}}', '{{"t": "Unit", "c": 1}}']) {{
    assert.throws(() => M.jsonDeserializeAdjacent(bad), "accepted bad adjacently tagged variant: " + bad);
}}
// And a field missing, of the wrong type, or out of range.
for (const bad of ['{{"x": 1}}', '{{"x": 1, "y": "2"}}', '{{"x": 1, "y": 2147483648}}', '{{"x": 1, "y": 1.5}}']) {{
    assert.throws(() => M.Point.jsonDeserialize(bad), "accepted a bad point: " + bad);
}}
assert.throws(() => M.jsonDeserializeChoice('{{"Char": "ab"}}'), "accepted two chars");
assert.throws(() => M.jsonDeserializeChoice('{{"Big": -1}}'), "accepted a negative u128");
assert.throws(() => M.jsonDeserializeChoice('{{"Big": 340282366920938463463374607431768211456}}'), "accepted u128::MAX + 1");
"#
        ),
        &reference,
    );

    assert_eq!(outputs.len(), 2, "{outputs:?}");
    let expected: serde_json::Value = serde_json::from_slice(&reference).unwrap();
    for output in &outputs {
        let value: JsonData = serde_json::from_str(output)
            .unwrap_or_else(|e| panic!("Rust could not read TypeScript's JSON: {e}\n{output}"));
        assert_eq!(value, sample(), "{output}");
        // Rust ignores keys it does not know, so compare the JSON itself too.
        let actual: serde_json::Value = serde_json::from_str(output).unwrap();
        assert_eq!(actual, expected, "{output}");
    }
}

/// Round-trips an `App`, which holds a `kv::Entry`, which holds ROOT types,
/// through JSON between `serde_json` and the generated TypeScript, whose
/// modules import each other and name each other's types (and so their
/// conversions) through those imports.
#[test]
fn test_typescript_json_runtime_across_root_and_namespace() {
    use common::across_namespaces::to_root::{App, Level, Outcome, Presence, Shared, kv};

    let dir = tempdir().unwrap();
    typescript::Installer::new("example", dir.path())
        .plugin(JsonPlugin)
        .generate(&common::across_namespaces::to_root::get_registry())
        .unwrap();

    let app = App {
        entry: kv::Entry {
            shared: Shared { id: 7 },
            level: Level::High,
            outcome: Outcome::Score(42),
            status: Presence::Offline,
            local: kv::Presence { since: u64::MAX },
        },
        shared: Shared { id: 3 },
    };
    let reference = serde_json::to_vec(&app).unwrap();

    let outputs = run_typescript_json_main(
        dir.path(),
        &format!(
            r#"import * as Kv from "./kv";
import * as Example from "./example";
{PRELUDE}
const expected = new Example.App(
    new Kv.Entry(
        new Example.Shared(7),
        Example.levelHigh(),
        Example.outcomeScore(42),
        Example.presenceOffline(),
        new Kv.Presence(18446744073709551615n),
    ),
    new Example.Shared(3),
);
const app = Example.App.jsonDeserialize(input);
assert.deepStrictEqual(app, expected, "app mismatch");
console.log("JSON:" + Example.App.jsonSerialize(app));
console.log("JSON:" + Example.App.jsonSerialize(expected));
"#
        ),
        &reference,
    );

    assert_eq!(outputs.len(), 2, "{outputs:?}");
    let expected: serde_json::Value = serde_json::from_slice(&reference).unwrap();
    for output in &outputs {
        let actual: serde_json::Value = serde_json::from_str(output).unwrap();
        assert_eq!(actual, expected, "{output}");
    }
}
