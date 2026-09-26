#![cfg(feature = "kotlin")]
//! Runtime tests for Kotlin bincode and JSON serialization.
//!
//! These tests generate Kotlin code, serialize data in Rust with bincode or
//! `serde_json`, then compile and run the generated Kotlin code to
//! deserialize, verify field values, and re-serialize — checking that the
//! bytes (for JSON, the values) round-trip correctly.
//!
//! # Toolchain requirement
//!
//! `kotlinc` and `java` must be on `PATH`. The test compiles all generated
//! `.kt` sources (including the serde runtime) into a single JAR with
//! `kotlinc -include-runtime`, then runs the JVM entry-point with
//! `java -classpath`.
//!
//! The JSON test builds with `gradle` instead, since the generated code needs
//! kotlinx.serialization's compiler plugin and runtime.
//!
//! Unlike the compilation-only test in `kotlin_generation.rs`, this test
//! actually *executes* the generated serialization logic and verifies the
//! bytes produced by the Kotlin code match what Rust's `bincode` produces
//! for the same value.

use std::{
    fs,
    io::{self, Write as _},
    path::{Path, PathBuf},
    process::Command,
};

use facet_generate::generation::{bincode::BincodePlugin, json::JsonPlugin, kotlin};
use tempfile::tempdir;

pub mod common;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect all `.kt` files under `dir`, recursively.
fn collect_kt_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(collect_kt_files(&path));
            } else if path.extension().is_some_and(|e| e == "kt") {
                out.push(path);
            }
        }
    }
    out
}

/// Format a `&[u8]` as a Kotlin `byteArrayOf(…)` literal.
///
/// Kotlin `Byte` is signed, so values > 127 must be cast to their signed
/// equivalents (e.g. 255u8 → -1i8).
fn quote_bytes_kotlin(bytes: &[u8]) -> String {
    let elems: Vec<String> = bytes.iter().map(|&b| b.cast_signed().to_string()).collect();
    format!("byteArrayOf({})", elems.join(", "))
}

/// Whether `kotlinc` is on `PATH`, so that a test can skip gracefully when it
/// is not (e.g. Windows CI runners that have Gradle but not the standalone
/// kotlinc compiler).
fn kotlinc_available() -> bool {
    match Command::new("kotlinc").arg("-version").output() {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("kotlinc not found on PATH — skipping runtime test");
            false
        }
        Err(e) => panic!("failed to probe kotlinc: {e}"),
        Ok(_) => true,
    }
}

/// Compile every `.kt` file under `dir` (generated types, serde runtime and
/// a top-level `Main.kt`) into a single self-contained JAR, and run it.
fn compile_and_run(dir: &Path) {
    let jar_path = dir.join("test.jar");
    let kt_files = collect_kt_files(dir);

    let status = Command::new("kotlinc")
        .args(&kt_files)
        .arg("-include-runtime")
        .arg("-d")
        .arg(&jar_path)
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success(), "kotlinc compilation failed");

    // `Main.kt` in the default package compiles to the JVM class `MainKt`.
    let status = Command::new("java")
        .arg("-classpath")
        .arg(&jar_path)
        .arg("MainKt")
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success(), "round-trip test failed");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_kotlin_bincode_runtime_on_uuid_data() {
    if !kotlinc_available() {
        return;
    }

    let registry = common::get_uuid_registry();
    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    // Generate the Kotlin source + serde/bincode runtime files.
    kotlin::Installer::new("com.example.testing", &dir)
        .plugin(BincodePlugin)
        .generate(&registry)
        .unwrap();

    let reference = common::get_uuid_reference_bytes();
    let id_str = common::UUID_ID.to_string();
    let parent_id_str = common::UUID_PARENT_ID.to_string();

    // Write a top-level Main.kt (default package → JVM class `MainKt`).
    let main_path = dir.join("Main.kt");
    let mut main_file = fs::File::create(&main_path).unwrap();
    writeln!(
        main_file,
        r#"import com.example.testing.UuidData
import java.util.UUID

fun main() {{
    val input = {bytes}
    val value = UuidData.bincodeDeserialize(input)

    check(value.id == UUID.fromString("{id}")) {{
        "id mismatch: ${{value.id}}"
    }}
    check(value.parentId == UUID.fromString("{parent_id}")) {{
        "parentId mismatch: ${{value.parentId}}"
    }}

    val output = value.bincodeSerialize()
    check(input.contentEquals(output)) {{
        "roundtrip failed:\n  input  = ${{input.toList()}}\n  output = ${{output.toList()}}"
    }}

    println("UUID roundtrip: PASSED")
}}
"#,
        bytes = quote_bytes_kotlin(&reference),
        id = id_str,
        parent_id = parent_id_str,
    )
    .unwrap();

    compile_and_run(&dir);
}

/// Round-trips values across a namespaced module and the root one: the ROOT
/// `App` holds a `kv::Entry`, which holds ROOT types (a struct, unit and data
/// enums, and an enum sharing its name with a `kv` struct). Each module names
/// the other's types from the root package (`com.example.testing.Shared`,
/// `com.example.testing.kv.Entry`).
#[test]
fn test_kotlin_bincode_runtime_across_root_and_namespace() {
    use common::across_namespaces::to_root::{App, Level, Outcome, Presence, Shared, kv};

    if !kotlinc_available() {
        return;
    }

    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    kotlin::Installer::new("com.example.testing", &dir)
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

    fs::write(
        dir.join("Main.kt"),
        format!(
            r#"import com.example.testing.App
import com.example.testing.Level
import com.example.testing.Outcome
import com.example.testing.Presence
import com.example.testing.Shared
import com.example.testing.kv.Entry

fun main() {{
    val expected = Entry(
        Shared(7u),
        Level.HIGH,
        Outcome.Score(42u),
        Presence.OFFLINE,
        com.example.testing.kv.Presence(9UL),
    )

    // kv -> ROOT: a namespaced type deserializing ROOT types
    val entryBytes = {entry_bytes}
    val entry = Entry.bincodeDeserialize(entryBytes)
    check(entry == expected) {{ "entry mismatch: $entry" }}
    check(entryBytes.contentEquals(entry.bincodeSerialize())) {{ "entry did not roundtrip" }}

    // ROOT -> kv -> ROOT
    val appBytes = {app_bytes}
    val app = App.bincodeDeserialize(appBytes)
    check(app == App(expected, Shared(3u))) {{ "app mismatch: $app" }}
    check(appBytes.contentEquals(app.bincodeSerialize())) {{ "app did not roundtrip" }}

    println("Root and namespace roundtrip: PASSED")
}}
"#,
            entry_bytes = quote_bytes_kotlin(&entry_bytes),
            app_bytes = quote_bytes_kotlin(&app_bytes),
        ),
    )
    .unwrap();

    compile_and_run(&dir);
}

/// Types whose bincode the Kotlin runtime once could not write (#127):
/// 128-bit integers, declared as `BigInteger`; `char`s, declared as `String`
/// so that one outside the BMP fits; and fixed-size arrays, which bincode
/// writes with no length prefix. Sequences and maps of `()` are here too.
#[allow(clippy::zero_sized_map_values)]
mod bincode_fixture {
    use std::collections::BTreeMap;

    use facet::Facet;
    use serde::Serialize;

    #[derive(Facet, Serialize)]
    pub struct BincodeData {
        pub u128_max: u128,
        pub u128_zero: u128,
        pub i128_min: i128,
        pub i128_max: i128,
        pub i128_zero: i128,
        pub i128_minus_one: i128,
        pub by_big: BTreeMap<u128, i128>,
        pub maybe_big: Option<i128>,
        pub ascii: char,
        pub two_bytes: char,
        pub three_bytes: char,
        pub four_bytes: char,
        pub chars: Vec<char>,
        pub maybe_char: Option<char>,
        pub units: Vec<()>,
        pub unit_values: BTreeMap<String, ()>,
        pub array: [u16; 3],
        pub nested_array: [[u8; 2]; 2],
        pub letter: Letter,
    }

    /// Nothing but a `char`, so that bytes that do not encode one are
    /// rejected by the `char` and not by what follows it.
    #[derive(Facet, Serialize)]
    pub struct Letter(pub char);

    /// The value the Kotlin side also builds, field for field.
    pub fn sample() -> BincodeData {
        BincodeData {
            u128_max: u128::MAX,
            u128_zero: 0,
            i128_min: i128::MIN,
            i128_max: i128::MAX,
            i128_zero: 0,
            i128_minus_one: -1,
            by_big: BTreeMap::from([(0, -1), (u128::MAX, i128::MIN)]),
            maybe_big: Some(i128::MAX),
            ascii: 'a',
            two_bytes: 'é',
            three_bytes: '€',
            four_bytes: '🦀',
            chars: vec!['z', 'ß', '✓', '😀'],
            maybe_char: Some('🦀'),
            units: vec![(), (), ()],
            unit_values: BTreeMap::from([("a".to_string(), ()), ("b".to_string(), ())]),
            array: [1, 2, u16::MAX],
            nested_array: [[1, 2], [3, u8::MAX]],
            letter: Letter('Ω'),
        }
    }

    /// The same value in Kotlin, as the generated types spell it.
    pub const KOTLIN_SAMPLE: &str = r#"
val u128Max = BigInteger("340282366920938463463374607431768211455")
val i128Min = BigInteger("-170141183460469231731687303715884105728")
val i128Max = BigInteger("170141183460469231731687303715884105727")

val sample = BincodeData(
    u128Max = u128Max,
    u128Zero = BigInteger.ZERO,
    i128Min = i128Min,
    i128Max = i128Max,
    i128Zero = BigInteger.ZERO,
    i128MinusOne = BigInteger.ONE.negate(),
    byBig = mapOf(BigInteger.ZERO to BigInteger.ONE.negate(), u128Max to i128Min),
    maybeBig = i128Max,
    ascii = "a",
    twoBytes = "é",
    threeBytes = "€",
    fourBytes = "🦀",
    chars = listOf("z", "ß", "✓", "😀"),
    maybeChar = "🦀",
    units = listOf(Unit, Unit, Unit),
    unitValues = mapOf("a" to Unit, "b" to Unit),
    array = listOf(1u, 2u, UShort.MAX_VALUE),
    nestedArray = listOf(listOf(1u, 2u), listOf(3u, UByte.MAX_VALUE)),
    letter = Letter("Ω"),
)
"#;
}

/// Round-trips 128-bit integers at their extremes, `char`s of one to four
/// UTF-8 bytes, fixed-size arrays and sequences and maps of `()` between
/// Rust's `bincode` and the generated Kotlin: Kotlin decodes Rust's bytes
/// into the value it builds itself and encodes both back into those bytes.
/// Values bincode cannot represent are rejected both ways (#127).
#[test]
fn test_kotlin_bincode_runtime_on_big_integers_chars_and_units() {
    use bincode_fixture::{BincodeData, KOTLIN_SAMPLE, sample};

    if !kotlinc_available() {
        return;
    }

    // Rust's `bincode` writes a `char` as its UTF-8 bytes, with no length.
    assert_eq!(bincode::serialize(&'a').unwrap(), b"a");
    assert_eq!(bincode::serialize(&'🦀').unwrap(), "🦀".as_bytes());

    let dir = tempdir().unwrap();
    let dir = dir.path().to_path_buf().join("testing");

    kotlin::Installer::new("com.example.testing", &dir)
        .plugin(BincodePlugin)
        .generate(&facet_generate::reflect!(BincodeData).unwrap())
        .unwrap();

    let reference = bincode::serialize(&sample()).unwrap();

    fs::write(
        dir.join("Main.kt"),
        format!(
            r#"import com.example.testing.BincodeData
import com.example.testing.Letter
import com.novi.serde.DeserializationError
import com.novi.serde.SerializationError
import java.math.BigInteger
{KOTLIN_SAMPLE}
fun main() {{
    val input = {bytes}
    val value = BincodeData.bincodeDeserialize(input)
    check(value == sample) {{ "decoded mismatch:\n  $value\n  $sample" }}

    for (output in listOf(value.bincodeSerialize(), sample.bincodeSerialize())) {{
        check(input.contentEquals(output)) {{
            "roundtrip failed:\n  input  = ${{input.toList()}}\n  output = ${{output.toList()}}"
        }}
    }}

    // A `BigInteger` out of range of the Rust integer is not serialized.
    val two = BigInteger.valueOf(2)
    for (bad in listOf(
        sample.copy(u128Max = u128Max + BigInteger.ONE),
        sample.copy(u128Zero = BigInteger.ONE.negate()),
        sample.copy(i128Min = i128Min - BigInteger.ONE),
        sample.copy(i128Max = i128Max + BigInteger.ONE),
        sample.copy(maybeBig = two.pow(200)),
    )) {{
        val error = runCatching {{ bad.bincodeSerialize() }}.exceptionOrNull()
        check(error is SerializationError) {{ "serialized an out-of-range integer: $error" }}
    }}
    // Nor is a `String` that is not exactly one Unicode scalar value: none,
    // two, a letter and a combining accent, or lone or reversed surrogates.
    for (bad in listOf("", "ab", "🦀🦀", "e" + 0x301.toChar(), "\uD83E", "\uDD80", "\uD83Ea", "\uDD80\uD83E")) {{
        val error = runCatching {{ Letter(bad).bincodeSerialize() }}.exceptionOrNull()
        check(error is SerializationError) {{ "serialized a bad char ${{bad.toList()}}: $error" }}
    }}

    // Nor are bytes that are not the UTF-8 encoding of one: a continuation
    // byte first, a byte UTF-8 never uses, a lead byte followed by something
    // other than a continuation or by too few of them, overlong encodings, a
    // surrogate, and a code point past U+10FFFF.
    for (bad in listOf(
        byteArrayOf(0x80.toByte()),
        byteArrayOf(0xff.toByte()),
        byteArrayOf(0xc3.toByte(), 0x41),
        byteArrayOf(0xf0.toByte(), 0x9f.toByte()),
        byteArrayOf(0xc0.toByte(), 0x80.toByte()),
        byteArrayOf(0xe0.toByte(), 0x80.toByte(), 0x80.toByte()),
        byteArrayOf(0xed.toByte(), 0xa0.toByte(), 0x80.toByte()),
        byteArrayOf(0xf4.toByte(), 0x90.toByte(), 0x80.toByte(), 0x80.toByte()),
    )) {{
        val error = runCatching {{ Letter.bincodeDeserialize(bad) }}.exceptionOrNull()
        check(error is DeserializationError) {{ "deserialized a bad char ${{bad.toList()}}: $error" }}
    }}

    println("Big integers, chars and units roundtrip: PASSED")
}}
"#,
            bytes = quote_bytes_kotlin(&reference),
        ),
    )
    .unwrap();

    compile_and_run(&dir);
}

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------

/// Types exercising every shape the JSON plugin encodes, whose JSON is
/// `serde_json`'s: the Kotlin side must read it and write JSON that reads
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
        pub modes: Vec<Mode>,
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

    /// Holds 128-bit integers, so it has its own serializer.
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

    /// Every field is one kotlinx writes the way Rust does, so the compiler
    /// plugin's serializer writes it.
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

    /// Shadows Kotlin's `List`.
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

    /// The value the Kotlin side also builds, field for field.
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
            nested: Some(Some(5)),
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
            list: List::Cons(1, Box::new(List::Cons(2, Box::new(List::Nil)))),
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

    /// The same value in Kotlin, as the generated types spell it.
    pub const KOTLIN_SAMPLE: &str = r##"
fun tree(value: UInt, left: Tree?, right: Tree?) = Tree(value, left, right)

val id: java.util.UUID = java.util.UUID.fromString("550e8400-e29b-41d4-a716-446655440000")
val u128Max = BigInteger("340282366920938463463374607431768211455")
val i128Min = BigInteger("-170141183460469231731687303715884105728")

val sample = JsonData(
    big = Big(
        u128Max = u128Max,
        i128Min = i128Min,
        i128Max = BigInteger("170141183460469231731687303715884105727"),
        u128Small = BigInteger.valueOf(1000),
        i128Negative = BigInteger.valueOf(-42),
        u64Max = ULong.MAX_VALUE,
        i64Min = Long.MIN_VALUE,
        i64Max = Long.MAX_VALUE,
        byBig = mapOf(u128Max to 1u),
        maybeBig = i128Min,
    ),
    floats = Floats(tenth = 0.1f, pi = Math.PI, whole = 3.0, tiny = 1e-300, negative = -2.5f, optional = null),
    text = Text(
        ascii = "a",
        accented = "é",
        crab = "🦀",
        escaped = "quote \" backslash \\ slash / tab \t newline \n dollar \$x unicode ✓",
        ref = "#/defs",
    ),
    bytes = com.novi.serde.Bytes(byteArrayOf(0, 1, 127, -128, -1)),
    nested = 5u,
    maybeChars = listOf("x", null),
    maps = Maps(
        byString = mapOf("one" to 1u, "two" to 2u),
        byInt = mapOf(1u to "one", 20u to "twenty"),
        byNegative = mapOf(-5L to true, Long.MAX_VALUE to false),
        byBool = mapOf(false to 0u, true to 1u),
        byChar = mapOf("k" to listOf("v", null)),
        byLevel = mapOf(Level.LOW to 1u, Level.HIGH to 2u),
        byUuid = mapOf(id to 7u),
        byNewtype = mapOf(NewType("key") to 3u),
    ),
    choices = listOf(
        Choice.Unit,
        Choice.NewType("new"),
        Choice.Char("c"),
        Choice.Tuple(1u, "t"),
        Choice.Struct(a = 1u, bee = "b"),
        Choice.Struct(a = 2u, bee = null),
        Choice.Other(-1),
        Choice.Nested(Choice.Nested(Choice.Unit)),
        Choice.Big(u128Max),
    ),
    levels = listOf(Level.LOW, Level.HIGH),
    modes = listOf(Mode.FAST, Mode.SLOW),
    internal = listOf(Internal.Unit, Internal.Wrapped(Point(1, -1)), Internal.Struct(3, "z")),
    adjacent = listOf(
        Adjacent.Unit,
        Adjacent.NewType(9u),
        Adjacent.NewType(null),
        Adjacent.Tuple(4u, "four"),
        Adjacent.Struct("adj"),
    ),
    tree = tree(1u, tree(2u, null, null), tree(3u, tree(4u, null, null), null)),
    list = com.example.List.Cons(1u, com.example.List.Cons(2u, com.example.List.Nil)),
    renamed = Renamed(snakeCaseField = 1u, explicit = 2u, withDash = 3u, default = true),
    unit = Unit,
    units = listOf(Unit, Unit),
    unitStruct = UnitStruct,
    newtype = NewType("wrapped"),
    tupleStruct = TupleStruct(7u, -7),
    pair = Pair(8u, "eight"),
    tuple = Triple(1u, "q", null),
    nestedTuples = listOf(Pair(-1, Pair(true, "yes"))),
    tupleMap = mapOf("pair" to Pair(1u, 2u)),
    unitValues = mapOf("a" to Unit, "b" to Unit),
    hollow = listOf(Hollow.Empty(Unit), Hollow.Something(3u)),
    array = listOf(1u, 2u, 3u),
    set = setOf(1u, 2u, 3u),
    id = id,
    ids = listOf(id),
)
"##;
}

/// Turns the Kotlin sources the installer generated in `dir` into a Gradle
/// project whose `runMain` task runs `main_kt` with `input.json` beside it
/// holding `input`, runs it, and returns what it printed on lines starting
/// `JSON:`, without that prefix.
///
/// Gradle, not `kotlinc`, because the JSON plugin's code needs
/// kotlinx.serialization's compiler plugin and runtime.
fn run_kotlin_json_main(dir: &Path, main_kt: &str, input: &[u8]) -> Vec<String> {
    let source_set = dir.join("src/main/kotlin");
    fs::create_dir_all(&source_set).unwrap();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name == "src" || name == "build.gradle.kts" {
            continue;
        }
        fs::rename(entry.path(), source_set.join(&name)).unwrap();
    }
    fs::write(source_set.join("Main.kt"), main_kt).unwrap();
    fs::write(dir.join("input.json"), input).unwrap();

    let manifest = dir.join("build.gradle.kts");
    let mut contents = fs::read_to_string(&manifest).unwrap();
    // Pin the JVM target, so the build does not fail with "Inconsistent
    // JVM-target compatibility" on a JDK newer than Kotlin supports.
    contents.push_str(
        r#"
java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

kotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
        allWarningsAsErrors.set(true)
    }
}

tasks.register<JavaExec>("runMain") {
    classpath = sourceSets["main"].runtimeClasspath
    mainClass.set("MainKt")
    workingDir = projectDir
}
"#,
    );
    fs::write(&manifest, contents).unwrap();

    let output = Command::new("gradle")
        .args(["--configuration-cache", "--quiet", "runMain"])
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "gradle runMain failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("JSON:").map(str::to_string))
        .collect()
}

/// Round-trips JSON between `serde_json` and the generated Kotlin, both ways:
/// Kotlin decodes what Rust wrote into the value it builds itself, and Rust
/// decodes what Kotlin wrote — re-encoding the decoded value, and encoding the
/// value it built — back into the original.
///
/// Kotlin's JSON is compared with Rust's by value, not byte for byte: the two
/// spell some floats differently (`1.0E-300` for `1e-300`), and Kotlin writes
/// a set in its own order.
#[test]
fn test_kotlin_json_runtime_round_trips_with_serde_json() {
    use json_fixture::{JsonData, KOTLIN_SAMPLE, sample};

    let dir = tempdir().unwrap();
    let dir = dir.path().join("testing");
    kotlin::Installer::new("com.example", &dir)
        .plugin(JsonPlugin)
        .generate(&facet_generate::reflect!(JsonData).unwrap())
        .unwrap();

    let reference = serde_json::to_vec(&sample()).unwrap();

    let outputs = run_kotlin_json_main(
        &dir,
        &format!(
            r#"import com.example.*
import java.io.File
import java.math.BigInteger
import kotlinx.serialization.json.Json
{KOTLIN_SAMPLE}
fun main() {{
    val input = File("input.json").readText()
    val value = Json.decodeFromString(JsonData.serializer(), input)

    // `Bytes` wraps a `ByteArray`, which `==` compares by reference.
    check(value.bytes.content.contentEquals(sample.bytes.content)) {{ "bytes mismatch: ${{value.bytes}}" }}
    check(value.copy(bytes = sample.bytes) == sample) {{ "decoded mismatch:\n  $value\n  $sample" }}

    // The registry records `struct EmptyStruct {{}}`, which Rust writes as
    // `{{}}`, as a unit struct, so that reads too.
    check(Json.decodeFromString(UnitStruct.serializer(), "{{}}") == UnitStruct) {{ "unit struct from {{}}" }}

    println("JSON:" + Json.encodeToString(JsonData.serializer(), value))
    println("JSON:" + Json.encodeToString(JsonData.serializer(), sample))

    // Malformed and truncated input is rejected.
    for (bad in listOf("", "{{}}", "[]", input.dropLast(1))) {{
        val accepted = runCatching {{ Json.decodeFromString(JsonData.serializer(), bad) }}.isSuccess
        check(!accepted) {{ "accepted bad input: $bad" }}
    }}
    // So is a variant the enum does not have, or one missing its payload.
    for (bad in listOf("\"Missing\"", "\"NewType\"", "{{\"Unit\": null, \"Tuple\": [1, \"t\"]}}")) {{
        val accepted = runCatching {{ Json.decodeFromString(Choice.serializer(), bad) }}.isSuccess
        check(!accepted) {{ "accepted bad variant: $bad" }}
    }}
}}
"#
        ),
        &reference,
    );

    assert_eq!(outputs.len(), 2, "{outputs:?}");
    let expected: serde_json::Value = serde_json::from_slice(&reference).unwrap();
    for output in &outputs {
        let value: JsonData = serde_json::from_str(output)
            .unwrap_or_else(|e| panic!("Rust could not read Kotlin's JSON: {e}\n{output}"));
        assert_eq!(value, sample(), "{output}");
        // Rust ignores keys it does not know, so compare the JSON itself too.
        let actual: serde_json::Value = serde_json::from_str(output).unwrap();
        assert_eq!(actual, expected, "{output}");
    }
}

/// Round-trips an `App`, which holds a `kv::Entry`, which holds ROOT types,
/// through JSON between `serde_json` and the generated Kotlin, whose modules
/// name each other's types (and so their serializers) from the root package.
#[test]
fn test_kotlin_json_runtime_across_root_and_namespace() {
    use common::across_namespaces::to_root::{App, Level, Outcome, Presence, Shared, kv};

    let dir = tempdir().unwrap();
    let dir = dir.path().join("testing");
    kotlin::Installer::new("com.example.testing", &dir)
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

    let outputs = run_kotlin_json_main(
        &dir,
        r#"import com.example.testing.App
import com.example.testing.Level
import com.example.testing.Outcome
import com.example.testing.Presence
import com.example.testing.Shared
import com.example.testing.kv.Entry
import java.io.File
import kotlinx.serialization.json.Json

fun main() {
    val expected = App(
        Entry(
            Shared(7u),
            Level.HIGH,
            Outcome.Score(42u),
            Presence.OFFLINE,
            com.example.testing.kv.Presence(ULong.MAX_VALUE),
        ),
        Shared(3u),
    )
    val app = Json.decodeFromString(App.serializer(), File("input.json").readText())
    check(app == expected) { "app mismatch: $app" }
    println("JSON:" + Json.encodeToString(App.serializer(), app))
    println("JSON:" + Json.encodeToString(App.serializer(), expected))
}
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
