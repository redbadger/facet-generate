# Serialization: bincode and JSON

Without a plugin, an installer writes type definitions only. Two plugins add code that
serializes those types in the target language, in the same encoding Rust uses for them:

- [`BincodePlugin`](crate::generation::bincode::BincodePlugin) — the bytes Rust's
  [`bincode`](https://docs.rs/bincode/1) 1.x writes with `bincode::serialize`.
- [`JsonPlugin`](crate::generation::json::JsonPlugin) — the JSON
  [`serde_json`](https://docs.rs/serde_json) writes for the same types.

This page covers what each plugin generates and how its output lines up with Rust's. Which
target type each Rust type becomes is in [Supported types](crate::guide::supported_types); the
package layout and toolchain for each language are on the [Swift](crate::guide::swift),
[Kotlin](crate::guide::kotlin), [C#](crate::guide::csharp) and
[TypeScript](crate::guide::typescript) pages.

## Choosing

| Plugins | What you get | Use it for |
|---|---|---|
| none | Type definitions only | Types you serialize some other way, or not at all |
| `BincodePlugin` | Binary serialization code and the bincode runtime | Passing values across an FFI boundary or to a Rust core in-process: compact, fast, no field names on the wire |
| `JsonPlugin` | JSON serialization through each platform's own JSON support | HTTP APIs, files, logs — anywhere a person may read the data or another service must |
| both | Both sets of code on the same types | A type that crosses both kinds of boundary |

Add a plugin with `.plugin(...)` on the installer. Both plugins can go on one installer, and
the generated module carries both sets of code:

**Rust**
```rust
use facet::Facet;
use facet_generate::{
    generation::{bincode::BincodePlugin, json::JsonPlugin, swift},
    reflection::RegistryBuilder,
};

#[derive(Facet)]
struct Point {
    x: f64,
    y: f64,
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let registry = RegistryBuilder::new().add_type::<Point>()?.build()?;

let out_dir = std::env::temp_dir().join("facet_generate_guide_serialization");
swift::Installer::new("Shapes", &out_dir)
    .plugin(BincodePlugin)
    .plugin(JsonPlugin)
    .generate(&registry)?;
# Ok(())
# }
```

Each plugin also has the installer write the runtime its code needs, so the generated package
builds on its own. (The Kotlin project needs its sources moved into `src/main/kotlin` first; see
the [Kotlin guide](crate::guide::kotlin#what-gets-written).) The two plugins together are covered by compilation tests: Swift with
`char` fields (#213), Kotlin with bytes and UUIDs (#204), and TypeScript with UUIDs (#191).

## Keep the `facet` and `serde` attributes in step

`facet_generate` reads the `#[facet(...)]` attributes; `serde` reads `#[serde(...)]`. Where an
attribute changes the wire format — `rename`, `rename_all`, `tag`, `content`, `skip` — declare it
in both, with the same value, or the Rust side and the generated side will disagree:

**Rust**
```rust
use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize)]
#[facet(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
struct Account {
    account_id: u32,
}

assert_eq!(
    serde_json::to_string(&Account { account_id: 7 }).unwrap(),
    r#"{"accountId":7}"#,
);
```

The examples below derive both `Facet` and serde's traits because they compare against Rust's
own `bincode` and `serde_json` output. The generated code needs only `Facet`; the serde derives
belong to whatever does the serializing on the Rust side.

## Bincode

### The wire format

The generated code reads and writes what `bincode::serialize` and `bincode::deserialize` produce
in bincode 1.x with its default configuration. The runtime tests take their expected bytes from
exactly those two functions.

| Rust value | Bytes |
|---|---|
| `u8`/`i8` … `u64`/`i64` | Fixed width, little-endian (no variable-length integers) |
| `u128`/`i128` | 16 bytes, little-endian: the low 64 bits, then the high 64 bits |
| `f32`/`f64` | The IEEE 754 bit pattern, little-endian |
| `bool` | One byte, `0` or `1` |
| `char` | Its UTF-8 bytes, one to four, with no length prefix |
| `String`, bytes, `Vec<T>`, maps, sets | A `u64` length, then the bytes or elements |
| `[T; N]`, tuples, structs | The elements or fields in order, with no length |
| `Option<T>` | A `0` byte for `None`; a `1` byte then the value for `Some` |
| enum | The variant's index as a `u32`, then its fields |
| `()`, unit struct | Nothing |

This is not the variable-length integer encoding of `bincode::DefaultOptions` or bincode 2's
`config::standard()`. Only `bincode::serialize` / `bincode::deserialize` from bincode 1.x are
tested.

Rust producing bytes for a small type:

**Rust**
```rust
use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize)]
struct Reading {
    id: u16,
    label: String,
    value: Option<u8>,
    initial: char,
    unit: Unit,
}

#[derive(Facet, Serialize, Deserialize)]
#[repr(C)]
enum Unit {
    Celsius,
    Fahrenheit,
}

let reading = Reading {
    id: 7,
    label: "hi".to_string(),
    value: Some(5),
    initial: 'é',
    unit: Unit::Fahrenheit,
};
assert_eq!(
    bincode::serialize(&reading).unwrap(),
    [
        7, 0, // id: u16
        2, 0, 0, 0, 0, 0, 0, 0, b'h', b'i', // label: u64 length, then UTF-8
        1, 5, // value: Some, then 5
        0xc3, 0xa9, // initial: 'é' as UTF-8, no length
        1, 0, 0, 0, // unit: variant index 1 as u32
    ],
);
```

### What each language gets

| Language | Encode | Decode | Runtime installed |
|---|---|---|---|
| Swift | `bincodeSerialize() throws -> [UInt8]` | `static bincodeDeserialize(input:)` | `Serde` target |
| Kotlin | `bincodeSerialize(): ByteArray` | `bincodeDeserialize(input)` on the companion | `com.novi.serde`, `com.novi.bincode` |
| C# | `BincodeSerialize(): byte[]` | `static BincodeDeserialize(byte[])` | `Facet.Runtime.Serde`, `Facet.Runtime.Bincode` |
| TypeScript | `serialize(serializer)` | `static deserialize(deserializer)` | `serde/`, `bincode/` |

Every type also gets the lower-level pair these wrap — `serialize(serializer:)` and
`deserialize(deserializer:)` in Swift, `serialize`/`deserialize` in Kotlin and TypeScript,
`Serialize`/`Deserialize` in C# — which read and write through a serializer you pass in.

Three exceptions to the table:

- A TypeScript enum is a union type, not a class, so it gets standalone
  `serialize{Enum}(value, serializer)` and `deserialize{Enum}(deserializer)` functions.
- TypeScript has no `bincodeSerialize` / `bincodeDeserialize` wrappers: create a
  `BincodeSerializer` or `BincodeDeserializer` from `./bincode` yourself.
- A C# enum whose variants are all unit variants is a plain `enum`, which can't hold methods,
  so its methods are on a static class beside it: `UnitBincode.BincodeSerialize(value)` and
  `UnitBincode.BincodeDeserialize(bytes)`.

The `bincodeDeserialize` wrappers in Swift, Kotlin and C# fail if any input is left over
("Some input bytes were not read"). In TypeScript, check
`deserializer.getBufferOffset()` against the input's length yourself if you need that.

Swift, decoding the bytes above and encoding them again:

**Swift**
```swift
import Serde
import Weather

let input: [UInt8] = [7, 0, 2, 0, 0, 0, 0, 0, 0, 0, 104, 105, 1, 5, 0xc3, 0xa9, 1, 0, 0, 0]
let reading = try Reading.bincodeDeserialize(input: input)
assert(reading == Reading(id: 7, label: "hi", value: 5, initial: "é", unit: .fahrenheit))
let output = try reading.bincodeSerialize()
assert(output == input)
```

TypeScript, doing the same:

**TypeScript**
```typescript
import { BincodeDeserializer, BincodeSerializer } from "./bincode";
import { Reading } from "./weather";

const input = new Uint8Array([7, 0, 2, 0, 0, 0, 0, 0, 0, 0, 104, 105, 1, 5, 0xc3, 0xa9, 1, 0, 0, 0]);
const deserializer = new BincodeDeserializer(input);
const reading = Reading.deserialize(deserializer);
if (deserializer.getBufferOffset() !== input.length) throw new Error("trailing bytes");

const serializer = new BincodeSerializer();
reading.serialize(serializer);
const output = serializer.getBytes(); // the same bytes as `input`
```

### Limits

- **Enum tagging does not apply.** The generated bincode code always writes the variant index,
  whatever `tag` or `content` says; tagging is for JSON. Rust's own `bincode` can't deserialize
  an internally or adjacently tagged enum (it needs `deserialize_any`), and writes an internally
  tagged one with its variant name as a string. An enum that crosses a bincode boundary should
  keep serde's default, external tagging.
- **Map and set entry order is not canonical.** The generated code writes a map or set in its
  own iteration order — for a Swift `Dictionary` or `Set`, no particular order — where a Rust
  `BTreeMap` writes its entries sorted. Either side reads entries in any order, so values round
  trip, but the bytes of a map or set with more than one entry may differ from Rust's. Don't
  compare or hash the encoded bytes.
- **A `char` must be one Unicode scalar value.** Swift's `Character`, and the `String` that
  Kotlin, C# and TypeScript use for a `char`, can hold more; the generated code refuses to
  serialize one that isn't exactly one scalar value, and refuses bytes that aren't the UTF-8
  encoding of one.

## JSON

### Matching `serde_json`

The generated code writes the JSON `serde_json` writes for the same Rust types, and reads what
it reads. Field and variant names come from the `facet` attributes, with `rename` and
`rename_all` applied. Beyond that:

| Rust | JSON |
|---|---|
| `Option<T>` | The value, or `null`. `None` is written as `null`; a missing field reads as `None`. |
| integers, including 64- and 128-bit | JSON numbers, written exactly (not as strings) |
| `f32`/`f64` | JSON numbers |
| `char` | A string of exactly one Unicode scalar value |
| bytes (`#[facet(facet_generate::bytes)]`) | An array of numbers, `[0, 255]` |
| `Vec<T>`, sets, `[T; N]` | Arrays |
| tuples, tuple structs | Arrays, including a one-element tuple (but see [known gaps](#what-is-tested-and-known-gaps)) |
| newtype struct | The inner value |
| unit struct, `()` | `null` |
| maps | Objects; keys that aren't strings — integers, `bool`, `char`, unit variants, UUIDs — are written as strings, as `serde_json` does |
| UUID | A string |

A struct, with its JSON from `serde_json`:

**Rust**
```rust
use std::collections::BTreeMap;

use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize)]
#[facet(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
struct Order {
    order_id: u64,
    note: Option<String>,
    initial: char,
    #[facet(facet_generate::bytes)]
    blob: Vec<u8>,
    by_id: BTreeMap<u32, String>,
}

let order = Order {
    order_id: u64::MAX,
    note: None,
    initial: 'é',
    blob: vec![0, 255],
    by_id: BTreeMap::from([(7, "seven".to_string())]),
};
assert_eq!(
    serde_json::to_string_pretty(&order).unwrap(),
    r#"{
  "orderId": 18446744073709551615,
  "note": null,
  "initial": "é",
  "blob": [
    0,
    255
  ],
  "byId": {
    "7": "seven"
  }
}"#,
);
```

### Enum representations

External tagging is serde's default. `#[facet(tag = "…")]` selects internal tagging and
`#[facet(tag = "…", content = "…")]` adjacent tagging, each matched by the same `#[serde(...)]`
attribute. A unit variant of an externally tagged enum is its name as a string.

**Rust**
```rust
use facet::Facet;
use serde::{Deserialize, Serialize};

// External (the default): a unit variant is a string, any other variant an
// object with the variant's name as its only key.
#[derive(Facet, Serialize, Deserialize)]
#[repr(C)]
enum Shape {
    Point,
    Circle(f64),
    Rect { w: f64, h: f64 },
}

// Internal: the tag sits among the variant's own fields.
#[derive(Facet, Serialize, Deserialize)]
#[repr(C)]
#[facet(tag = "type")]
#[serde(tag = "type")]
enum Event {
    Started,
    Moved { x: i32, y: i32 },
}

// Adjacent: the tag and the content side by side.
#[derive(Facet, Serialize, Deserialize)]
#[repr(C)]
#[facet(tag = "t", content = "c")]
#[serde(tag = "t", content = "c")]
enum Either {
    Left(u8),
    Right(String),
}

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap()
}

assert_eq!(json(&Shape::Point), r#""Point""#);
assert_eq!(json(&Shape::Circle(1.5)), r#"{"Circle":1.5}"#);
assert_eq!(json(&Shape::Rect { w: 1.0, h: 2.0 }), r#"{"Rect":{"w":1.0,"h":2.0}}"#);

assert_eq!(json(&Event::Started), r#"{"type":"Started"}"#);
assert_eq!(json(&Event::Moved { x: 1, y: 2 }), r#"{"type":"Moved","x":1,"y":2}"#);

assert_eq!(json(&Either::Left(3)), r#"{"t":"Left","c":3}"#);
assert_eq!(json(&Either::Right("r".into())), r#"{"t":"Right","c":"r"}"#);
# facet_generate::reflection::RegistryBuilder::new()
#     .add_type::<Shape>().unwrap()
#     .add_type::<Event>().unwrap()
#     .add_type::<Either>().unwrap()
#     .build()
#     .unwrap();
```

`#[facet(untagged)]` is not supported: reflection rejects an untagged enum with
[`Error::UntaggedEnum`](crate::error::Error::UntaggedEnum), rather than generate code that
won't read or write serde's untagged JSON.

### What each language gets

| Language | Built on | Encode | Decode |
|---|---|---|---|
| Swift | `Codable`, `JSONEncoder` / `JSONDecoder` | `jsonSerialize() throws -> [UInt8]` | `static jsonDeserialize(input:)` |
| Kotlin | kotlinx.serialization | `Json.encodeToString(T.serializer(), value)` | `Json.decodeFromString(T.serializer(), text)` |
| C# | System.Text.Json | `JsonSerialize(): string` | `static JsonDeserialize(string)` |
| TypeScript | `JSON.stringify` / `JSON.parse` | `static jsonSerialize(value): string` | `static jsonDeserialize(text)` |

- **Swift**: types conform to `Codable`, with `CodingKeys` and hand-written `init(from:)` /
  `encode(to:)` where Swift's synthesized ones would differ from serde. The runtime adds
  `JsonCoding.swift` to the `Serde` target. `jsonSerialize` and `jsonDeserialize` use a default
  `JSONEncoder` / `JSONDecoder`; an encoder with a key-encoding strategy changes the keys.
- **Kotlin**: types are `@Serializable`, with `@SerialName` giving each wire name. A struct whose
  fields kotlinx already encodes as serde does uses the serializer the compiler plugin
  generates; every other type — an enum with payloads, for example, or a struct with a field
  kotlinx would encode differently — names its own `JsonSerializer`, built on `JsonCoding.kt` in the runtime. The installer adds
  the `kotlinx-serialization-json` dependency and serialization compiler plugin to
  `build.gradle.kts`. Kotlin has no `jsonSerialize` helper: use your `Json` instance.
- **C#**: each type carries `[JsonConverter(typeof(TJsonConverter))]` and properties carry
  `[JsonPropertyName("…")]`, so `JsonSerializer.Serialize(value)` also produces serde's JSON with
  no options set. `JsonSerialize` / `JsonDeserialize` wrap `JsonSerde` in the
  `Facet.Runtime.Json` runtime. An all-unit-variant enum has no wrappers; use
  `JsonSerde.Serialize(value)` or `JsonSerializer`.
- **TypeScript**: a class gets static `toJson(value)` / `fromJson(json)`, which convert to and
  from a plain JSON value, and `jsonSerialize(value)` / `jsonDeserialize(text)`, which go to and
  from text. An enum gets the same as functions: `toJson{Enum}`, `fromJson{Enum}`,
  `jsonSerialize{Enum}`, `jsonDeserialize{Enum}`. They share one runtime file, `serde/json.ts`.

Reading and writing in each language:

**Swift**
```swift
let order = try Order.jsonDeserialize(input: Array(text.utf8))
let bytes: [UInt8] = try order.jsonSerialize()
```

**Kotlin**
```kotlin
import kotlinx.serialization.json.Json

val json = Json { ignoreUnknownKeys = true }
val order = json.decodeFromString(Order.serializer(), text)
val back = json.encodeToString(Order.serializer(), order)
```

**C#**
```csharp
var order = Order.JsonDeserialize(text);
string back = order.JsonSerialize();
```

**TypeScript**
```typescript
import { Order } from "./example";

const order = Order.jsonDeserialize(text);
const back = Order.jsonSerialize(order);
```

### Differences to know about

- **Unknown keys.** serde ignores object keys a struct doesn't have, as do the generated Swift,
  C# and TypeScript. Kotlin's default `Json` rejects an unknown key on a type that uses the
  compiler-generated serializer (`Encountered an unknown key`). Configure your `Json` with
  `ignoreUnknownKeys = true` to match serde.
- **Big integers in TypeScript.** A JavaScript number holds integers exactly only up to 2^53.
  `serde/json.ts` writes larger 64- and 128-bit integers with `JSON.rawJSON`, and reads them from
  the source text the `JSON.parse` reviver receives. Both need Node 21+, Deno 1.37+, Chrome 114+,
  Safari 18.4+ or Firefox 135+. On an older engine, only an integer beyond 2^53 throws. Read with
  `jsonDeserialize` (or the runtime's `parse`), not a bare `JSON.parse`, which rounds them.
- **Compare JSON by value, not text.** Key order, float spelling (Kotlin writes `1.0E-300` for
  `1e-300`) and escaping can differ from `serde_json`'s; the value read back is the same.

## What is tested, and known gaps

For each language, runtime tests in `tests/<lang>_runtime.rs` generate code, compile and run it
with the real toolchain, and check it against Rust:

- **Bincode**: the generated code decodes bytes from `bincode::serialize` into the value it
  builds itself, and encodes both back into those same bytes. The fixtures cover structs,
  enums, options, sequences, maps, fixed-size arrays, nested tuples, UUIDs,
  128-bit integers, `char`s of one to four UTF-8 bytes, and types split across namespaces.
- **JSON**: a fixture covering every shape the plugin encodes — the three enum representations,
  renames, nested options, recursive types, big integers, floats, escaped strings, bytes, maps
  with non-string keys, units and tuples — is written by `serde_json`, decoded by the generated
  code, and compared with the value it builds itself. The generated code then
  encodes both, and Rust decodes that JSON back into the original value. Malformed and
  truncated input is rejected.

Known gaps:

- `#[facet(untagged)]` enums are rejected at reflection, as above.
- Kotlin and Swift JSON write a one-element tuple as its bare element, where `serde_json`
  writes `[x]` ([#248](https://github.com/redbadger/facet-generate/issues/248)).
- Kotlin and TypeScript declare `Option<Option<T>>` as a single option, so a bincode
  `Some(None)` reads back as `None`
  ([#249](https://github.com/redbadger/facet-generate/issues/249)). C# rejects the type. In JSON
  `serde_json` writes both as `null`, so no language can tell them apart.
- Enum tagging is JSON-only; see [Limits](#limits) for bincode.
- Map and set bytes aren't canonical in bincode.
- Only bincode 1.x's `bincode::serialize` format is supported.
- serde attributes the reflector doesn't read, such as `default`, `flatten` and
  `skip_serializing_if`, change serde's output without changing the generated code. Avoid them
  on shared types. A generated reader requires every field that isn't an `Option`.
