# TypeScript

The TypeScript installer writes an npm package: a `package.json`, one ES module
file per namespace, and, when a plugin needs one, the serialization runtime in
`serde/` and `bincode/`. The generated modules import each other and the
runtime without file extensions (`./geometry`, `./serde`).

## Quick start

Reflect the types into a [`Registry`](crate::Registry), then hand it to
[`typescript::Installer`](crate::generation::typescript::Installer):

**Rust**
```rust
use facet::Facet;
use facet_generate as fg;
use facet_generate::{
    generation::{bincode::BincodePlugin, json::JsonPlugin, typescript},
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
let out_dir = std::env::temp_dir().join("facet_generate_guide_typescript");

typescript::Installer::new("drawing", &out_dir)
    .plugin(BincodePlugin)
    .plugin(JsonPlugin)
    .generate(&registry)?;

assert!(out_dir.join("package.json").exists());
assert!(out_dir.join("geometry.ts").exists());
# Ok(())
# }
```

The `fg::` attributes expand to paths in the `facet-generate-attrs` crate, so a
crate that uses them depends on it as well as on `facet` and `facet_generate`:
`cargo add facet facet_generate facet-generate-attrs`.

The installer is a builder. Everything on it is optional apart from the two
arguments to [`new`](crate::generation::typescript::Installer::new):

| Method | What it does |
|---|---|
| [`new(package_name, dir)`](crate::generation::typescript::Installer::new) | Names the package, which is also the root module's file name (`drawing.ts`), and the directory it is written to. |
| [`plugin`](crate::generation::typescript::Installer::plugin) | Adds an [`EmitterPlugin`](crate::generation::plugin::EmitterPlugin), such as [`BincodePlugin`](crate::generation::bincode::BincodePlugin) or [`JsonPlugin`](crate::generation::json::JsonPlugin). Call it once per plugin; they run in the order added. Without one, only the type declarations are written. |
| [`external_packages`](crate::generation::typescript::Installer::external_packages) | Takes namespaces from packages generated separately; see [External packages](#external-packages). |
| [`generate`](crate::generation::typescript::Installer::generate) | Checks the namespaces, then writes the runtime, the modules and `package.json`. |

[`make_manifest`](crate::generation::typescript::Installer::make_manifest)
returns the `package.json` contents, as a `serde_json::Value`, without writing
anything.

## What gets written

The example above, with only `BincodePlugin`, writes:

**Directory tree**
```text
drawing/
├── package.json
├── drawing.ts                   Drawing, Shape (the root namespace)
├── geometry.ts                  Point (namespace "geometry")
├── bincode/                     the runtime
│   ├── bincodeDeserializer.ts
│   ├── bincodeSerializer.ts
│   └── index.ts
└── serde/
    ├── binaryDeserializer.ts
    ├── binarySerializer.ts
    ├── deserializer.ts
    ├── index.ts
    ├── serializer.ts
    └── types.ts
```

and this `package.json`:

**JSON** (`package.json`)
```json
{
  "devDependencies": {
    "typescript": "^5.8.3"
  },
  "name": "drawing",
  "version": "0.1.0"
}
```

The manifest names the package, gives it version `0.1.0`, and lists
`typescript` `^5.8.3` as its one dev dependency. External packages, and any
dependency a plugin declares, go in `dependencies`. There is no `tsconfig.json`,
no `"type"`, and no `main` or `exports` entry: the package is source to compile
or bundle as part of your own project.

## Runtimes

| Plugins | Runtime files |
|---|---|
| none | none |
| [`BincodePlugin`](crate::generation::bincode::BincodePlugin) | `serde/` (the `Serializer` and `Deserializer` interfaces and their binary base classes) and `bincode/` (`BincodeSerializer`, `BincodeDeserializer`) |
| [`JsonPlugin`](crate::generation::json::JsonPlugin) | `serde/json.ts` |
| both | the union of the two |

The generated modules import the bincode interfaces from `./serde` and the JSON
runtime from `./serde/json`. When an external package provides the namespace
`serde`, no runtime is written, and the modules import from that package
instead.

## Namespaces

Each namespace set with `#[facet(fg::namespace = "...")]` becomes a module file
named after the namespace, `<namespace>.ts`, beside the root module
`<package>.ts`. A module that uses another's types imports it whole, under the
namespace's name in `UpperCamelCase`:

**TypeScript** (`drawing.ts`)
```typescript
import type { Serializer, Deserializer } from "./serde";
import * as Geometry from "./geometry";

export type Shape =
    | { kind: "Circle"; centre: Geometry.Point; radius: float64 }
    | { kind: "Label"; value: str }
    | { kind: "Empty" };
```

A namespaced module that uses root types imports the root module the same way.

Before writing anything, the installer rejects namespaces that would not
type-check or would overwrite each other. Each error names both sides and says
what to change:

- two modules whose files differ only in case (`kv.ts` and `Kv.ts`, or `app.ts`
  beside the root package's `App.ts`), which are one file on a case-insensitive
  file system;
- a namespace imported under a name the module already has: a type it declares
  (TS2440), another namespace (`my_ns` and `MyNs`), a name a plugin imports, or
  an alias the module exports:

  **Error message**
  ```text
  TypeScript: namespace "kv" is imported as `Kv` in `Example.ts`, the same as type `Kv` in the root namespace. Rename the type with `#[facet(rename = "...")]` or choose a different namespace
  ```

- a namespace imported under the name of a global that the module constructs,
  which the import would shadow (TS2351):

  **Error message**
  ```text
  TypeScript: namespace "map" is imported as `Map` in `Example.ts`, the same as the global `Map`, which `Example.ts` constructs, so `new Map(...)` would find the namespace instead. Choose a different namespace
  ```

- a namespace written to `serde.ts`, which `./serde` would find instead of the
  runtime in `serde/`.

A type named like something the generated code imports is rejected too, for
example a type called `Serializer` beside `BincodePlugin`:

**Error message**
```text
TypeScript: type `Serializer` collides with the `Serializer` import used by the generated code; rename it with #[facet(rename = "...")]
```

## External packages

An [`ExternalPackage`](crate::generation::ExternalPackage) says that a
namespace's types come from another package, generated separately. The
installer skips that namespace's module, imports it from the package instead of
a sibling file, and adds the package to `dependencies`:

| [`PackageLocation`](crate::generation::PackageLocation) | Import | `dependencies` entry |
|---|---|---|
| `Path("../geometry")` | `import * as Geometry from "geometry"` | `"geometry": "file:../geometry"` |
| `Path("../geometry")`, `module_name: Some("geometry")` | `import * as Geometry from "geometry/geometry"` | `"geometry": "file:../geometry"` |
| `Url("https://registry.npmjs.org/geometry")`, `version: Some("^1.0.0")` | `import * as Geometry from "geometry"` | `"geometry": "^1.0.0"` |

The import names the package after the namespace, and
[`module_name`](crate::generation::ExternalPackage::module_name), for a `Path`,
picks the module inside it. The generated package has no `main` or `exports`,
so point `module_name` at the namespace's file, as in the second row. For a
`Url`, the dependency is named by the URL's last segment, or the last two for a
scoped `@scope/name` package, at [`version`](crate::generation::ExternalPackage::version)
or `*`; `module_name` is not used, and the import still names the namespace,
so the two agree only when the package is named like the namespace
([#242](https://github.com/redbadger/facet-generate/issues/242)).

## Using the generated code

With both plugins, a value round-trips through bincode and through the JSON
that `serde_json` writes:

**TypeScript**
```typescript
import { BincodeDeserializer, BincodeSerializer } from "./bincode";
import { Drawing, shapeCircle, shapeEmpty, shapeLabel } from "./drawing";
import { Point } from "./geometry";

const drawing = new Drawing("sketch", [
    shapeCircle(new Point(0, 0), 1.5),
    shapeLabel("hello"),
    shapeEmpty(),
]);

// bincode
const serializer = new BincodeSerializer();
drawing.serialize(serializer);
const bytes: Uint8Array = serializer.getBytes();
const decoded = Drawing.deserialize(new BincodeDeserializer(bytes));

// JSON
const json: string = Drawing.jsonSerialize(drawing);
const parsed: Drawing = Drawing.jsonDeserialize(json);
```

A class has `serialize` and a static `deserialize` for bincode, and static
`toJson`, `fromJson`, `jsonSerialize` and `jsonDeserialize` for JSON. An enum is
a type rather than a class, so its helpers are functions beside it:
`serializeShape`, `deserializeShape`, `toJsonShape`, `fromJsonShape`,
`jsonSerializeShape` and `jsonDeserializeShape`. [Serialization: bincode and
JSON](crate::guide::serialization) covers what the two formats look like on the
wire.

## Naming

Type names are kept as they are in the registry, after any
`#[facet(rename = "...")]`. Properties keep the Rust field names too, so a field
`hex_code` is `hex_code` in TypeScript, and so on the wire. An enum's variants
are its `kind` strings, spelled as in Rust (`"Circle"`); the constructor
functions are named after the enum in `lowerCamelCase` followed by the variant
(`shapeCircle`), and the payload of a newtype variant is its `value`.

Reserved words are legal property names but not parameter names, so a field
named like one keeps its name as a property and gets a constructor parameter
with a trailing underscore: a field `default` is `this.default = default_`. A
character that cannot appear in an identifier becomes an underscore in the
parameter (`with-dash` becomes `with_dash`).

A type named like a global the generated code writes (`Map`, `Error`,
`Uint8Array`) shadows it for the whole module, so the generated code reaches
the global through `globalThis`, such as `globalThis.Map<K, V>`.

[Plugins](crate::guide::plugins) that write TypeScript should name bindings the same way, through
[`typescript::param_name`](crate::generation::typescript::param_name) and
[`typescript::is_reserved_word`](crate::generation::typescript::is_reserved_word).

## Language-specific behaviour

- **Structs.** A struct becomes an `export class` whose constructor declares
  its public properties.
- **Enums.** Every enum becomes a discriminated union on `kind`, with an
  exported constructor function per variant and an exhaustive
  `matchShape(value, { Circle: …, Label: …, Empty: … })`.
- **Type aliases.** Each module declares the aliases it uses: `float64` and
  the other number types for `number`, `int64`, `uint64`, `int128` and
  `uint128` for `bigint`, `str` and `char` for `string`, `bytes` for
  `Uint8Array`, `Optional<T>` for `T | null`, `Seq<T>` for `T[]`, and a branded
  `Uuid` string.
- **Bytes.** A field marked `#[facet(fg::bytes)]` becomes `bytes`, a
  `Uint8Array`.

The [supported types](crate::guide::supported_types) page lists what every
Rust type becomes.

## Toolchain requirements

- TypeScript 5.8, as the manifest's dev dependency asks for.
- A module resolver that accepts extensionless relative imports, such as a
  bundler, `"moduleResolution": "bundler"`, or Deno's `--sloppy-imports`.
- `bigint`, for 64- and 128-bit integers, so an ES2020 or later target.
- With `JsonPlugin`, 64- and 128-bit integers beyond 2^53 are read and written
  exactly through `JSON.rawJSON` and the `JSON.parse` reviver's
  `context.source`, which need Node 21+, Deno 1.37+, Chrome 114+, Safari 18.4+
  or Firefox 135+. On an older engine only such an integer throws.
- The examples on this page were run with Deno 2.9.6.
- This crate's tests type-check generated code with
  `deno check --sloppy-imports` (`tests/typescript_generation.rs`), and run it
  against Rust's bytes and JSON with `deno test --sloppy-imports`
  (`tests/typescript_runtime.rs`).
