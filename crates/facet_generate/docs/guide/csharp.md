# C#

The C# installer writes an SDK-style .NET project: a `.csproj` file, one
file-scoped C# namespace per namespace under the root namespace, and the
runtime under `Facet/Runtime/`. The project builds with `dotnet build` as it is
written.

## Quick start

Reflect the types into a [`Registry`](crate::Registry), then hand it to
[`csharp::Installer`](crate::generation::csharp::Installer):

**Rust**
```rust
use facet::Facet;
use facet_generate as fg;
use facet_generate::{
    generation::{bincode::BincodePlugin, csharp, json::JsonPlugin},
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
let out_dir = std::env::temp_dir().join("facet_generate_guide_csharp");

csharp::Installer::new("Example.Drawing", &out_dir)
    .plugin(BincodePlugin)
    .plugin(JsonPlugin)
    .generate(&registry)?;

assert!(out_dir.join("Example.Drawing.csproj").exists());
assert!(out_dir.join("Example/Drawing/geometry/Geometry.cs").exists());
# Ok(())
# }
```

The `fg::` attributes expand to paths in the `facet-generate-attrs` crate, so a
crate that uses them depends on it as well as on `facet` and `facet_generate`:
`cargo add facet facet_generate facet-generate-attrs`.

The installer is a builder. Everything on it is optional apart from the two
arguments to [`new`](crate::generation::csharp::Installer::new):

| Method | What it does |
|---|---|
| [`new(package_name, dir)`](crate::generation::csharp::Installer::new) | Names the root namespace, dot-separated (`Example.Drawing`), which also names the `.csproj` file, and the directory the project is written to. |
| [`plugin`](crate::generation::csharp::Installer::plugin) | Adds an [`EmitterPlugin`](crate::generation::plugin::EmitterPlugin), such as [`BincodePlugin`](crate::generation::bincode::BincodePlugin) or [`JsonPlugin`](crate::generation::json::JsonPlugin). Call it once per plugin. Without one, only the type declarations are written. |
| [`external_packages`](crate::generation::csharp::Installer::external_packages) | Takes namespaces from projects or NuGet packages generated separately; see [External packages](#external-packages). |
| [`generate`](crate::generation::csharp::Installer::generate) | Checks the namespaces, then writes the runtime, the sources and the project file. |

[`make_manifest`](crate::generation::csharp::Installer::make_manifest) returns
the text of the `.csproj` file without writing anything.

## What gets written

The example above, with only `BincodePlugin`, writes:

**Directory tree**
```text
Drawing/
├── Example.Drawing.csproj
├── Example/Drawing/
│   ├── Drawing.cs               Drawing, Shape (the root namespace)
│   └── geometry/
│       └── Geometry.cs          Point (namespace "geometry")
└── Facet/Runtime/               the runtime
    ├── Bincode/
    │   ├── BincodeDeserializer.cs
    │   ├── BincodeSerializer.cs
    │   ├── FacetHelpers.cs
    │   ├── IFacetDeserializable.cs
    │   └── IFacetSerializable.cs
    └── Serde/
        ├── DeserializationError.cs
        ├── IDeserializer.cs
        ├── ISerializer.cs
        ├── SerializationError.cs
        └── Unit.cs
```

and this project file:

**XML** (`Example.Drawing.csproj`)
```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <RootNamespace>Example.Drawing</RootNamespace>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="CommunityToolkit.Mvvm" Version="8.4.0" />
  </ItemGroup>
</Project>
```

The project targets `net10.0` with implicit usings and nullable reference
types enabled, and sets `RootNamespace` to the package name. Its one package
reference, `CommunityToolkit.Mvvm` 8.4.0, provides the `ObservableObject` base
class and the `[ObservableProperty]` source generator that the generated
classes use. System.Text.Json, which `JsonPlugin`'s code uses, comes with .NET.
External packages add references of their own.

## Runtimes

The runtime is written under `Facet/Runtime/`, in the namespaces
`Facet.Runtime.Serde`, `Facet.Runtime.Bincode` and `Facet.Runtime.Json`.

| Plugins | Runtime files |
|---|---|
| none | `Serde/Unit.cs` |
| [`BincodePlugin`](crate::generation::bincode::BincodePlugin) | `Serde/` (`Unit`, `ISerializer`, `IDeserializer`, the errors) and `Bincode/` (`BincodeSerializer`, `BincodeDeserializer`, `IFacetSerializable`, `IFacetDeserializable`, `FacetHelpers`) |
| [`JsonPlugin`](crate::generation::json::JsonPlugin) | `Serde/` and `Json/` (`JsonSerde`, `FacetJson`) |
| both | the union of the two |

`Unit.cs` is written even without a plugin, since Rust's `()` becomes the
`Unit` type it declares. The C# installer writes the runtime whatever external
packages are configured; unlike the other languages, it does not take it from
an external package for the namespace `serde`
([#232](https://github.com/redbadger/facet-generate/issues/232)).

## Namespaces

Each namespace set with `#[facet(fg::namespace = "...")]` becomes the C#
namespace `<RootNamespace>.<Namespace>`, each segment in `UpperCamelCase`,
declared file-scoped in `<package path>/<namespace>/<Namespace>.cs`: namespace
`geometry` under `Example.Drawing` becomes `Example.Drawing.Geometry`, in
`Example/Drawing/geometry/Geometry.cs`. The directory keeps the namespace's
spelling; the file is named in `UpperCamelCase`. Types in the root namespace go
in the root namespace, in a file named after its last segment.

Type references to other modules and to the root namespace are written fully
qualified, so no `using` directives are needed between modules:

**C#** (`Example/Drawing/Drawing.cs`)
```csharp
namespace Example.Drawing;

public abstract record Shape : IFacetSerializable, IFacetDeserializable<Shape> {
    public sealed partial record Circle(Example.Drawing.Geometry.Point Centre, double Radius) : Shape;
    public sealed partial record Label(string Value) : Shape;
    public sealed partial record Empty() : Shape;
    // …
}
```

Every generated namespace is nested in the root namespace, so before writing
anything the installer rejects namespaces that would not compile or would
overwrite each other. Each error names both sides and says what to change:

- a namespace that becomes the name of a root type, which the root namespace
  cannot hold both of (CS0101):

  **Error message**
  ```text
  C#: namespace "kv" becomes `Example.Kv`, the same as type `Kv` in the root namespace. Rename the type with `#[facet(rename = "...")]` or choose a different namespace
  ```

- a namespace that becomes the name of a builtin the generated code writes
  unqualified, when the registry uses it:

  **Error message**
  ```text
  C#: namespace "unit" becomes `Example.Unit`, the same as the builtin `Unit`, which the generated code writes unqualified, so every module in `Example` would find the namespace instead. Choose a different namespace
  ```

- two namespaces whose files differ only in case (`kv` and `Kv`), which are one
  file on a case-insensitive file system;
- a qualified reference whose first segment names a type or namespace in scope,
  which C# would look the rest of the reference up in:

  **Error message**
  ```text
  C#: `App` refers to type `Inner` in namespace "app" as `App.App.Inner`, whose first segment is `App`, the same as namespace "app", which becomes `App.App`. Choose a different namespace or package name
  ```

A type named like one the generated code declares or uses (`Enum`, `Facet`,
`BincodeSerializer`, `ISerializer`, …) is rejected too.

## External packages

An [`ExternalPackage`](crate::generation::ExternalPackage) says that a
namespace's types come from another project, generated separately. The
installer skips that namespace's module and adds a reference to the project
file. The references to its types are written as if it had been generated
here, `<RootNamespace>.<Namespace>.Type`, so generate the other project with
the same root namespace.

| [`PackageLocation`](crate::generation::PackageLocation) | Project file entry |
|---|---|
| `Path("../Geometry/Geometry.csproj")` | `<ProjectReference Include="../Geometry/Geometry.csproj" />` |
| `Url("https://www.nuget.org/packages/Example.Geometry/1.4.0")` | `<PackageReference Include="Example.Geometry" Version="1.4.0" />` |

For a `Url`, the NuGet package ID is
[`module_name`](crate::generation::ExternalPackage::module_name) when it is
set, and otherwise read from the URL: a `.nupkg` file name, the segment after
`packages/` or `package/` (as on nuget.org and in the v2 API), or the last
segment. The version is [`version`](crate::generation::ExternalPackage::version)
when set, then the version in the URL, then `1.0.0`.

Two things stand in the way of referencing a separately generated project
today. Both projects share a root namespace, so both project files get the same
name, which NuGet rejects as an ambiguous project name until one is renamed.
And each project carries its own copy of the runtime, in the same namespaces.
Without a plugin the pair builds, but with one the compiler sees two
`ISerializer` types and the build fails (CS1503)
([#240](https://github.com/redbadger/facet-generate/issues/240)).

## Using the generated code

With both plugins, a value round-trips through bincode and through the JSON
that `serde_json` writes:

**C#**
```csharp
using System.Collections.ObjectModel;
using Example.Drawing;
using Example.Drawing.Geometry;

var drawing = new Drawing {
    Name = "sketch",
    Shapes = new ObservableCollection<Shape> {
        new Shape.Circle(new Point { X = 0, Y = 0 }, 1.5),
        new Shape.Label("hello"),
        new Shape.Empty(),
    },
};

// bincode
byte[] bytes = drawing.BincodeSerialize();
Drawing decoded = Drawing.BincodeDeserialize(bytes);

// JSON, through System.Text.Json
string json = drawing.JsonSerialize();
Drawing parsed = Drawing.JsonDeserialize(json);
```

`BincodeDeserialize` throws `DeserializationError` when bytes are left over
after the value. Each type also has a `[JsonConverter]`, so
`JsonSerializer.Serialize` and `JsonSerializer.Deserialize<T>` write and read
the same JSON. The variants of an enum are records, with value equality; the
classes are `ObservableObject`s, which compare by reference. [Serialization:
bincode and JSON](crate::guide::serialization) covers what the two formats look
like on the wire.

## Naming

Type names are kept as they are in the registry, after any
`#[facet(rename = "...")]`. A field becomes a private `_lowerCamelCase` field
marked `[ObservableProperty]`, from which the MVVM toolkit generates the
`UpperCamelCase` property: `hex_code` becomes `_hexCode` and `HexCode`. The
positional parameters of a variant's record are `UpperCamelCase` (`Centre`,
`Value`), and the members of an enum whose variants are all unit variants keep
the variant names (`DarkRed`). With `JsonPlugin`, `[JsonPropertyName]` and the
converters keep the Rust names on the wire.

A variant or field renamed to a name that starts with a digit, such as
`#[facet(rename = "2fa")]`, cannot be written as a C# identifier, and
generation fails:

**Error message**
```text
C#: field `2nd` of `Person` would become `2nd`, which is not a valid identifier because it starts with a digit; rename it with #[facet(rename = "...")] to a name that starts with a letter
```

This check runs as each module is generated, so the runtime may already be on
disk ([#250](https://github.com/redbadger/facet-generate/issues/250)). A hyphen
or a space in a rename is dropped by the casing: `on-hold` becomes the record
`OnHold`, and `first-name` the property `FirstName`.

A local or parameter named like a C# keyword is escaped as a verbatim
identifier: the deserializer's local for a field `class` is `var @class`.
Contextual keywords are valid there and left alone.

A type named like a builtin the generated code writes through a `using`
directive (`HashSet`, `Dictionary`, `ObservableCollection`, `Guid`, `Int128`,
`UInt128`, `Unit`) shadows it in its namespace, so the generated code writes the
builtin through its `global::` name, such as
`global::System.Collections.Generic.HashSet`. Lower-case keyword types (`int`,
`string`) cannot be shadowed.

[Plugins](crate::guide::plugins) that write C# should escape names the same way, through
[`csharp::escape_identifier`](crate::generation::csharp::escape_identifier).

## Language-specific behaviour

- **Classes.** A struct becomes a `public partial class` deriving from
  `ObservableObject`, with an `[ObservableProperty]` field per Rust field, so
  it raises `PropertyChanged` for MVVM data binding. A `Vec<T>` becomes an
  `ObservableCollection<T>`. The classes are `partial` for the toolkit's source
  generator.
- **Enums.** An enum whose variants are all unit variants becomes a
  `public enum`. Any other enum becomes a `public abstract record` with a
  `public sealed partial record` nested inside it for each variant. A unit
  struct becomes a sealed record too.
- **Bincode.** `BincodePlugin` implements `IFacetSerializable` and
  `IFacetDeserializable<T>`, with `Serialize` / `Deserialize` and the
  `BincodeSerialize` / `BincodeDeserialize` wrappers. The helpers for
  collections, maps and options live once, in `FacetHelpers.cs`, rather than in
  each module.
- **JSON.** `JsonPlugin` writes a `JsonConverter` for each type, attaches it
  with `[JsonConverter]`, and adds `JsonSerialize` / `JsonDeserialize`.
- **Tuples.** A tuple becomes a `ValueTuple`, `(A, B)`, and a one-element
  tuple is declared as its element. With `JsonPlugin` a one-element tuple is
  still written as a one-element array, `[x]`, as `serde_json` writes it.
- **Nested options.** C# cannot declare an `Option<Option<T>>`: `T??` is not a
  type, and `Nullable<T>` cannot nest. Generation fails before anything is
  written:

  **Error message**
  ```text
  C#: type `Nested` in the root namespace holds an `Option<Option<T>>`, which C# cannot declare (`T??`); wrap the inner option in a struct or an enum
  ```

- **Bytes.** A field marked `#[facet(fg::bytes)]` becomes `byte[]`.

The [supported types](crate::guide::supported_types) page lists what every
Rust type becomes.

## Toolchain requirements

- The .NET 10 SDK, for `net10.0`.
- `CommunityToolkit.Mvvm` 8.4.0, restored from NuGet on the first build.
- The examples on this page were built with the .NET SDK 10.0.401.
- This crate's tests build installer output with `dotnet build`
  (`tests/csharp_generation.rs`), and run generated code against Rust's bytes
  and JSON with `dotnet run`, after adding `<OutputType>Exe</OutputType>` to
  the project (`tests/csharp_runtime.rs`).
