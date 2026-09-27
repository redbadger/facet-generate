# Why `facet_generate`, and how it compares

## The problem

An app with a Rust core and native shells — Swift on iOS, Kotlin on Android, C# on Windows,
TypeScript on the web — has one data model that every language must agree on. The Rust types
are the natural source of truth. Mirroring them by hand in four languages works until someone
adds a field, reorders a variant or renames a type on one side only; then the shells decode
garbage or fail at runtime, and nothing warns you at build time.

`facet_generate` generates the mirrors. You derive [`Facet`](facet::Facet) on your Rust types,
and the pipeline runs in three steps:

1. [`RegistryBuilder`](crate::reflection::RegistryBuilder) walks the types' `facet` shape
   information, starting from the root types you name, and builds a language-neutral
   [`Registry`](crate::Registry) of every type they reach.
2. A per-language installer —
   [`swift::Installer`](crate::generation::swift::Installer),
   [`kotlin::Installer`](crate::generation::kotlin::Installer),
   [`csharp::Installer`](crate::generation::csharp::Installer) or
   [`typescript::Installer`](crate::generation::typescript::Installer) — turns the registry into
   idiomatic types: Swift structs and enums, Kotlin data classes and sealed interfaces, C#
   classes and records, TypeScript classes and discriminated unions.
3. Optionally, [`BincodePlugin`](crate::generation::bincode::BincodePlugin) or
   [`JsonPlugin`](crate::generation::json::JsonPlugin) adds serialization code whose bytes or
   JSON match what Rust's `bincode` or `serde_json` produce, and the installer copies in the
   runtime library that code calls.

The output is a package: a Swift package with a `Package.swift`, a `.csproj`, or a TypeScript
package with `package.json`, each of which builds as it stands, or a Kotlin project with a
`build.gradle.kts`. The Kotlin sources are written at the project root rather than in
`src/main/kotlin`, so move them, or copy them into a Gradle module of your own, before building
([#241](https://github.com/redbadger/facet-generate/issues/241)).

[Supported types](crate::guide::supported_types) lists every Rust type the reflector accepts
and what it becomes in each language. [Serialization](crate::guide::serialization) covers the
two formats, and the language pages ([Swift](crate::guide::swift),
[Kotlin](crate::guide::kotlin), [C#](crate::guide::csharp),
[TypeScript](crate::guide::typescript)) cover the packages.

## Why reflection with `facet`

A code generator needs to know the shape of each type: its fields, their types, its variants,
and the attributes that change its name or wire form. There are three common ways to get that
from Rust: parse the source code, trace what serde does at runtime, or read reflection data.

`#[derive(Facet)]` takes the third way. As facet's own
[“Why facet?”](https://facet.rs/guide/why/) puts it, serde generates *code* — a serializer
for your type — while facet generates *data*: a static description of the type, with its
fields, their names and types, and its attributes. `facet_generate` reads that description
directly. It sees facet's built-in attributes, such as `rename`, `rename_all`, `skip`,
`transparent`, `tag` and `content`, which mirror serde's (facet has a
[side-by-side comparison with serde](https://facet.rs/guide/serde/)), and its own extension
attributes in the `fg` namespace, such as `#[facet(fg::namespace = "...")]` and
`#[facet(fg::bytes)]`. Attributes that serde would discard, like a namespace, are just more data
in the shape.

[serde-reflection](https://github.com/zefchain/serde-reflection/tree/main/serde-reflection),
the project `facet_generate` descends from, takes the second way. Its `Tracer` runs a type's
`Serialize` and `Deserialize` implementations against a recording serializer and deserializer,
and infers the format from what they do. That works on any type serde can handle, including
hand-written implementations, but its README lists the costs:

- Each enum has to be traced separately to discover all its variants.
- For mutually recursive types, the first variant of each enum must be a base case, or tracing
  doesn't terminate.
- A type whose `Deserialize` validates its input needs sample values supplied first.
- A generic type can't be traced with more than one set of type parameters in a session, and
  two types with the same name from different modules collide.
- Attributes that binary formats can't express, such as `#[serde(flatten)]` and
  `#[serde(tag = "...")]`, aren't supported.

Reading `facet` shapes avoids the tracing rules: every variant is in the shape, recursion is a
reference to a type already seen, and nothing is executed. Some limits remain. A user-defined
generic type is registered under its own name with its parameters resolved, so `Page<String>`
becomes a `Page` whose items are strings, and using `Page<u32>` as well in the same registry is
an error. Two different Rust types that would generate the same name are an error too, until you
rename one or give it a namespace.

## Types and serialization, not a bridge

`facet_generate` generates types and, optionally, the code that serializes them. It doesn't
generate foreign-function bindings: there are no functions or objects in its output, and
nothing that loads a native library. Moving the bytes between Rust and the other language is
left to whatever transport you choose — [UniFFI](https://mozilla.github.io/uniffi-rs/),
[BoltFFI](https://github.com/boltffi/boltffi), [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/),
hand-written JNI or C ABI functions, an HTTP request, or a file.

[Crux](https://github.com/redbadger/crux) is built this way, and its maintainers' reason is
given in [#36](https://github.com/redbadger/facet-generate/issues/36): Crux separates the
*bridge*, for which it now uses BoltFFI (it used UniFFI when that answer was written), from the
*data that crosses the bridge*, for which it generates types with `facet_generate` and
serializes them with serde. The bridge stays a
handful of functions that pass bytes, the same in every Crux app however large its data model
grows, and everything that crosses it is plain data rather than objects with behaviour.

The whole bridge can be as small as this; the same function could be exported through any of
the transports above:

**Rust**
```rust
use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize)]
#[repr(C)]
enum Request {
    Get { key: String },
    Delete { key: String },
}

#[derive(Facet, Serialize, Deserialize, Debug, PartialEq)]
#[repr(C)]
enum Response {
    Value(Option<String>),
    Deleted,
}

/// The only function the shell calls: bincode bytes in, bincode bytes out.
pub fn handle(request: &[u8]) -> Vec<u8> {
    let response = match bincode::deserialize(request).expect("valid request") {
        Request::Get { key } => Response::Value(Some(format!("value of {key}"))),
        Request::Delete { .. } => Response::Deleted,
    };
    bincode::serialize(&response).expect("serializable response")
}

// The shells' `Request` and `Response` types, and their bincode code, are generated from the
// same Rust types.
let registry = facet_generate::reflection::RegistryBuilder::new()
    .add_type::<Request>()?
    .add_type::<Response>()?
    .build()?;
assert_eq!(registry.len(), 2);

let request = bincode::serialize(&Request::Delete { key: "a".into() })?;
let response: Response = bincode::deserialize(&handle(&request))?;
assert_eq!(response, Response::Deleted);
# Ok::<(), Box<dyn std::error::Error>>(())
```

UniFFI can carry rich types itself: its records and enums are
[lowered into a `RustBuffer`](https://mozilla.github.io/uniffi-rs/latest/internals/lifting_and_lowering.html)
of serialized bytes on every call. The difference is where the data model lives. With UniFFI
alone, every type in the model is part of the FFI interface and uses UniFFI's own encoding.
With `facet_generate` behind a byte-passing bridge, the interface doesn't change when the model
does, and the encoding is serde's, so the same bytes can also be stored, logged, or sent over a
network to a Rust server.

## How it compares

The tools below all generate code in other languages from Rust definitions, but they cover
different parts of the problem. Versions and details are as of September 2026:
`facet_generate` 0.22.0, [UniFFI](https://github.com/mozilla/uniffi-rs) 0.32.2,
[typeshare](https://github.com/1Password/typeshare) 1.0.5 (the `typeshare-cli` tool is
1.13.4), [serde-reflection](https://crates.io/crates/serde-reflection) 0.6.0 with
[serde-generate](https://crates.io/crates/serde-generate) 0.34.1,
[ts-rs](https://github.com/Aleph-Alpha/ts-rs) 12.0.1, and
[specta](https://github.com/specta-rs/specta) 2.0 (release candidate 2.0.0-rc.25; the latest
stable release is 1.0.5, and the per-language exporters are 0.0.x).

| Tool | Target languages | Generates | Source of the type information |
| --- | --- | --- | --- |
| `facet_generate` | Swift, Kotlin, C#, TypeScript | Types, optional serialization, runtimes, package manifests | `facet` reflection data, read at run time |
| UniFFI | Kotlin, Swift, Python, Ruby; others third-party | FFI bindings: functions, objects, records, enums | UDL file or proc-macros |
| typeshare | Kotlin, Swift, TypeScript, Scala; Go, Python experimental | Types | `#[typeshare]` on Rust source, parsed by a CLI |
| serde-reflection + serde-generate | C++, Java, Python, Rust, Go, C#, Swift, OCaml, Dart, Kotlin; TypeScript, Solidity partial | Types, serialization, runtimes | Tracing serde's `Serialize`/`Deserialize` |
| ts-rs | TypeScript | Type declarations | `#[derive(TS)]` |
| specta | TypeScript, Swift stable; Kotlin, C#, Go and others partial | Types | `#[derive(Type)]`, collected at run time |

| Tool | Wire format | serde attributes | Namespaces | Generics |
| --- | --- | --- | --- | --- |
| `facet_generate` | Bincode 1, JSON (serde_json's) | facet's equivalents | `fg::namespace` | One instantiation per type |
| UniFFI | Its own buffer encoding | Not used | One per crate | Built-in containers |
| typeshare | JSON, via each language's JSON library | Read | Multi-file output | Supported |
| serde-reflection + serde-generate | Bincode 1, BCS | Binary-compatible ones | Module name, external definitions | One instantiation per type |
| ts-rs | None generated | Read (`serde-compat`) | One file per type, with imports | Supported |
| specta | JSON, via the target's library (Swift `Codable`) | Read | Exporter-dependent | Supported |

Each tool can also be extended: `facet_generate` through
[`EmitterPlugin`](crate::generation::plugin::EmitterPlugin) (see
[Writing a plugin](crate::guide::plugins)); UniFFI through third-party bindings generators;
serde-generate through custom code attached to a type; ts-rs through `#[ts(as = "...")]` and
`#[ts(type = "...")]`; specta through exporter crates built on its type collection; typeshare
through per-language type mappings and decorators in its configuration.

### Lineage

`facet_generate` grew out of vendored copies of
[serde-reflection and serde-generate](https://github.com/zefchain/serde-reflection), which were
written at Facebook for the Diem project (as `novifinancial/serde-reflection`) and are now
maintained by Zefchain. Its [`Format`](crate::reflection::format::Format) and
[`ContainerFormat`](crate::reflection::format::ContainerFormat) model comes from
serde-reflection, much of its source still carries the Facebook copyright notice, and its
runtime libraries descend from serde-generate's — which is why the Kotlin runtime lives in the
`com.novi.serde` package and the Swift runtime is a target called `Serde`. What changed is the
front end: `facet` reflection replaced serde tracing. The generators have since gained
namespaces, plugins, package manifests, C# and JSON output.

### When to choose something else

**UniFFI** generates the bridge itself: functions, objects with methods, callbacks, async and
errors across the FFI, for Kotlin, Swift, Python and Ruby, with more languages from third
parties. If you want to call a Rust API from Swift or Kotlin rather than exchange messages
with a Rust core, UniFFI is the tool, and `facet_generate` has nothing equivalent. The two
combine well: UniFFI (or BoltFFI) for a small bridge, `facet_generate` for the data across it.

**typeshare** is the simplest option when JSON is the wire format and serde's JSON attributes
already describe your types. It reads serde attributes directly, handles generics, targets
Scala and (experimentally) Go and Python, and needs no build step in your crate — a CLI reads
the source. It doesn't target C# or produce a binary format, and because it reads source rather
than compiled types, a type it can't see, such as a `DateTime`, is mapped to a foreign type in
its configuration.

**serde-reflection and serde-generate** cover more languages than any tool here — C++, Java,
Python, Go, OCaml and Dart among them — and support BCS, a canonical binary format for
hashing and signing. They work on existing serde types without adding a derive. Choose them if
you need one of those languages or BCS, or can't add `#[derive(Facet)]` to your types; expect
the tracing rules above, and no JSON code generation.

**ts-rs** is focused on TypeScript and does it thoroughly: it reads most of serde's
attributes, including `flatten` and `untagged`, handles generic types, and has implementations for many ecosystem crates. If your only other language is TypeScript
and the wire format is JSON produced by serde_json, it understands more of serde than
`facet_generate` does.

**specta** is the choice when you also want the types of functions — it powers
[tauri-specta](https://github.com/specta-rs/tauri-specta) and [rspc](https://github.com/specta-rs/rspc) —
or schema outputs such as OpenAPI, JSON Schema, Zod or Valibot. Its TypeScript and Swift
exporters are marked stable; the others, including Kotlin and C#, are marked partial. Like
typeshare, it leaves serialization to each language's JSON library, and has no binary format.

**`facet_generate`** fits when the shells are Swift, Kotlin, C# or TypeScript, you want the
serialization code generated along with the types (in bincode for compactness or JSON for
readability, with bytes that match Rust's), and you want each language's output as a package,
with namespaces becoming Swift targets, Kotlin packages, C# namespaces or TypeScript modules.
