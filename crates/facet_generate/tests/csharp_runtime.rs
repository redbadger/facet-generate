#![cfg(feature = "csharp")]
//! Runtime tests for C# bincode and JSON serialization.
//!
//! These tests generate C# code, serialize data in Rust with bincode or
//! `serde_json`, then run the generated C# code to deserialize, verify, and
//! re-serialize — checking that the bytes (for JSON, the values) roundtrip
//! correctly.

use std::{
    fs,
    io::Write as _,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use facet::Facet;
use facet_generate::{
    generation::{bincode::BincodePlugin, csharp, json::JsonPlugin},
    reflect,
};
use serde::Serialize;
use tempfile::tempdir;

pub mod common;

use common::{Choice, Test};

/// Turn a `.csproj` class library into an executable so `dotnet run` works.
fn make_executable(dir: &std::path::Path, package_name: &str) {
    let csproj_path = dir.join(format!("{package_name}.csproj"));
    let content = fs::read_to_string(&csproj_path).unwrap();
    let content = content.replace(
        "<TargetFramework>",
        "<OutputType>Exe</OutputType>\n    <TargetFramework>",
    );
    fs::write(&csproj_path, content).unwrap();
}

/// Runs the project in `dir`, and returns what it printed.
fn dotnet_run(dir: &std::path::Path) -> String {
    const TIMEOUT: Duration = Duration::from_secs(300);

    let mut child = Command::new("dotnet")
        .arg("run")
        .current_dir(dir)
        .env("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1")
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .env("DOTNET_NOLOGO", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let deadline = Instant::now() + TIMEOUT;
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "dotnet run timed out after {} seconds:\nstdout: {}\nstderr: {}",
                TIMEOUT.as_secs(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }

        thread::sleep(Duration::from_millis(100));
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "dotnet run failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout).unwrap()
}

fn quote_bytes(bytes: &[u8]) -> String {
    format!(
        "new byte[] {{ {} }}",
        bytes
            .iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[test]
fn test_csharp_bincode_runtime_with_keyword_fields() {
    #[derive(Facet, Serialize)]
    #[allow(clippy::struct_excessive_bools)]
    struct KeywordFields {
        event: bool,
        lock: bool,
        class: bool,
        namespace: bool,
    }

    let registry = reflect!(KeywordFields).unwrap();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let reference = bincode::serialize(&KeywordFields {
        event: true,
        lock: false,
        class: true,
        namespace: false,
    })
    .unwrap();

    make_executable(&dir, "Example.Testing");
    fs::write(
        dir.join("Program.cs"),
        format!(
            r#"using System;
using System.Linq;
using Example.Testing;

byte[] input = {bytes};
var value = KeywordFields.BincodeDeserialize(input);

if (!value.Event || value.Lock || !value.Class || value.Namespace)
    throw new Exception("Keyword field values did not deserialize correctly");
if (!input.SequenceEqual(value.BincodeSerialize()))
    throw new Exception("Keyword field roundtrip failed");
"#,
            bytes = quote_bytes(&reference),
        ),
    )
    .unwrap();

    dotnet_run(&dir);
}

#[test]
fn test_csharp_bincode_runtime_on_optional_c_style_enums() {
    #[derive(Facet, Serialize)]
    #[repr(C)]
    #[allow(dead_code)]
    enum ContactGroup {
        Align,
        Partner,
    }

    #[derive(Facet, Serialize)]
    struct ContactFilter {
        group: Option<ContactGroup>,
        groups: Vec<Option<ContactGroup>>,
    }

    let registry = reflect!(ContactFilter).unwrap();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let samples = [
        ContactFilter {
            group: None,
            groups: vec![],
        },
        ContactFilter {
            group: Some(ContactGroup::Align),
            groups: vec![None, Some(ContactGroup::Align)],
        },
        ContactFilter {
            group: Some(ContactGroup::Partner),
            groups: vec![Some(ContactGroup::Partner), None],
        },
    ];
    let inputs = samples
        .iter()
        .map(|sample| quote_bytes(&bincode::serialize(sample).unwrap()))
        .collect::<Vec<_>>()
        .join(",\n        ");

    make_executable(&dir, "Example.Testing");
    fs::write(
        dir.join("Program.cs"),
        format!(
            r#"using System;
using System.Linq;
using Example.Testing;

static void Assert(bool condition, string message)
{{
    if (!condition) throw new Exception("Assertion failed: " + message);
}}

byte[][] inputs = new byte[][] {{
        {inputs}
}};

for (int i = 0; i < inputs.Length; i++)
{{
    byte[] input = inputs[i];
    var value = ContactFilter.BincodeDeserialize(input);

    switch (i)
    {{
        case 0:
            Assert(value.Group is null, "None group should remain null");
            Assert(value.Groups.Count == 0, "None sample should have no groups");
            break;
        case 1:
            Assert(value.Group.HasValue && value.Group.Value == ContactGroup.Align, "Align group should roundtrip");
            Assert(value.Groups.Count == 2, "Align sample should have two groups");
            Assert(value.Groups[0] is null, "first nested group should remain null");
            Assert(value.Groups[1].HasValue && value.Groups[1].Value == ContactGroup.Align, "nested Align should roundtrip");
            break;
        case 2:
            Assert(value.Group.HasValue && value.Group.Value == ContactGroup.Partner, "Partner group should roundtrip");
            Assert(value.Groups.Count == 2, "Partner sample should have two groups");
            Assert(value.Groups[0].HasValue && value.Groups[0].Value == ContactGroup.Partner, "nested Partner should roundtrip");
            Assert(value.Groups[1] is null, "last nested group should remain null");
            break;
    }}

    Assert(input.SequenceEqual(value.BincodeSerialize()), $"sample {{i}} did not roundtrip");
}}

Console.WriteLine("Optional C-style enum roundtrip: PASSED");
"#,
        ),
    )
    .unwrap();

    dotnet_run(&dir);
}

#[test]
fn test_csharp_bincode_runtime_on_uuid_data() {
    let registry = common::get_uuid_registry();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let reference = common::get_uuid_reference_bytes();
    let id_str = common::UUID_ID.to_string();
    let parent_id_str = common::UUID_PARENT_ID.to_string();

    make_executable(&dir, "Example.Testing");

    let program_path = dir.join("Program.cs");
    let mut program = std::fs::File::create(program_path).unwrap();
    writeln!(
        program,
        r#"using System;
using System.Linq;
using Example.Testing;
using Facet.Runtime.Serde;
using Facet.Runtime.Bincode;

static void Assert(bool condition, string message)
{{
    if (!condition) throw new Exception("Assertion failed: " + message);
}}

byte[] input = {bytes};
var value = UuidData.BincodeDeserialize(input);

Assert(value.Id == new Guid("{id}"), $"Id should be {id}, got {{value.Id}}");
Assert(value.ParentId == new Guid("{parent_id}"), $"ParentId should be {parent_id}, got {{value.ParentId}}");

var output = value.BincodeSerialize();
Assert(input.SequenceEqual(output), $"Roundtrip failed: {{input.Length}} bytes in, {{output.Length}} bytes out");

Console.WriteLine("UUID roundtrip: PASSED");
"#,
        bytes = quote_bytes(&reference),
        id = id_str,
        parent_id = parent_id_str,
    )
    .unwrap();

    dotnet_run(&dir);
}

#[test]
fn test_csharp_bincode_runtime_on_simple_data() {
    let registry = common::get_simple_registry();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let reference = bincode::serialize(&Test {
        a: vec![4, 6],
        b: (-3, 5),
        c: Choice::C { x: 7 },
    })
    .unwrap();

    make_executable(&dir, "Example.Testing");

    let program_path = dir.join("Program.cs");
    let mut program = fs::File::create(program_path).unwrap();
    writeln!(
        program,
        r#"using System;
using System.Linq;
using Example.Testing;
using Facet.Runtime.Serde;
using Facet.Runtime.Bincode;

static void Assert(bool condition, string message)
{{
    if (!condition) throw new Exception("Assertion failed: " + message);
}}

byte[] input = {0};
var value = Test.BincodeDeserialize(input);

// Verify deserialized values
Assert(value.A.Count == 2, "A should have 2 elements");
Assert(value.A[0] == 4, "A[0] should be 4");
Assert(value.A[1] == 6, "A[1] should be 6");
Assert(value.B == (-3L, 5UL), "B should be (-3, 5)");
Assert(value.C is Choice.C, "C should be Choice.C variant");
var c = (Choice.C)value.C;
Assert(c.X == 7, "C.X should be 7");

// Roundtrip: re-serialize and check bytes match
var output = value.BincodeSerialize();
Assert(input.SequenceEqual(output), "Roundtrip failed: serialized bytes don't match");

// Verify error on extra bytes
byte[] tooLong = input.Concat(new byte[] {{ 0 }}).ToArray();
try
{{
    Test.BincodeDeserialize(tooLong);
    Assert(false, "Should have thrown on extra bytes");
}}
catch (DeserializationError)
{{
    // expected
}}

// Verify error on completely invalid bytes
try
{{
    Test.BincodeDeserialize(new byte[] {{ 0, 1 }});
    Assert(false, "Should have thrown on invalid bytes");
}}
catch (Exception)
{{
    // expected — may be DeserializationError or EndOfStreamException
}}

Console.WriteLine("Simple data roundtrip: PASSED");
"#,
        quote_bytes(&reference),
    )
    .unwrap();

    dotnet_run(&dir);
}

/// Round-trips values across a namespaced module and the root one: the ROOT
/// `App` holds a `kv::Entry`, which holds ROOT types (a struct, unit and data
/// enums, and an enum sharing its name with a `kv` struct). Each module names
/// the other's types from the root package (`Company.Models.Shared`,
/// `Company.Models.Kv.Entry`), and a unit enum through its helper class.
#[test]
fn test_csharp_bincode_runtime_across_root_and_namespace() {
    use common::across_namespaces::to_root::{App, Level, Outcome, Presence, Shared, kv};

    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    csharp::Installer::new("Company.Models", &dir)
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
    let entry_bytes = bincode::serialize(&entry).unwrap();
    let app_bytes = bincode::serialize(&App {
        entry,
        shared: Shared { id: 3 },
    })
    .unwrap();

    make_executable(&dir, "Company.Models");
    fs::write(
        dir.join("Program.cs"),
        format!(
            r#"using System;
using System.Linq;

static void Assert(bool condition, string message)
{{
    if (!condition) throw new Exception("Assertion failed: " + message);
}}

static void AssertEntry(Company.Models.Kv.Entry entry, string context)
{{
    Assert(entry.Shared.Id == 7, context + ": Shared.Id should be 7");
    Assert(entry.Level == Company.Models.Level.High, context + ": Level should be High");
    Assert(entry.Outcome is Company.Models.Outcome.Score {{ Value: 42 }}, context + ": Outcome should be Score(42)");
    Assert(entry.Status == Company.Models.Presence.Offline, context + ": Status should be Offline");
    Assert(entry.Local.Since == 9, context + ": Local.Since should be 9");
}}

// kv -> ROOT: a namespaced type deserializing ROOT types
byte[] entryBytes = {entry_bytes};
var entry = Company.Models.Kv.Entry.BincodeDeserialize(entryBytes);
AssertEntry(entry, "entry");
Assert(entryBytes.SequenceEqual(entry.BincodeSerialize()), "entry did not roundtrip");

// ROOT -> kv -> ROOT
byte[] appBytes = {app_bytes};
var app = Company.Models.App.BincodeDeserialize(appBytes);
AssertEntry(app.Entry, "app.Entry");
Assert(app.Shared.Id == 3, "app.Shared.Id should be 3");
Assert(appBytes.SequenceEqual(app.BincodeSerialize()), "app did not roundtrip");

Console.WriteLine("Root and namespace roundtrip: PASSED");
"#,
            entry_bytes = quote_bytes(&entry_bytes),
            app_bytes = quote_bytes(&app_bytes),
        ),
    )
    .unwrap();

    dotnet_run(&dir);
}

/// The repro of redbadger/facet-generate#174 round trips: a variant named like
/// the type it holds, and a property named like a sibling variant, which the
/// variant's record declares again to hide the nested record it inherits.
#[test]
fn test_csharp_bincode_runtime_on_variants_named_like_types_or_properties() {
    #[derive(Facet, Serialize)]
    struct Presence {
        x: u32,
    }

    #[derive(Facet, Serialize)]
    #[repr(C)]
    enum Event {
        Presence(Presence),
        Seen { presence: u32, other: Presence },
    }

    #[derive(Facet, Serialize)]
    #[repr(C)]
    #[allow(dead_code)]
    enum Shape {
        Value { x: u32 },
        Wrap(u32),
    }

    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    csharp::Installer::new("Example", &dir)
        .plugin(BincodePlugin)
        .generate(&reflect!(Event, Shape).unwrap())
        .unwrap();

    let presence = bincode::serialize(&Event::Presence(Presence { x: 5 })).unwrap();
    let seen = bincode::serialize(&Event::Seen {
        presence: 3,
        other: Presence { x: 4 },
    })
    .unwrap();
    let wrap = bincode::serialize(&Shape::Wrap(9)).unwrap();

    make_executable(&dir, "Example");
    fs::write(
        dir.join("Program.cs"),
        format!(
            r#"using System;
using System.Linq;
using Example;

static void Assert(bool condition, string message)
{{
    if (!condition) throw new Exception("Assertion failed: " + message);
}}

byte[] presenceBytes = {presence};
var presence = Event.BincodeDeserialize(presenceBytes);
Assert(presence is Event.Presence {{ Value.X: 5 }}, "Presence(Presence {{ x: 5 }})");
Assert(presenceBytes.SequenceEqual(presence.BincodeSerialize()), "Presence did not roundtrip");

byte[] seenBytes = {seen};
var seen = Event.BincodeDeserialize(seenBytes);
Assert(seen is Event.Seen {{ Presence: 3, Other.X: 4 }}, "Seen {{ presence: 3, other: Presence {{ x: 4 }} }}");
Assert(seenBytes.SequenceEqual(seen.BincodeSerialize()), "Seen did not roundtrip");

byte[] wrapBytes = {wrap};
var wrap = Shape.BincodeDeserialize(wrapBytes);
Assert(wrap is Shape.Wrap {{ Value: 9 }}, "Wrap(9)");
Assert(wrapBytes.SequenceEqual(wrap.BincodeSerialize()), "Wrap did not roundtrip");

Console.WriteLine("Variants named like types or properties roundtrip: PASSED");
"#,
            presence = quote_bytes(&presence),
            seen = quote_bytes(&seen),
            wrap = quote_bytes(&wrap),
        ),
    )
    .unwrap();

    dotnet_run(&dir);
}

#[test]
#[ignore = "too slow for now, let's fix it later"]
fn test_csharp_bincode_runtime_on_supported_types() {
    let registry = common::get_registry();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    csharp::Installer::new("Example.Testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let positive_encodings = common::get_positive_samples()
        .iter()
        .map(|bytes| quote_bytes(bytes))
        .collect::<Vec<_>>()
        .join(",\n        ");

    make_executable(&dir, "Example.Testing");

    let program_path = dir.join("Program.cs");
    let mut program = fs::File::create(program_path).unwrap();
    writeln!(
        program,
        r#"using System;
using System.Linq;
using Example.Testing;
using Facet.Runtime.Serde;
using Facet.Runtime.Bincode;

static void Assert(bool condition, string message)
{{
    if (!condition) throw new Exception("Assertion failed: " + message);
}}

byte[][] positiveInputs = new byte[][] {{
        {positive_encodings}
}};

int passed = 0;
for (int i = 0; i < positiveInputs.Length; i++)
{{
    byte[] input = positiveInputs[i];
    var value = SerdeData.BincodeDeserialize(input);
    var output = value.BincodeSerialize();
    Assert(
        input.SequenceEqual(output),
        $"Roundtrip failed for sample {{i}}: input length={{input.Length}}, output length={{output.Length}}"
    );

    // Self-equality via byte comparison: deserialize twice, serialize both,
    // check bytes match. (C# partial classes lack structural Equals, so we
    // compare serialized bytes instead.)
    var value2 = SerdeData.BincodeDeserialize(input);
    var output2 = value2.BincodeSerialize();
    Assert(
        output.SequenceEqual(output2),
        $"Self-equality (via bytes) failed for sample {{i}}"
    );

    // Mutation testing: flip the high bit of each byte (up to 40) and verify
    // that deserialization either fails or produces a different value.
    for (int j = 0; j < Math.Min(40, input.Length); j++)
    {{
        byte[] mutated = (byte[])input.Clone();
        mutated[j] ^= 0x80;
        try
        {{
            var mutatedValue = SerdeData.BincodeDeserialize(mutated);
            var mutatedOutput = mutatedValue.BincodeSerialize();
            Assert(
                !input.SequenceEqual(mutatedOutput),
                $"Mutated byte {{j}} should give different serialized output for sample {{i}}"
            );
        }}
        catch (Exception)
        {{
            // Deserialization failure on mutated input is acceptable
        }}
    }}

    passed++;
}}

Console.WriteLine($"Supported types roundtrip + mutation: {{passed}}/{{positiveInputs.Length}} PASSED");
"#,
    )
    .unwrap();

    dotnet_run(&dir);
}

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------

/// Types exercising every shape the JSON plugin encodes, whose JSON is
/// `serde_json`'s: the C# side must read it and write JSON that reads back to
/// the same Rust value.
///
/// The Kotlin runtime test's fixture, but for what C# cannot hold: a `char`
/// outside the Basic Multilingual Plane (a C# `char` is one UTF-16 unit), and
/// `Option<Option<T>>` (C# has no `T??`). It adds the variants of #174.
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
        pub events: Vec<Event>,
        pub tree: Tree,
        pub list: HashSet,
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
        pub symbol: char,
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

    /// Variants named like a type they hold, and like a sibling's property
    /// (#174).
    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum Event {
        Presence(Presence),
        Seen { presence: u32, other: Presence },
        Value,
        Wrap(u32),
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Presence {
        pub x: u32,
    }

    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct Tree {
        pub value: u32,
        pub left: Option<Box<Tree>>,
        pub right: Option<Box<Tree>>,
    }

    /// Shadows .NET's `HashSet`, which the `set` field is.
    #[derive(Facet, Serialize, Deserialize, Debug, PartialEq, Eq)]
    #[repr(C)]
    pub enum HashSet {
        Nil,
        Cons(u32, Box<HashSet>),
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

    /// The value the C# side also builds, field for field.
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
                symbol: '✓',
                escaped: "quote \" backslash \\ slash / tab \t newline \n dollar $x crab 🦀"
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
            events: vec![
                Event::Presence(Presence { x: 5 }),
                Event::Seen {
                    presence: 3,
                    other: Presence { x: 4 },
                },
                Event::Value,
                Event::Wrap(9),
            ],
            tree: tree(
                1,
                Some(tree(2, None, None)),
                Some(tree(3, Some(tree(4, None, None)), None)),
            ),
            list: HashSet::Cons(1, Box::new(HashSet::Cons(2, Box::new(HashSet::Nil)))),
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

    /// The same value in C#, as the generated types spell it.
    pub const CSHARP_SAMPLE: &str = r##"
static Tree MakeTree(uint value, Tree? left, Tree? right) =>
    new Tree { Value = value, Left = left, Right = right };

static JsonData Sample()
{
    var id = new Guid("550e8400-e29b-41d4-a716-446655440000");
    return new JsonData
    {
        Big = new Big
        {
            U128Max = UInt128.MaxValue,
            I128Min = Int128.MinValue,
            I128Max = Int128.MaxValue,
            U128Small = 1000u,
            I128Negative = -42,
            U64Max = ulong.MaxValue,
            I64Min = long.MinValue,
            I64Max = long.MaxValue,
            ByBig = new Dictionary<UInt128, byte> { [UInt128.MaxValue] = 1 },
            MaybeBig = Int128.MinValue,
        },
        Floats = new Floats { Tenth = 0.1f, Pi = Math.PI, Whole = 3.0, Tiny = 1e-300, Negative = -2.5f, Optional = null },
        Text = new Text
        {
            Ascii = 'a',
            Accented = 'é',
            Symbol = '✓',
            Escaped = "quote \" backslash \\ slash / tab \t newline \n dollar $x crab 🦀",
            Ref = "#/defs",
        },
        Bytes = new byte[] { 0, 1, 127, 128, 255 },
        Maybe = 5,
        MaybeChars = new ObservableCollection<char?> { 'x', null },
        Maps = new Maps
        {
            ByString = new Dictionary<string, uint> { ["one"] = 1, ["two"] = 2 },
            ByInt = new Dictionary<uint, string> { [1] = "one", [20] = "twenty" },
            ByNegative = new Dictionary<long, bool> { [-5] = true, [long.MaxValue] = false },
            ByBool = new Dictionary<bool, byte> { [false] = 0, [true] = 1 },
            ByChar = new Dictionary<char, ObservableCollection<char?>> { ['k'] = new ObservableCollection<char?> { 'v', null } },
            ByLevel = new Dictionary<Level, byte> { [Level.Low] = 1, [Level.High] = 2 },
            ByUuid = new Dictionary<Guid, byte> { [id] = 7 },
            ByNewtype = new Dictionary<NewType, byte> { [new NewType { Value = "key" }] = 3 },
        },
        Choices = new ObservableCollection<Choice>
        {
            new Choice.Unit(),
            new Choice.NewType("new"),
            new Choice.Char('c'),
            new Choice.Tuple(1, 't'),
            new Choice.Struct(1, "b"),
            new Choice.Struct(2, null),
            new Choice.Other(-1),
            new Choice.Nested(new Choice.Nested(new Choice.Unit())),
            new Choice.Big(UInt128.MaxValue),
        },
        Levels = new ObservableCollection<Level> { Level.Low, Level.High },
        Modes = new ObservableCollection<Mode> { Mode.Fast, Mode.Slow },
        Internal = new ObservableCollection<Internal>
        {
            new Internal.Unit(),
            new Internal.Wrapped(new Point { X = 1, Y = -1 }),
            new Internal.Struct(3, 'z'),
        },
        Adjacent = new ObservableCollection<Adjacent>
        {
            new Adjacent.Unit(),
            new Adjacent.NewType(9),
            new Adjacent.NewType(null),
            new Adjacent.Tuple(4, "four"),
            new Adjacent.Struct("adj"),
        },
        Events = new ObservableCollection<Event>
        {
            new Event.Presence(new Presence { X = 5 }),
            new Event.Seen(3, new Presence { X = 4 }),
            new Event.Value(),
            new Event.Wrap(9),
        },
        Tree = MakeTree(1, MakeTree(2, null, null), MakeTree(3, MakeTree(4, null, null), null)),
        List = new Example.HashSet.Cons(1, new Example.HashSet.Cons(2, new Example.HashSet.Nil())),
        Renamed = new Renamed { SnakeCaseField = 1, Explicit = 2, WithDash = 3, Default = true },
        Unit = default,
        Units = new ObservableCollection<Unit> { default, default },
        UnitStruct = new UnitStruct(),
        Newtype = new NewType { Value = "wrapped" },
        TupleStruct = new TupleStruct { Field0 = 7, Field1 = -7 },
        Pair = (8, "eight"),
        Tuple = (1, 'q', null),
        NestedTuples = new ObservableCollection<(sbyte, (bool, string))> { (-1, (true, "yes")) },
        TupleMap = new Dictionary<string, (byte, byte)> { ["pair"] = (1, 2) },
        UnitValues = new Dictionary<string, Unit> { ["a"] = default, ["b"] = default },
        Hollow = new ObservableCollection<Hollow> { new Hollow.Empty(default), new Hollow.Something(3) },
        Array = new ushort[] { 1, 2, 3 },
        Set = new System.Collections.Generic.HashSet<uint> { 1, 2, 3 },
        Id = id,
        Ids = new ObservableCollection<Guid> { id },
    };
}
"##;
}

/// Writes `program` as the `Program.cs` of the project the installer
/// generated in `dir`, with `input.json` beside it holding `input`, runs it,
/// and returns what it printed on lines starting `JSON:`, without that prefix.
fn run_csharp_json_program(
    dir: &std::path::Path,
    package: &str,
    program: &str,
    input: &[u8],
) -> Vec<String> {
    make_executable(dir, package);
    fs::write(dir.join("Program.cs"), program).unwrap();
    fs::write(dir.join("input.json"), input).unwrap();
    dotnet_run(dir)
        .lines()
        .filter_map(|line| line.strip_prefix("JSON:").map(str::to_string))
        .collect()
}

/// Round-trips JSON between `serde_json` and the generated C#, both ways: C#
/// decodes what Rust wrote into the value it builds itself, and Rust decodes
/// what C# wrote — re-encoding the decoded value, and encoding the value it
/// built — back into the original.
///
/// The generated classes compare by reference, so the C# side compares the
/// decoded value with its own by the JSON it writes for each. C#'s JSON is
/// compared with Rust's by value, not byte for byte: the two spell some
/// floats and escapes differently (`1E-300` for `1e-300`, `"` for `\"`).
#[test]
fn test_csharp_json_runtime_round_trips_with_serde_json() {
    use json_fixture::{CSHARP_SAMPLE, JsonData, sample};

    let dir = tempdir().unwrap();
    let dir = dir.path().join("testing");
    csharp::Installer::new("Example", &dir)
        .plugin(JsonPlugin)
        .generate(&reflect!(JsonData).unwrap())
        .unwrap();

    let reference = serde_json::to_vec(&sample()).unwrap();

    let outputs = run_csharp_json_program(
        &dir,
        "Example",
        &format!(
            r##"using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Text.Json;
using Example;
using Facet.Runtime.Json;
using Facet.Runtime.Serde;

static void Assert(bool condition, string message)
{{
    if (!condition) throw new Exception("Assertion failed: " + message);
}}

static bool Rejects<T>(string input)
{{
    try
    {{
        JsonSerde.Deserialize<T>(input);
        return false;
    }}
    catch (Exception)
    {{
        return true;
    }}
}}
{CSHARP_SAMPLE}
var input = File.ReadAllText("input.json");
var value = JsonData.JsonDeserialize(input);
var sample = Sample();

Assert(value.Big.U128Max == UInt128.MaxValue, "u128 max");
Assert(value.Big.I128Min == Int128.MinValue, "i128 min");
Assert(value.Big.U64Max == ulong.MaxValue, "u64 max");
Assert(value.Text.Ref == "#/defs", "$ref");
Assert(value.Text.Escaped == sample.Text.Escaped, "escaped text");
Assert(value.Bytes.AsSpan().SequenceEqual(sample.Bytes), "bytes");
Assert(value.Choices[4] is Choice.Struct {{ A: 1, Bee: "b" }}, "struct variant");
Assert(value.Events[1] is Event.Seen {{ Presence: 3, Other.X: 4 }}, "variant named like a type");
Assert(value.Maps.ByNewtype.Keys.Single().Value == "key", "newtype key");
Assert(value.JsonSerialize() == sample.JsonSerialize(), $"decoded mismatch:\n  {{value.JsonSerialize()}}\n  {{sample.JsonSerialize()}}");

// No options are needed: each type names its converter.
Assert(JsonSerializer.Serialize(sample) == sample.JsonSerialize(), "default options");

// The registry records `struct EmptyStruct {{}}`, which Rust writes as `{{}}`,
// as a unit struct, so that reads too.
Assert(JsonSerde.Deserialize<UnitStruct>("{{}}") is not null, "unit struct from {{}}");

// Keys a type does not have are ignored, as Rust ignores them.
Assert(JsonSerde.Deserialize<Point>("{{\"x\": 1, \"extra\": [true], \"y\": 2}}") is {{ X: 1, Y: 2 }}, "unknown key");

Console.WriteLine("JSON:" + value.JsonSerialize());
Console.WriteLine("JSON:" + sample.JsonSerialize());

// Malformed and truncated input is rejected.
foreach (var bad in new[] {{ "", "{{}}", "[]", "null", input[..^1] }})
{{
    Assert(Rejects<JsonData>(bad), "accepted bad input: " + bad);
}}
// So is a variant the enum does not have, or one missing its payload.
foreach (var bad in new[] {{ "\"Missing\"", "\"NewType\"", "{{\"Unit\": null, \"Tuple\": [1, \"t\"]}}", "{{\"Tuple\": [1]}}" }})
{{
    Assert(Rejects<Choice>(bad), "accepted bad variant: " + bad);
}}
foreach (var bad in new[] {{ "{{\"kind\": \"Missing\"}}", "{{}}", "\"Fast\"" }})
{{
    Assert(Rejects<Mode>(bad), "accepted bad internally tagged variant: " + bad);
}}
// And a field missing, or out of range.
Assert(Rejects<Point>("{{\"x\": 1}}"), "accepted a missing field");
Assert(Rejects<Point>("{{\"x\": 1, \"y\": 2147483648}}"), "accepted an out-of-range field");
"##
        ),
        &reference,
    );

    assert_eq!(outputs.len(), 2, "{outputs:?}");
    let expected: serde_json::Value = serde_json::from_slice(&reference).unwrap();
    for output in &outputs {
        let value: JsonData = serde_json::from_str(output)
            .unwrap_or_else(|e| panic!("Rust could not read C#'s JSON: {e}\n{output}"));
        assert_eq!(value, sample(), "{output}");
        // Rust ignores keys it does not know, so compare the JSON itself too.
        let actual: serde_json::Value = serde_json::from_str(output).unwrap();
        assert_eq!(actual, expected, "{output}");
    }
}

/// Round-trips an `App`, which holds a `kv::Entry`, which holds ROOT types,
/// through JSON between `serde_json` and the generated C#, whose namespaces
/// name each other's types (and so reach their converters) from the root
/// package.
#[test]
fn test_csharp_json_runtime_across_root_and_namespace() {
    use common::across_namespaces::to_root::{App, Level, Outcome, Presence, Shared, kv};

    let dir = tempdir().unwrap();
    let dir = dir.path().join("testing");
    csharp::Installer::new("Company.Models", &dir)
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

    let outputs = run_csharp_json_program(
        &dir,
        "Company.Models",
        r#"using System;
using System.IO;

static void Assert(bool condition, string message)
{
    if (!condition) throw new Exception("Assertion failed: " + message);
}

var expected = new Company.Models.App
{
    Entry = new Company.Models.Kv.Entry
    {
        Shared = new Company.Models.Shared { Id = 7 },
        Level = Company.Models.Level.High,
        Outcome = new Company.Models.Outcome.Score(42),
        Status = Company.Models.Presence.Offline,
        Local = new Company.Models.Kv.Presence { Since = ulong.MaxValue },
    },
    Shared = new Company.Models.Shared { Id = 3 },
};
var app = Company.Models.App.JsonDeserialize(File.ReadAllText("input.json"));
Assert(app.Entry.Outcome is Company.Models.Outcome.Score { Value: 42 }, "outcome");
Assert(app.Entry.Local.Since == ulong.MaxValue, "local");
Assert(app.JsonSerialize() == expected.JsonSerialize(), "app mismatch: " + app.JsonSerialize());
Console.WriteLine("JSON:" + app.JsonSerialize());
Console.WriteLine("JSON:" + expected.JsonSerialize());
"#,
        &reference,
    );

    assert_eq!(outputs.len(), 2, "{outputs:?}");
    let expected: serde_json::Value = serde_json::from_slice(&reference).unwrap();
    for output in &outputs {
        let actual: serde_json::Value = serde_json::from_str(output).unwrap();
        assert_eq!(actual, expected, "{output}");
    }
}
