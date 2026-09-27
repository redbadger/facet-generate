# Swift

The Swift installer writes a Swift package: a `Package.swift` manifest, one
SwiftPM target per namespace, and, when a plugin needs one, a `Serde` target
holding the serialization runtime. The package builds with `swift build` as it
is written.

## Quick start

Reflect the types into a [`Registry`](crate::Registry), then hand it to
[`swift::Installer`](crate::generation::swift::Installer):

**Rust**
```rust
use facet::Facet;
use facet_generate as fg;
use facet_generate::{
    generation::{bincode::BincodePlugin, json::JsonPlugin, swift},
    reflection::RegistryBuilder,
};

#[derive(Facet)]
#[facet(fg::namespace = "geometry")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Facet)]
#[repr(C)]
pub enum Shape {
    Circle { centre: Point, radius: f64 },
    Label(String),
    Empty,
}

#[derive(Facet)]
pub struct Drawing {
    pub name: String,
    pub shapes: Vec<Shape>,
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let registry = RegistryBuilder::new().add_type::<Drawing>()?.build()?;
let out_dir = std::env::temp_dir().join("facet_generate_guide_swift");

swift::Installer::new("Drawing", &out_dir)
    .plugin(BincodePlugin)
    .plugin(JsonPlugin)
    .platforms(&[".iOS(.v16)".to_string(), ".macOS(.v13)".to_string()])
    .generate(&registry)?;

assert!(out_dir.join("Package.swift").exists());
assert!(out_dir.join("Sources/Geometry/Geometry.swift").exists());
# Ok(())
# }
```

The `fg::` attributes expand to paths in the `facet-generate-attrs` crate, so a
crate that uses them depends on it as well as on `facet` and `facet_generate`:
`cargo add facet facet_generate facet-generate-attrs`.

The installer is a builder. Everything on it is optional apart from the two
arguments to [`new`](crate::generation::swift::Installer::new):

| Method | What it does |
|---|---|
| [`new(package_name, dir)`](crate::generation::swift::Installer::new) | Names the package and the directory it is written to. The root target is the package name in `UpperCamelCase`. |
| [`plugin`](crate::generation::swift::Installer::plugin) | Adds an [`EmitterPlugin`](crate::generation::plugin::EmitterPlugin), such as [`BincodePlugin`](crate::generation::bincode::BincodePlugin) or [`JsonPlugin`](crate::generation::json::JsonPlugin). Call it once per plugin. Without one, only the type declarations are written. |
| [`external_packages`](crate::generation::swift::Installer::external_packages) | Takes namespaces from packages generated separately; see [External packages](#external-packages). |
| [`platforms`](crate::generation::swift::Installer::platforms) | Sets the manifest's `platforms:` line, from raw SwiftPM expressions such as `".iOS(.v16)"`. With none, the line is left out and SwiftPM uses its defaults. |
| [`generate`](crate::generation::swift::Installer::generate) | Checks the namespaces, then writes the runtime, the sources and the manifest. |

[`make_manifest`](crate::generation::swift::Installer::make_manifest) returns
the text of `Package.swift` without writing anything.

## What gets written

The example above, with only `BincodePlugin` and no `platforms`, writes:

**Directory tree**
```text
Drawing/
├── Package.swift
└── Sources/
    ├── Drawing/
    │   └── Drawing.swift        Drawing, Shape (the root namespace)
    ├── Geometry/
    │   └── Geometry.swift       Point (namespace "geometry")
    └── Serde/                   the runtime
        ├── BinaryDeserializer.swift
        ├── BinarySerializer.swift
        ├── BincodeDeserializer.swift
        ├── BincodeSerializer.swift
        ├── Deserializer.swift
        ├── Indirect.swift
        ├── Int128.swift
        ├── Serializer.swift
        └── UInt128.swift
```

and this manifest:

**Swift** (`Package.swift`)
```swift
// swift-tools-version: 5.8
import PackageDescription

let package = Package(
    name: "Drawing",
    products: [
        .library(
            name: "Drawing",
            targets: ["Drawing"]
        )
    ],
    targets: [
        .target(
            name: "Drawing",
            dependencies: ["Geometry", "Serde"]
        ),
        .target(
            name: "Geometry",
            dependencies: ["Serde"]
        ),
        .target(
            name: "Serde",
            dependencies: []
        ),
    ]
)
```

The manifest declares tools version 5.8 and has no dependencies of its own. It
gains a `dependencies:` section only for [external
packages](#external-packages), and a `platforms:` line only when you set one.
The single library product, named after the package, lists the targets that no
other target depends on. When every type is in a named namespace there is no
root target, since SwiftPM rejects a target with no sources, and the product
lists the namespace targets instead.

## Runtimes

A plugin's runtime is written to `Sources/Serde/` and declared as the `Serde`
target, which every generated target then depends on and imports.

| Plugins | `Sources/Serde/` |
|---|---|
| none | not written |
| [`BincodePlugin`](crate::generation::bincode::BincodePlugin) | `Serializer`, `Deserializer`, `BinarySerializer`, `BinaryDeserializer`, `BincodeSerializer`, `BincodeDeserializer`, `Indirect`, `Int128`, `UInt128` |
| [`JsonPlugin`](crate::generation::json::JsonPlugin) | `JsonCoding`, `Indirect`, `Int128`, `UInt128` |
| both | the union of the two |

When an external package provides the namespace `serde`, the installer writes
no runtime and the `Serde` target comes from that package instead; see
[External packages](#external-packages).

## Namespaces

Each namespace set with `#[facet(fg::namespace = "...")]` becomes a SwiftPM
target named after the namespace in `UpperCamelCase`, written to
`Sources/<Target>/<Target>.swift`. Types in the root namespace go in the
package's own target. A module refers to another's types through the module
name and imports it:

**Swift** (`Sources/Drawing/Drawing.swift`)
```swift
import Geometry
import Serde

indirect public enum Shape: Hashable, Equatable {
    case circle(centre: Geometry.Point, radius: Double)
    case label(String)
    case empty
    // …
}
```

A namespaced type that refers to a root type makes its target depend on the
root target. SwiftPM rejects a cycle of target dependencies, so a root type
holding a namespaced type that refers back to the root is rejected before
anything is written:

**Rust**
```rust
use facet::Facet;
use facet_generate as fg;
use facet_generate::{generation::{bincode::BincodePlugin, swift}, reflect};

#[derive(Facet)]
#[facet(fg::namespace)] // back in the root namespace
pub struct Shared {
    pub id: u32,
}

#[derive(Facet)]
#[facet(fg::namespace = "kv")]
pub struct Entry {
    pub shared: Shared,
}

#[derive(Facet)]
pub struct App {
    pub entry: Entry,
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let out_dir = std::env::temp_dir().join("facet_generate_guide_swift_cycle");
# let _ = std::fs::remove_dir_all(&out_dir);
let error = swift::Installer::new("Example", &out_dir)
    .plugin(BincodePlugin)
    .generate(&reflect!(App)?)
    .unwrap_err();

assert_eq!(
    error.to_string(),
    "Swift targets cannot depend on each other in a cycle, and these would: \
     `Example` references `Entry` in `Kv`; `Kv` references `Shared` in `Example`. \
     Move the types that one of these targets references into a namespace of their \
     own (`#[facet(fg::namespace = \"…\")]`), which the targets can both depend on"
);
assert!(!out_dir.join("Sources").exists());
# Ok(())
# }
```

Before writing anything, the installer also rejects a namespace whose target
would not compile or would overwrite another. Each error names the namespace
and what it collides with, and says what to change:

- two namespaces that become the same target (`kv` and `Kv`), or a namespace
  and the root package (`app` in package `App`);
- a namespace named like a type in scope where it is used, so that Swift finds
  the type instead of the module:

  **Error message**
  ```text
  Swift: namespace "kv" becomes the module `Kv`, which qualifies its types in module `Example`, the same as type `Kv` in the root namespace. Rename the type with `#[facet(rename = "...")]` or choose a different namespace
  ```

- a namespace named like a standard-library type (`string`, `error`,
  `result`), a `Serde` runtime type, or a Foundation type in a module that
  imports Foundation:

  **Error message**
  ```text
  Swift: namespace "string" becomes the module `String`, which qualifies its types in module `Example`, the same as the standard-library type `String`, which Swift finds instead of the module. Choose a different namespace
  ```

- a namespace named like a module the package imports: `Swift`, `Serde`
  (with a plugin), `Foundation`, or an SDK module Foundation loads.

A type named like something the generated code uses is rejected too, for
example a type called `Serializer` beside `BincodePlugin`:

**Error message**
```text
Swift: type `Serializer` collides with the runtime protocol `Serde.Serializer` used by the generated code; rename it with #[facet(rename = "...")]
```

## External packages

An [`ExternalPackage`](crate::generation::ExternalPackage) says that a
namespace's types come from another package, generated separately. The
installer skips that namespace's module, adds the package to the manifest's
`dependencies:`, and makes each target that uses it depend on a product named
after the namespace in `UpperCamelCase`. The types are still written as
`Geometry.Point` and the module is imported as `import Geometry`.

| [`PackageLocation`](crate::generation::PackageLocation) | Manifest entry |
|---|---|
| `Path("../Geometry")` | `.package(path: "../Geometry")` |
| `Url("https://github.com/example/geometry")`, `version: Some("1.2.0")` | `.package(url: "https://github.com/example/geometry", from: "1.2.0")` |

A `Url` needs a [`version`](crate::generation::ExternalPackage::version), since
SwiftPM needs a requirement for a package it fetches. Without one, the
installer fails before writing anything:

**Error message**
```text
Swift: the external package for namespace `geometry` at `https://github.com/example/geometry` has no version, and SwiftPM needs one to fetch it. Set its `version`, which the manifest writes as `from: "<version>"`, or give its location as a `PackageLocation::Path`
```

`module_name` is not used by Swift.

Every package the installer writes with a plugin holds its own `Serde` target,
and SwiftPM refuses two targets of the same name in one package graph. So when
more than one generated package uses a plugin, generate the runtime once as a
package of its own, from an empty registry, and point every package at it as
the external package for the namespace `serde`. The
[`Installer`](crate::generation::swift::Installer#sharing-the-runtime-between-packages)
docs show how. A package generated this way can also take other namespaces
from sibling packages, as in the table above.

## Using the generated code

With both plugins, a value round-trips through bincode and through the JSON
that `serde_json` writes:

**Swift**
```swift
import Drawing
import Geometry

let drawing = Drawing(
    name: "sketch",
    shapes: [
        .circle(centre: Point(x: 0, y: 0), radius: 1.5),
        .label("hello"),
        .empty,
    ]
)

// bincode
let bytes: [UInt8] = try drawing.bincodeSerialize()
let decoded = try Drawing.bincodeDeserialize(input: bytes)
assert(decoded == drawing)

// JSON
let json: [UInt8] = try drawing.jsonSerialize()
let parsed = try Drawing.jsonDeserialize(input: json)
assert(parsed == drawing)
```

`bincodeDeserialize` throws `DeserializationError.invalidInput` when bytes are
left over after the value. `serialize(serializer:)` and
`deserialize(deserializer:)` take any `Serializer` or `Deserializer` from the
runtime, for writing several values to one buffer. [Serialization: bincode and
JSON](crate::guide::serialization) covers what the two formats look like on the
wire.

## Naming

Type names are kept as they are in the registry, after any
`#[facet(rename = "...")]`. Fields, enum cases and initialiser labels are
`lowerCamelCase`: `hex_code` becomes `hexCode` and the variant `DarkRed`
becomes `case darkRed`. With `JsonPlugin`, the `CodingKeys` map each back to
its Rust name (`case hexCode = "hex_code"`), so the JSON is unchanged.

A variant or field renamed to a name that starts with a digit, such as
`#[facet(rename = "2fa")]`, cannot be written as a Swift identifier, and
generation fails:

**Error message**
```text
Swift: variant `2fa` of `Status` would become `2fa`, which is not a valid identifier because it starts with a digit; rename it with #[facet(rename = "...")] to a name that starts with a letter
```

This check runs as each module is generated, so the runtime may already be on
disk ([#250](https://github.com/redbadger/facet-generate/issues/250)). A hyphen
or a space in a rename is dropped by the casing: `on-hold` becomes
`case onHold`.

A name that is a Swift keyword is escaped with backticks: a field `default`
becomes ``public var `default`: Bool`` and a variant `Class` becomes
``case `class` ``. Contextual keywords such as `type` and `get` are legal
identifiers and left alone.

A type named like a standard-library type the generated code writes bare
(`Set`, `String`, `Bool`, `Double`, …, or `UUID` from Foundation) shadows it
for the whole module, so the generated code writes the builtin fully
qualified: `Swift.Set<T>`, `Swift.String`, `Foundation.UUID`.

[Plugins](crate::guide::plugins) that write Swift should name things the same way, through
[`swift::field_name`](crate::generation::swift::field_name),
[`swift::case_name`](crate::generation::swift::case_name) and
[`swift::escape_identifier`](crate::generation::swift::escape_identifier).

## Language-specific behaviour

- **Structs and enums.** A struct becomes a `public struct` with `public var`
  properties and a memberwise `public init`. An enum becomes an
  `indirect public enum`, so it can hold itself. A tuple becomes a native
  tuple, of any length.
- **`Hashable` and `Equatable`.** A type declares `Hashable` when everything it
  holds is hashable, and `Equatable` when everything it holds is equatable.
  The installer decides this once over the whole registry, so a module that
  holds another module's type agrees with it. Native tuples and `Void` are not
  hashable. A type from an external package is assumed to conform to both.
  Using a type that is not hashable as a `Set` element or a dictionary key is
  an error at generation time.
- **`Codable`.** `JsonPlugin` adds `Codable`, after `Hashable` and `Equatable`,
  with the coding keys and the `init(from:)` / `encode(to:)` that the Rust
  JSON needs.
- **Bytes.** A field marked `#[facet(fg::bytes)]` becomes `[UInt8]`.

The [supported types](crate::guide::supported_types) page lists what every
Rust type becomes.

## Toolchain requirements

- The manifest declares `swift-tools-version: 5.8`, the oldest toolchain it
  asks for. The examples on this page were built with Swift 6.4.
- The package sets no minimum platform unless you pass
  [`platforms`](crate::generation::swift::Installer::platforms).
- This crate's tests build installer output with
  `swift build --disable-index-store` (`tests/swift_generation.rs`), and run
  generated code against Rust's bytes and JSON with `swift run`
  (`tests/swift_runtime.rs`). The runtime's own tests run with `swift test` in
  `runtime/swift`.
