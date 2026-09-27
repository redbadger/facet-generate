# Supported types

Reflection turns each Rust type into a language-neutral
[`Format`](crate::reflection::format::Format), and every container (struct or enum) into a
[`ContainerFormat`](crate::reflection::format::ContainerFormat) entry in the
[`Registry`](crate::Registry). Each generator then maps those formats to native types. This page
lists, for each Rust type, what it becomes in Swift, Kotlin, C# and TypeScript, and what is
rejected or left out.

The declared types are the same whether you generate with no plugin, with
[`BincodePlugin`](crate::generation::bincode::BincodePlugin) or with
[`JsonPlugin`](crate::generation::json::JsonPlugin), except where a footnote says otherwise. The
plugins add conformances, annotations and serialization code around the declarations, which the
[serialization guide](crate::guide::serialization) and the language pages cover.

Only the root types need registering; everything they reach is reflected with them:

**Rust**
```rust
use std::collections::BTreeMap;

use facet::Facet;
use facet_generate::reflection::{
    RegistryBuilder,
    format::{ContainerFormat, Format, QualifiedTypeName},
};

#[derive(Facet)]
struct Order {
    id: u64,
    note: Option<String>,
    lines: Vec<Line>,
    totals: BTreeMap<String, f64>,
    status: Status,
    #[facet(skip)]
    cache: u32,
}

#[derive(Facet)]
struct Line {
    sku: String,
    quantity: u32,
}

#[derive(Facet)]
#[repr(C)]
enum Status {
    Open,
    Shipped { tracking: String },
}

let registry = RegistryBuilder::new().add_type::<Order>()?.build()?;

// `Order`, `Line` and `Status`
assert_eq!(registry.len(), 3);

let order = &registry[&QualifiedTypeName::root("Order".to_string())];
let ContainerFormat::Struct(fields, _) = order else {
    panic!("expected a struct");
};
// `cache` is skipped
assert_eq!(fields.len(), 5);
assert_eq!(fields[1].value, Format::Option(Box::new(Format::Str)));
# Ok::<(), facet_generate::error::Error>(())
```

Field names are converted to each language's convention: `lowerCamelCase` in Swift and Kotlin,
and in C# a `_camelCase` backing field for a `PascalCase` `[ObservableProperty]` (record
parameters are `PascalCase`). The TypeScript classes keep the Rust names. The tables show the
types only.

## Primitives

TypeScript declares fields with aliases, written at the top of each module; the alias is shown
with the type it stands for.

| Rust | Swift | Kotlin | C# | TypeScript |
|---|---|---|---|---|
| `bool` | `Bool` | `Boolean` | `bool` | `bool` (`boolean`) |
| `i8` | `Int8` | `Byte` | `sbyte` | `int8` (`number`) |
| `i16` | `Int16` | `Short` | `short` | `int16` (`number`) |
| `i32` | `Int32` | `Int` | `int` | `int32` (`number`) |
| `i64` | `Int64` | `Long` | `long` | `int64` (`bigint`) |
| `i128` | `Int128` | `BigInteger` | `Int128` | `int128` (`bigint`) |
| `u8` | `UInt8` | `UByte` | `byte` | `uint8` (`number`) |
| `u16` | `UInt16` | `UShort` | `ushort` | `uint16` (`number`) |
| `u32` | `UInt32` | `UInt` | `uint` | `uint32` (`number`) |
| `u64` | `UInt64` | `ULong` | `ulong` | `uint64` (`bigint`) |
| `u128` | `UInt128` | `BigInteger` | `UInt128` | `uint128` (`bigint`) |
| `isize`[^size] | `Int64` | `Long` | `long` | `int64` (`bigint`) |
| `usize`[^size] | `UInt64` | `ULong` | `ulong` | `uint64` (`bigint`) |
| `f32` | `Float` | `Float` | `float` | `float32` (`number`) |
| `f64` | `Double` | `Double` | `double` | `float64` (`number`) |
| `char` | `Character` | `String` | `string` | `char` (`string`) |
| `()` | `Void` | `Unit` | `Unit`[^unit] | `unit` (`null`) |

[^size]: `isize` and `usize` take their width from the layout of the target the generator runs
    on: `i64`/`u64` on a 64-bit host, as shown, and `i32`/`u32` on a 32-bit one.

[^unit]: `Facet.Runtime.Serde.Unit`, a file the C# installer writes with or without a plugin.

## Strings, UUIDs and dates

| Rust | Swift | Kotlin | C# | TypeScript |
|---|---|---|---|---|
| `String`, `&str`, `Box<str>`, `Arc<str>`, `Cow<str>` | `String` | `String` | `string` | `str` (`string`) |
| `uuid::Uuid` | `UUID` | `UUID`[^uuid] | `Guid` | `Uuid` |
| `chrono::DateTime<Utc>` | `String` | `String` | `string` | `str` (`string`) |

In TypeScript `Uuid` is a branded string, declared in the module as
`string & { readonly __uuid: unique symbol }`.

`Uuid` and `DateTime<Utc>` need facet's `uuid` and `chrono` features. No other `chrono` type is
supported (see [Rejected when reflecting](#rejected-when-reflecting)).

[^uuid]: `java.util.UUID`. With `JsonPlugin` the module declares
    `typealias UUID = @Serializable(with = UUIDSerializer::class) java.util.UUID` and uses that.

## Bytes

A byte field is a sequence of `u8` unless it carries `#[facet(fg::bytes)]`, which gives it the
language's byte-array type. The attribute applies to `Vec<u8>`, `&[u8]`, `[u8; N]` and
`bytes::Bytes` (facet's `bytes` feature), and to an `Option` of any of them.

| Rust | Swift | Kotlin | C# | TypeScript |
|---|---|---|---|---|
| `#[facet(fg::bytes)]` field | `[UInt8]` | `Bytes`[^bytes] | `byte[]` | `bytes` (`Uint8Array`) |
| `Vec<u8>`, `bytes::Bytes` without the attribute | `[UInt8]` | `List<UByte>` | `ObservableCollection<byte>` | `Seq<uint8>` |

[^bytes]: `com.novi.serde.Bytes`, from the serde runtime the Kotlin installer writes with a
    plugin. With `JsonPlugin` the module declares
    `typealias Bytes = @Serializable(with = BytesSerializer::class) com.novi.serde.Bytes`. With no
    plugin there is no runtime and the output does not compile (see
    [Output that does not compile](#output-that-does-not-compile)).

## Collections and pointers

In TypeScript, `Optional<T>` is `T | null`, `Seq<T>` is `T[]`, and `ListTuple<[T]>` is `[T][]`,
an array of one-element tuples.

| Rust | Swift | Kotlin | C# | TypeScript |
|---|---|---|---|---|
| `Option<T>` | `T?` | `T? = null` | `T?` | `Optional<T>` |
| `Vec<T>`, `&[T]`, `Box<[T]>` | `[T]` | `List<T>` | `ObservableCollection<T>` | `Seq<T>` |
| `[T; N]` | `[T]` | `List<T>` | `T[]` | `ListTuple<[T]>` |
| `HashSet<T>`, `BTreeSet<T>` | `Set<T>`[^hashable] | `Set<T>` | `HashSet<T>` | `Seq<T>` |
| `HashMap<K, V>`, `BTreeMap<K, V>` | `[K: V]`[^hashable] | `Map<K, V>` | `Dictionary<K, V>` | `Map<K, V>` |
| `Box<T>`, `Rc<T>`, `Arc<T>` | `T` | `T` | `T` | `T` |

The reflected format of a fixed-size array records its length, but the declared type does not. A
TypeScript set is an array.

`Option<Option<T>>` is declared as `T??` in Swift and Kotlin, and `Optional<Optional<T>>` in
TypeScript. C# cannot declare it, and generation fails (see
[Rejected when generating](#rejected-when-generating)). Kotlin's `T??` is `T?`, and TypeScript's
type is `T | null`, so neither can tell `None` from `Some(None)`, and a bincode `Some(None)`
reads back as `None` ([#249](https://github.com/redbadger/facet-generate/issues/249)). Swift's
`T??` keeps the difference. JSON cannot hold it in any language: `serde_json` writes both as
`null`.

`VecDeque`, `LinkedList` and `BinaryHeap` do not implement `Facet` (in facet 0.46), so they cannot
be used.

[^hashable]: The element or key type must be `Hashable` in Swift. `()`, a tuple of two or more
    elements, and any struct or enum that holds either are not, and generation fails (see
    [Rejected when generating](#rejected-when-generating)).

## Tuples

| Rust | Swift | Kotlin | C# | TypeScript |
|---|---|---|---|---|
| `(A,)` | `A` | `A` | `A` | `Tuple<[A]>` |
| `(A, B)` | `(A, B)` | `Pair<A, B>` | `(A, B)` | `Tuple<[A, B]>` |
| `(A, B, C)` | `(A, B, C)` | `Triple<A, B, C>` | `(A, B, C)` | `Tuple<[A, B, C]>` |
| `(A, …, L)`, 4 to 12 elements | `(A, …, L)` | `Tuple4<…>` to `Tuple12<…>`[^tuplen] | `(A, …, L)` | `Tuple<[A, …, L]>` |

TypeScript's `Tuple<T>` is `T`, so `Tuple<[A, B]>` is the tuple type `[A, B]`.

Swift, Kotlin and C# declare a one-element tuple as its element. Bincode writes it as the
element, as Rust does. `serde_json` writes it as a one-element array, `[x]`, and so do C# and
TypeScript JSON, but Kotlin and Swift JSON write the bare element
([#248](https://github.com/redbadger/facet-generate/issues/248)).

facet implements `Facet` for tuples of up to 4 elements by default, and up to 12 with its
`tuples-12` feature; there is no implementation for longer tuples. Tuple structs and tuple
variants are not tuples in this sense: they become classes, and can have any number of fields.

[^tuplen]: `com.novi.serde.Tuple4` to `Tuple12`, from the serde runtime the Kotlin installer
    writes with a plugin. With no plugin the output does not compile.

## Structs

| Rust | Swift | Kotlin | C# | TypeScript |
|---|---|---|---|---|
| `struct U;` | `struct` with no fields | `data object` | `sealed record` | `class` with no fields |
| `struct N(u32);` | `struct` with `value` | `data class` with `value` | `partial class` with `_value` | `class` with `value` |
| `struct T(u32, String);` | `struct` with `field0`, `field1` | `data class` with `field0`, `field1` | `partial class` with `_field0`, `_field1` | `class` with `field0`, `field1` |
| `struct S { x: u32 }` | `struct` | `data class` | `partial class` | `class` |

A C# class derives from `ObservableObject` (from `CommunityToolkit.Mvvm`). A struct whose fields
are all skipped is generated as a unit struct.

## Enums

| Rust | Swift | Kotlin | C# | TypeScript |
|---|---|---|---|---|
| Unit variants only | `indirect enum` | `enum class` | `enum` | union of `{ kind: "A" }` objects |
| Any data variants | `indirect enum` with associated values | `sealed interface` | `abstract record` | union on `kind` |

A unit variant is a Kotlin `data object` and a C# `sealed record` with no parameters. A newtype
variant's payload is called `value`, a tuple variant's `field0`, `field1`, …, and a struct
variant keeps its field names. A struct variant whose fields are all skipped is a unit variant.

This enum:

**Rust**
```rust
# use facet::Facet;
#[derive(Facet)]
#[repr(C)]
pub enum Data {
    Unit,
    Newtype(u32),
    Tuple(u32, String),
    Struct { x: u32, y: String },
}
```

is declared as follows (with no plugin; TypeScript also gets a constructor function per variant
and a `matchData` function, not shown).

**Swift**
```swift
indirect public enum Data {
    case unit
    case newtype(UInt32)
    case tuple(UInt32, String)
    case `struct`(x: UInt32, y: String)
}
```

**Kotlin**
```kotlin
sealed interface Data {
    data object Unit: Data

    data class Newtype(
        val value: UInt,
    ) : Data

    data class Tuple(
        val field0: UInt,
        val field1: String,
    ) : Data

    data class Struct(
        val x: UInt,
        val y: String,
    ) : Data
}
```

**C#**
```csharp
public abstract record Data {
    public sealed record Unit() : Data;

    public sealed record Newtype(uint Value) : Data;

    public sealed record Tuple(uint Field0, string Field1) : Data;

    public sealed record Struct(uint X, string Y) : Data;
}
```

**TypeScript**
```typescript
export type Data =
    | { kind: "Unit" }
    | { kind: "Newtype"; value: uint32 }
    | { kind: "Tuple"; field0: uint32; field1: str }
    | { kind: "Struct"; x: uint32; y: str };
```

### Tagging

Reflection reads facet's attributes, never serde's: an enum is externally tagged unless it has
`#[facet(tag = "...")]` (internally tagged) or `#[facet(tag = "...", content = "...")]`
(adjacently tagged). Give an enum the same attributes for facet as for serde.

Tagging does not change the Swift, Kotlin or C# declarations. It changes the TypeScript union's
discriminant: `#[facet(tag = "type")]` gives `{ type: "A"; … }`, and
`#[facet(tag = "t", content = "c")]` gives `{ t: "A"; c: … }`. An internally tagged newtype
variant holding a struct is `{ type: "E" } & Struct`. With `JsonPlugin`, the tagging also decides
the JSON, as the [serialization guide](crate::guide::serialization) describes.

`#[facet(untagged)]` is not supported, and reflection rejects an untagged enum (see
[Rejected when reflecting](#rejected-when-reflecting)).

## Other features

- **Generics.** A generic struct or enum is generated once, for the type arguments it is used
  with, under its bare name: `Page<Order>` becomes `Page`, with `Order` in place of `T`. Using it
  with a second set of type arguments is an error (below). `Option`, `Vec`, the sets and maps,
  `Box`, `Rc` and `Arc` are exempt.
- **Recursion.** A type can refer to itself through `Option<Box<T>>`, `Vec<T>` or a map. Swift
  enums are always `indirect`. A Swift struct field that refers back to its struct is declared
  `@Indirect`, a property wrapper from the serde runtime, when a plugin is configured; with no
  plugin it is not, and the output does not compile.
- **`#[facet(transparent)]`.** A newtype marked transparent is replaced by the type it wraps,
  through any number of layers, and gets no declaration of its own.
- **`#[facet(skip)]`.** The field or variant is left out.
- **`#[facet(opaque)]`.** The field is left out, as with `skip`. A newtype variant whose payload
  is opaque becomes a unit variant. Reflection does not look at the type of a skipped or opaque
  field, so either attribute keeps a field of an unsupported type out of the way.
- **Standard types that are structs.** A type facet describes as a plain struct is generated as
  one, under its Rust name: `Range<u32>` becomes a struct `Range` with `start` and `end`,
  `PhantomData<T>` a unit struct `PhantomData`, and `NonZeroU32` a newtype struct `NonZero`.

## Rejected when reflecting

[`RegistryBuilder::add_type`](crate::reflection::RegistryBuilder::add_type) and
[`build`](crate::reflection::RegistryBuilder::build) return an
[`Error`](crate::error::Error):

- A field whose type cannot be generated, wherever the unsupported type sits in it
  ([`UnsupportedFieldType`](crate::error::Error::UnsupportedFieldType)). That includes
  `Result`, `url::Url`, `chrono::NaiveDate`, `chrono::NaiveDateTime`,
  `chrono::DateTime<FixedOffset>`, `std::time::Duration`, `std::path::PathBuf` and
  `std::net::IpAddr`: the only opaque scalar types supported are `String`, `uuid::Uuid` and
  `chrono::DateTime<Utc>`. Mark the field `#[facet(skip)]` or `#[facet(opaque)]` to leave it out:
  `` field `after` of `Timeout` has type `Option<Duration>`, which can't be generated because `Duration` is not supported. Mark the field `#[facet(skip)]` or `#[facet(opaque)]` to leave it out, or change its type ``.
- An enum marked `#[facet(untagged)]`, whether it is added itself or reached through a field
  ([`UntaggedEnum`](crate::error::Error::UntaggedEnum)):
  ``enum `Value` is `#[facet(untagged)]`, which is not supported. Use an externally, internally (`#[facet(tag = "...")]`) or adjacently (`#[facet(tag = "...", content = "...")]`) tagged representation``.

- A generic type used with two sets of type arguments:
  `unsupported generic type: Page<String>, the type may have already been used with different parameters`.
- Two Rust types that would generate the same name in one namespace:
  `` two types generate as "Key" in namespace "ROOT": `app::Key` and `app::other::Key`. Rename one with `#[facet(rename = "...")]` or give it its own namespace with `#[facet(fg::namespace = "...")]` ``.
- A type that would inherit two different namespaces:
  `ambiguous namespace inheritance: "Row" in both "a" and "b"`.
- An `fg::namespace` value that is not an identifier (`invalid namespace identifier`), or not a
  string (`` bad attribute format: use `#[facet(fg::namespace = "my_ns")]` or `#[facet(fg::namespace)]` ``).
- A reference to a type that is not in the registry
  ([`DanglingTypeReference`](crate::error::Error::DanglingTypeReference)). Reflection registers
  every type it refers to, so this is a bug in `facet_generate`, and the message asks you to
  report it.

## Rejected when generating

An installer's `generate` returns an [`std::io::Error`] of kind
[`InvalidInput`](std::io::ErrorKind::InvalidInput):

- Swift, a map key or set element that is not `Hashable` (`()`, a tuple of two or more elements,
  or a struct or enum that holds one):
  `Map key type is not Hashable in Swift; native tuples and dictionaries do not conform to Hashable`,
  or the same message beginning `Set element type`. A dictionary whose keys and values are
  hashable is in fact accepted, whatever the message says
  ([#238](https://github.com/redbadger/facet-generate/issues/238)).
- Kotlin, a tuple of more than 12 elements:
  ``Kotlin: type `Wide` in the root namespace holds a tuple of 13 elements, but the serde runtime's tuple types stop at `Tuple12`; group some of its elements into a struct or a nested tuple``.
  Types derived with facet cannot hold such a tuple, so this only applies to a registry built
  some other way.
- C#, an `Option<Option<T>>`, including one inside a one-element tuple:
  ``C#: type `Nested` in the root namespace holds an `Option<Option<T>>`, which C# cannot declare (`T??`); wrap the inner option in a struct or an enum``.
- Kotlin, two variants of one enum, or two fields of one struct or struct variant, that would get
  the same identifier once renamed:
  ``Kotlin: variants `on-hold` and `on_hold` of `Status` would both become `ON_HOLD`; rename one of them with #[facet(rename = "...")]``.
- Swift and C#, a variant or field renamed to a name that starts with a digit:
  ``Swift: field `2nd` of `Probe` would become `2nd`, which is not a valid identifier because it starts with a digit; rename it with #[facet(rename = "...")] to a name that starts with a letter``.
  Kotlin writes such a name with a `_` in front instead, and TypeScript quotes it.
- A type or field name that the language cannot use, and that renaming would fix. For example, a
  Kotlin type named `Bytes` next to a byte field:
  ``Kotlin: type `Bytes` collides with the `Bytes` import used by the generated code; rename it with #[facet(rename = "...")]``.
- A namespace that becomes the same package, namespace, class or file as something else. For
  example:
  ``Kotlin: namespace "Kit" becomes the package `com.x.Kit`, the same as type `Kit` in the root namespace, the class `com.x.Kit`. Rename the type with `#[facet(rename = "...")]` or choose a different namespace``.
- Swift namespaces whose targets would depend on each other in a cycle, and a Swift external
  package at a URL with no version (see the [Swift guide](crate::guide::swift)).

Namespace collisions, Kotlin's tuple sizes and identifier collisions, C#'s `Option<Option<T>>`,
and Swift's target cycles and external package versions are checked before anything is written.
The other checks run as each module is generated, so the runtime and the modules before it may
already be on disk, and the failing module's file may be left empty or partly written
([#238](https://github.com/redbadger/facet-generate/issues/238),
[#250](https://github.com/redbadger/facet-generate/issues/250)).

## Output that does not compile

These are accepted by reflection and generation, but the generated code does not build:

- Kotlin with no plugin: a `#[facet(fg::bytes)]` field or a tuple of 4 to 12 elements. `Bytes`
  and `Tuple4` to `Tuple12` come from the serde runtime, which is only written with a plugin
  ([#130](https://github.com/redbadger/facet-generate/issues/130)).
- Swift with no plugin: a struct or enum used as a map key or set element, or a struct that
  refers to itself. Without a plugin no type declares `Hashable`, and no field is `@Indirect`
  ([#237](https://github.com/redbadger/facet-generate/issues/237)).
