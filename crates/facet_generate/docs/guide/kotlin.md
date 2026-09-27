# Kotlin

The Kotlin installer writes a Gradle project: a `build.gradle.kts` build
script, one Kotlin package per namespace under the root package, and, when a
plugin needs one, the serialization runtime in `com.novi.serde` and
`com.novi.bincode`.

Unlike the other installers' output, the project does not build with
`gradle build` as it is written: the sources sit at the project root rather
than in `src/main/kotlin`, and the build script sets no JVM target. Move the
sources, or copy them into an existing Gradle module, as [What gets
written](#what-gets-written) shows
([#241](https://github.com/redbadger/facet-generate/issues/241)).

## Quick start

Reflect the types into a [`Registry`](crate::Registry), then hand it to
[`kotlin::Installer`](crate::generation::kotlin::Installer):

**Rust**
```rust
use facet::Facet;
use facet_generate as fg;
use facet_generate::{
    generation::{bincode::BincodePlugin, json::JsonPlugin, kotlin},
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
let out_dir = std::env::temp_dir().join("facet_generate_guide_kotlin");

kotlin::Installer::new("com.example.drawing", &out_dir)
    .plugin(BincodePlugin)
    .plugin(JsonPlugin)
    .generate(&registry)?;

assert!(out_dir.join("build.gradle.kts").exists());
assert!(out_dir.join("com/example/drawing/geometry/Geometry.kt").exists());
# Ok(())
# }
```

The `fg::` attributes expand to paths in the `facet-generate-attrs` crate, so a
crate that uses them depends on it as well as on `facet` and `facet_generate`:
`cargo add facet facet_generate facet-generate-attrs`.

The installer is a builder. Everything on it is optional apart from the two
arguments to [`new`](crate::generation::kotlin::Installer::new):

| Method | What it does |
|---|---|
| [`new(package_name, dir)`](crate::generation::kotlin::Installer::new) | Names the root package, dot-separated (`com.example.drawing`), and the directory the project is written to. |
| [`plugin`](crate::generation::kotlin::Installer::plugin) | Adds an [`EmitterPlugin`](crate::generation::plugin::EmitterPlugin), such as [`BincodePlugin`](crate::generation::bincode::BincodePlugin) or [`JsonPlugin`](crate::generation::json::JsonPlugin). Call it once per plugin; they run in the order added. Without one, only the type declarations are written. |
| [`external_packages`](crate::generation::kotlin::Installer::external_packages) | Takes namespaces from packages generated separately; see [External packages](#external-packages). |
| [`generate`](crate::generation::kotlin::Installer::generate) | Checks the namespaces, then writes the runtime, the sources and the build script. |

[`make_manifest`](crate::generation::kotlin::Installer::make_manifest) returns
the text of `build.gradle.kts` without writing anything.

## What gets written

The example above, with only `BincodePlugin`, writes:

**Directory tree**
```text
drawing/
├── build.gradle.kts
└── com/
    ├── example/drawing/
    │   ├── Drawing.kt           Drawing, Shape (the root namespace)
    │   └── geometry/
    │       └── Geometry.kt      Point (namespace "geometry")
    └── novi/                    the runtime
        ├── bincode/
        │   ├── BincodeDeserializer.kt
        │   └── BincodeSerializer.kt
        └── serde/
            ├── BinaryDeserializer.kt
            ├── BinarySerializer.kt
            ├── Bytes.kt
            ├── DeserializationError.kt
            ├── Deserializer.kt
            ├── Int128.kt
            ├── SerdeByteArrayOutput.kt
            ├── SerializationError.kt
            ├── Serializer.kt
            ├── Slice.kt
            ├── Tuple4.kt … Tuple12.kt
            └── UInt128.kt
```

and this build script:

**Kotlin** (`build.gradle.kts`)
```kotlin
plugins {
    kotlin("jvm") version "2.2.0"
    kotlin("plugin.serialization") version "2.2.0"
    `java-library`
}

group = "com.example.drawing"
version = "1.0.0"

repositories {
    mavenCentral()
}

dependencies {}

tasks.withType<Jar> {
    manifest {
        attributes["Implementation-Title"] = "com.example.drawing"
        attributes["Implementation-Version"] = "1.0.0"
    }
}
```

The script applies the Kotlin JVM and kotlinx.serialization Gradle plugins,
version 2.2.0, sets `group` to the root package and `version` to `1.0.0`, and
resolves from Maven Central. With `JsonPlugin` its `dependencies` block gains
`implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.9.0")`;
external packages add entries too.

The sources are written at the project root, in package directories, but a
Gradle Kotlin project compiles `src/main/kotlin`. Before running
`gradle build`, move everything except `build.gradle.kts` into
`src/main/kotlin/`, or copy the package directories into an existing source
set, as Crux does. The script sets no
JVM target either: on a JDK newer than the Kotlin compiler supports, pin one, as
this crate's tests do
([#241](https://github.com/redbadger/facet-generate/issues/241)):

**Kotlin** (`build.gradle.kts`)
```kotlin
java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

kotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }
}
```

## Runtimes

| Plugins | Runtime files |
|---|---|
| none | none |
| [`BincodePlugin`](crate::generation::bincode::BincodePlugin) | `com/novi/serde/` (the serializer interfaces, `Bytes`, `Int128`, `UInt128`, `Tuple4` to `Tuple12`, the errors) and `com/novi/bincode/` (`BincodeSerializer`, `BincodeDeserializer`) |
| [`JsonPlugin`](crate::generation::json::JsonPlugin) | `com/novi/serde/`, plus `com/novi/serde/JsonCoding.kt`, which needs kotlinx.serialization |
| both | the union of the two |

The generated types refer to some runtime types whichever plugin is used:
`Bytes` for a `#[facet(fg::bytes)]` field and `Tuple4` to `Tuple12` for longer
tuples. Without a plugin no runtime is written, so a registry holding either
does not compile on its own.

When an external package provides the namespace `serde`, no runtime is
written, and the generated code imports the runtime from the package given by
that entry's [`PackageLocation::Path`](crate::generation::PackageLocation::Path)
instead of `com.novi.serde`.
An external package for the namespace `bincode` does the same for the
`com.novi.bincode` imports, though the bincode runtime is still written unless
`serde` is external too.

## Namespaces

Each namespace set with `#[facet(fg::namespace = "...")]` becomes the package
`<root package>.<namespace>`, keeping the namespace's spelling, written to
`<package path>/<Namespace>.kt`: namespace `geometry` under
`com.example.drawing` becomes `com.example.drawing.geometry`, in
`com/example/drawing/geometry/Geometry.kt`. Types in the root namespace go in
the root package, in a file named after its last segment.

Every type reference is written fully qualified, in the module's own package
too, so no imports between modules are needed:

**Kotlin** (`com/example/drawing/Drawing.kt`)
```kotlin
package com.example.drawing

sealed interface Shape {
    data class Circle(
        val centre: com.example.drawing.geometry.Point,
        val radius: Double,
    ) : Shape
    // …
}
```

Before writing anything, the installer rejects namespaces that would not
compile or would overwrite each other. Each error names both sides and says
what to change:

- a namespace spelled like a root type, whose class would have the same fully
  qualified name as the namespace's package:

  **Error message**
  ```text
  Kotlin: namespace "Kit" becomes the package `com.x.Kit`, the same as type `Kit` in the root namespace, the class `com.x.Kit`. Rename the type with `#[facet(rename = "...")]` or choose a different namespace
  ```

  A namespace that differs from a type only in case (`kv` beside `Kv`) is fine:
  Kotlin names are case-sensitive.
- two namespaces whose files differ only in case (`kv` and `Kv`), which are one
  file on a case-insensitive file system;
- a type named like the first segment of a qualified reference in its module,
  such as a root type `Example` in root package `Example`, which Kotlin would
  look the rest of the reference up in.

A type holding a tuple of more than twelve elements is rejected too, as the
runtime stops at `Tuple12`. So is a type named like something the generated
code imports, for example a type called `Serializer` beside `BincodePlugin`:

**Error message**
```text
Kotlin: type `Serializer` collides with the `Serializer` import used by the generated code; rename it with #[facet(rename = "...")]
```

## External packages

An [`ExternalPackage`](crate::generation::ExternalPackage) says that a
namespace's types come from another package, generated separately. The
installer skips that namespace's module and adds a dependency to the build
script. How the types are referred to depends on the
[`PackageLocation`](crate::generation::PackageLocation):

| Location | References | `dependencies` entry |
|---|---|---|
| `Path("com.example.shapes")` | `com.example.shapes.geometry.Point` | `implementation(files("com.example.shapes"))` |
| `Url("https://repo.example.com/com.example:geometry")`, `version: Some("2.0.0")` | `com.example.drawing.geometry.Point`, as if generated here | `implementation("com.example:geometry:2.0.0")` |

With a `Path`, the value is the package that the external namespace's package
sits under: generate the other project with that as its root package, and its
`geometry` types land in `com.example.shapes.geometry`. The build script lists
the same value as a file dependency, which Gradle does not resolve to that
project's classes, so put them on the classpath yourself, for example with a
project dependency. With a `Url`, the dependency is the URL's last segment
followed by the version (`1.0.0` when there is none), and the references
assume the other project was generated with the same root package as this
one. `module_name` is not used by Kotlin. Using a `Path` as both a package and
a file, and a `Url` that does not affect the references, are tracked in
[#241](https://github.com/redbadger/facet-generate/issues/241).

## Using the generated code

With both plugins, a value round-trips through bincode, and through
kotlinx.serialization's `Json` in the format that `serde_json` writes:

**Kotlin**
```kotlin
import com.example.drawing.Drawing
import com.example.drawing.Shape
import com.example.drawing.geometry.Point
import kotlinx.serialization.json.Json

val drawing = Drawing(
    name = "sketch",
    shapes = listOf(
        Shape.Circle(centre = Point(x = 0.0, y = 0.0), radius = 1.5),
        Shape.Label("hello"),
        Shape.Empty,
    ),
)

// bincode
val bytes: ByteArray = drawing.bincodeSerialize()
check(Drawing.bincodeDeserialize(bytes) == drawing)

// JSON
val json = Json { ignoreUnknownKeys = true }
val text: String = json.encodeToString(Drawing.serializer(), drawing)
check(json.decodeFromString(Drawing.serializer(), text) == drawing)
```

`bincodeDeserialize` throws `DeserializationError` when bytes are left over
after the value. `JsonPlugin` generates no helpers of its own in Kotlin: the
types are `@Serializable`, and you configure `Json` as your code needs. serde
ignores unknown keys, so `ignoreUnknownKeys = true` reads what a newer Rust
side writes. [Serialization: bincode and JSON](crate::guide::serialization)
covers what the two formats look like on the wire.

## Naming

Type names are kept as they are in the registry, after any
`#[facet(rename = "...")]`, and so are the classes of an enum's variants,
referred to as `Shape.Circle`. Properties are `lowerCamelCase`: `hex_code`
becomes `hexCode`. The constants of an enum whose variants are all unit
variants are upper-cased: `DarkRed` becomes `DARKRED`. With `JsonPlugin`,
`@SerialName` keeps the Rust name on the wire for both
(`@SerialName("hex_code") val hexCode`, `@SerialName("DarkRed") DARKRED`).

A variant or field renamed to a name that is not a Kotlin identifier, such as
`#[facet(rename = "on-hold")]` or `"2nd"`, is rewritten: each character an
identifier cannot hold becomes `_`, and a leading digit gets a `_` in front.
So `on-hold` is the constant `ON_HOLD`, or the variant class `on_hold`, and a
field `2nd` is the property `_2nd`. The wire name is still the rename, in
`@SerialName` and in bincode's and JSON's generated code. Two variants of one
enum, or two fields of one struct or struct variant, that end up with the same
identifier are rejected before anything is written:

**Error message**
```text
Kotlin: variants `on-hold` and `on_hold` of `Status` would both become `ON_HOLD`; rename one of them with #[facet(rename = "...")]
```

A property or a variant class named like a Kotlin hard keyword is escaped with
backticks: ``val `in`: Boolean``, ``val `class`: Boolean``,
``data object `in` ``. Soft and modifier keywords such as `value`, `field`,
`data` and `get` are legal identifiers and left alone, as is `default`, which
is not a keyword in Kotlin.

A type named like a prelude type the generated code writes bare (`Set`, `List`,
`Map`, `String`, `Int`, `Pair`, …) shadows it for the whole package, so the
generated code writes the builtin fully qualified: `kotlin.collections.Set<T>`,
`kotlin.String`.

[Plugins](crate::guide::plugins) that write Kotlin should name things the same way, through
[`kotlin::field_name`](crate::generation::kotlin::field_name),
[`kotlin::variant_class_name`](crate::generation::kotlin::variant_class_name),
[`kotlin::enum_constant_name`](crate::generation::kotlin::enum_constant_name)
and [`kotlin::escape_identifier`](crate::generation::kotlin::escape_identifier).

## Language-specific behaviour

- **Structs.** A struct becomes a `data class` with `val` properties, and a
  unit struct a `data object`.
- **Enums.** An enum whose variants are all unit variants becomes an
  `enum class`. Any other enum becomes a `sealed interface`, with a
  `data class` for each variant that carries data and a `data object` for each
  unit variant.
- **Tuples.** A one-element tuple is declared as its element. Two- and
  three-element tuples become `Pair` and `Triple`; four to twelve become the
  runtime's `Tuple4` to `Tuple12`.
- **JSON.** `JsonPlugin` marks each type `@Serializable`, names each property
  and unit-enum constant with `@SerialName`, and adds a nested `JsonSerializer`
  for the types whose JSON the compiler plugin would not write the way Rust
  does, such as enums with data.
- **Bytes.** A field marked `#[facet(fg::bytes)]` becomes the runtime's
  `Bytes`.

The [supported types](crate::guide::supported_types) page lists what every
Rust type becomes.

## Toolchain requirements

- Gradle, with the Kotlin JVM and kotlinx.serialization plugins at 2.2.0,
  which the build script declares.
- With `JsonPlugin`, `kotlinx-serialization-json` 1.9.0 from Maven Central.
- A JDK. The build script sets no JVM target; pin one as shown in [What gets
  written](#what-gets-written) if Gradle reports inconsistent JVM-target
  compatibility.
- The examples on this page were built with Gradle 9.7.1 on JDK 25.
- This crate's tests move the sources into `src/main/kotlin`, pin JVM 17, and
  build with `gradle --configuration-cache build`
  (`tests/kotlin_generation.rs`). The bincode runtime tests compile the
  sources with `kotlinc -include-runtime` and run them with `java`; the JSON
  ones build and run with Gradle, since they need the kotlinx.serialization
  compiler plugin (`tests/kotlin_runtime.rs`).
