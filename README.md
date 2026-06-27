# `facet_generate` · [![GitHub license](https://img.shields.io/github/license/redbadger/facet-generate?color=blue)](https://github.com/redbadger/facet-generate/blob/master/LICENSE) [![Crate version](https://img.shields.io/crates/v/facet_generate.svg)](https://crates.io/crates/facet_generate) [![Docs](https://img.shields.io/badge/docs.rs-facet_generate-green)](https://docs.rs/facet_generate/) [![Build status](https://img.shields.io/github/actions/workflow/status/redbadger/facet-generate/build.yaml)](https://github.com/redbadger/facet-generate/actions)

Reflect types annotated with [`#[derive(Facet)]`](https://crates.io/crates/facet) into Swift, Kotlin, TypeScript, and C#. Optionally generates serialization and deserialization code for [Bincode](https://github.com/bincode-org/bincode) and JSON encodings.

## Usage

```sh
cargo add facet facet_generate
```

```rust
use facet::Facet;
use facet_generate::reflection::RegistryBuilder;

#[derive(Facet)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Facet)]
#[repr(C)]
enum Shape {
    Circle { centre: Point, radius: f64 },
    Rectangle { position: Point, width: f64, height: f64 },
}

// Point is discovered automatically as a field type of Shape
let registry = RegistryBuilder::new()
    .add_type::<Shape>()?
    .build()?;
```

To generate code from the registry, use a language-specific `Installer`, then call `generate()` — the installer splits by namespace, installs runtimes, generates each module, and writes the package manifest. Add a plugin such as `BincodePlugin` to include `serialize`/`deserialize` methods and install the appropriate runtime library; omit `.plugin(...)` for plain type definitions only.

```rust
use facet_generate::generation::bincode::BincodePlugin;

// Swift
swift::Installer::new("MyPackage", &out_dir)
    .plugin(BincodePlugin)
    .generate(&registry)?;

// Kotlin
kotlin::Installer::new("com.example", &out_dir)
    .plugin(BincodePlugin)
    .generate(&registry)?;

// TypeScript
typescript::Installer::new("example", &out_dir)
    .plugin(BincodePlugin)
    .generate(&registry)?;

// C#
csharp::Installer::new("Example", &out_dir)
    .plugin(BincodePlugin)
    .generate(&registry)?;
```

With `BincodePlugin`, structs gain `serialize`/`deserialize` methods and enums gain standalone `serializeX`/`deserializeX` functions alongside a discriminated union type, per-variant constructor functions, and an exhaustive `matchX` helper. The examples below show the full generated module for both `Point` (struct) and `Shape` (enum) in each language.

> [!NOTE]
> The code blocks below are generated from the real output of the code
> generators and kept in sync by the `readme` integration test
> (`crates/facet_generate/tests/readme.rs`). Do not edit them by hand — run
> `UPDATE_EXPECT=1 cargo test -p facet_generate --test readme` to refresh them.

<details>
<summary>Swift</summary>

<!-- generated:swift:start -->

```swift
public struct Point: Hashable, Equatable {
    public var x: Double
    public var y: Double

    public init(x: Double, y: Double) {
        self.x = x
        self.y = y
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_f64(value: self.x)
        try serializer.serialize_f64(value: self.y)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Point {
        try deserializer.increase_container_depth()
        let x = try deserializer.deserialize_f64()
        let y = try deserializer.deserialize_f64()
        try deserializer.decrease_container_depth()
        return Point(x: x, y: y)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Point {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Shape: Hashable, Equatable {
    case circle(centre: Point, radius: Double)
    case rectangle(position: Point, width: Double, height: Double)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .circle(let centre, let radius):
            try serializer.serialize_variant_index(value: 0)
            try centre.serialize(serializer: serializer)
            try serializer.serialize_f64(value: radius)
        case .rectangle(let position, let width, let height):
            try serializer.serialize_variant_index(value: 1)
            try position.serialize(serializer: serializer)
            try serializer.serialize_f64(value: width)
            try serializer.serialize_f64(value: height)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Shape {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let centre = try Point.deserialize(deserializer: deserializer)
            let radius = try deserializer.deserialize_f64()
            try deserializer.decrease_container_depth()
            return .circle(centre: centre, radius: radius)
        case 1:
            let position = try Point.deserialize(deserializer: deserializer)
            let width = try deserializer.deserialize_f64()
            let height = try deserializer.deserialize_f64()
            try deserializer.decrease_container_depth()
            return .rectangle(position: position, width: width, height: height)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Shape: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Shape {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
```

<!-- generated:swift:end -->

</details>

<details>
<summary>Kotlin</summary>

<!-- generated:kotlin:start -->

```kotlin
data class Point(
    val x: Double,
    val y: Double,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        serializer.serialize_f64(x)
        serializer.serialize_f64(y)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Point {
            deserializer.increase_container_depth()
            val x = deserializer.deserialize_f64()
            val y = deserializer.deserialize_f64()
            deserializer.decrease_container_depth()
            return Point(x, y)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Point {
            if (input == null) {
                throw DeserializationError("Cannot deserialize null array")
            }
            val deserializer = BincodeDeserializer(input)
            val value = deserialize(deserializer)
            if (deserializer.get_buffer_offset() < input.size) {
                throw DeserializationError("Some input bytes were not read")
            }
            return value
        }
    }
}

sealed interface Shape {
    fun serialize(serializer: Serializer)

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    data class Circle(
        val centre: com.example.Point,
        val radius: Double,
    ) : Shape {
        override fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(0)
            centre.serialize(serializer)
            serializer.serialize_f64(radius)
            serializer.decrease_container_depth()
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Circle {
                deserializer.increase_container_depth()
                val centre = com.example.Point.deserialize(deserializer)
                val radius = deserializer.deserialize_f64()
                deserializer.decrease_container_depth()
                return Circle(centre, radius)
            }
        }
    }

    data class Rectangle(
        val position: com.example.Point,
        val width: Double,
        val height: Double,
    ) : Shape {
        override fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(1)
            position.serialize(serializer)
            serializer.serialize_f64(width)
            serializer.serialize_f64(height)
            serializer.decrease_container_depth()
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Rectangle {
                deserializer.increase_container_depth()
                val position = com.example.Point.deserialize(deserializer)
                val width = deserializer.deserialize_f64()
                val height = deserializer.deserialize_f64()
                deserializer.decrease_container_depth()
                return Rectangle(position, width, height)
            }
        }
    }

    companion object {
        @Throws(DeserializationError::class)
        fun deserialize(deserializer: Deserializer): Shape {
            val index = deserializer.deserialize_variant_index()
            return when (index) {
                0 -> Circle.deserialize(deserializer)
                1 -> Rectangle.deserialize(deserializer)
                else -> throw DeserializationError("Unknown variant index for Shape: $index")
            }
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Shape {
            if (input == null) {
                throw DeserializationError("Cannot deserialize null array")
            }
            val deserializer = BincodeDeserializer(input)
            val value = deserialize(deserializer)
            if (deserializer.get_buffer_offset() < input.size) {
                throw DeserializationError("Some input bytes were not read")
            }
            return value
        }
    }
}
```

<!-- generated:kotlin:end -->

</details>

<details>
<summary>TypeScript</summary>

<!-- generated:typescript:start -->

```typescript
type float64 = number;

export class Point {
    constructor (public x: float64, public y: float64) {
    }

    public serialize(serializer: Serializer): void {
        serializer.serializeF64(this.x);
        serializer.serializeF64(this.y);
    }

    static deserialize(deserializer: Deserializer): Point {
        const x = deserializer.deserializeF64();
        const y = deserializer.deserializeF64();
        return new Point(x,y);
    }
}

export type Shape =
    | { kind: "Circle"; centre: Point; radius: float64 }
    | { kind: "Rectangle"; position: Point; width: float64; height: float64 };

export const shapeCircle = (centre: Point, radius: float64): Shape => ({ kind: "Circle", centre, radius });

export const shapeRectangle = (position: Point, width: float64, height: float64): Shape => ({ kind: "Rectangle", position, width, height });

export function matchShape<R>(value: Shape, cases: {
    Circle: (v: Extract<Shape, { kind: "Circle" }>) => R;
    Rectangle: (v: Extract<Shape, { kind: "Rectangle" }>) => R;
}): R {
    return cases[value.kind as Shape["kind"]](value as never);
}

export function serializeShape(value: Shape, serializer: Serializer): void {
    switch (value.kind) {
        case "Circle": {
            serializer.serializeVariantIndex(0);
            value.centre.serialize(serializer);
            serializer.serializeF64(value.radius);
            break;
        }
        case "Rectangle": {
            serializer.serializeVariantIndex(1);
            value.position.serialize(serializer);
            serializer.serializeF64(value.width);
            serializer.serializeF64(value.height);
            break;
        }
        default: throw new Error("Unknown variant: " + (value as any).kind);
    }
}

export function deserializeShape(deserializer: Deserializer): Shape {
    const index = deserializer.deserializeVariantIndex();
    switch (index) {
        case 0: {
            const centre = Point.deserialize(deserializer);
            const radius = deserializer.deserializeF64();
            return { kind: "Circle", centre, radius };
        }
        case 1: {
            const position = Point.deserialize(deserializer);
            const width = deserializer.deserializeF64();
            const height = deserializer.deserializeF64();
            return { kind: "Rectangle", position, width, height };
        }
        default: throw new Error("Unknown variant index for Shape: " + index);
    }
}
```

<!-- generated:typescript:end -->

</details>

<details>
<summary>C#</summary>

<!-- generated:csharp:start -->

```csharp
namespace Example;

public partial class Point : ObservableObject, IFacetSerializable, IFacetDeserializable<Point> {
    [ObservableProperty]
    private double _x;
    [ObservableProperty]
    private double _y;

    public void Serialize(ISerializer serializer)
    {
        serializer.IncreaseContainerDepth();
        serializer.SerializeF64(X);
        serializer.SerializeF64(Y);
        serializer.DecreaseContainerDepth();
    }

    public static Point Deserialize(IDeserializer deserializer)
    {
        deserializer.IncreaseContainerDepth();
        var x = deserializer.DeserializeF64();
        var y = deserializer.DeserializeF64();
        deserializer.DecreaseContainerDepth();
        return new Point {
            X = x,
            Y = y,
        };
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Point BincodeDeserialize(byte[] input)
    {
        if (input is null)
        {
            throw new DeserializationError("Cannot deserialize null array");
        }
        var deserializer = new BincodeDeserializer(input);
        var value = Deserialize(deserializer);
        if (deserializer.GetBufferOffset() < input.Length)
        {
            throw new DeserializationError("Some input bytes were not read");
        }
        return value;
    }
}

public abstract record Shape : IFacetSerializable, IFacetDeserializable<Shape> {
    public sealed partial record Circle(Point Centre, double Radius) : Shape;

    public sealed partial record Rectangle(Point Position, double Width, double Height) : Shape;

    public abstract void Serialize(ISerializer serializer);

    private static Shape DeserializeCircle(IDeserializer deserializer)
    {
        var centre = Point.Deserialize(deserializer);
        var radius = deserializer.DeserializeF64();
        return new Circle(centre, radius);
    }

    public sealed partial record Circle
    {
        public override void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex(0);
            Centre.Serialize(serializer);
            serializer.SerializeF64(Radius);
            serializer.DecreaseContainerDepth();
        }

    }
    private static Shape DeserializeRectangle(IDeserializer deserializer)
    {
        var position = Point.Deserialize(deserializer);
        var width = deserializer.DeserializeF64();
        var height = deserializer.DeserializeF64();
        return new Rectangle(position, width, height);
    }

    public sealed partial record Rectangle
    {
        public override void Serialize(ISerializer serializer)
        {
            serializer.IncreaseContainerDepth();
            serializer.SerializeVariantIndex(1);
            Position.Serialize(serializer);
            serializer.SerializeF64(Width);
            serializer.SerializeF64(Height);
            serializer.DecreaseContainerDepth();
        }

    }
    public static Shape Deserialize(IDeserializer deserializer)
    {
        var index = deserializer.DeserializeVariantIndex();
        return index switch
        {
            0 => DeserializeCircle(deserializer),
            1 => DeserializeRectangle(deserializer),
            _ => throw new DeserializationError("Unknown variant index for Shape: " + index),
        }
        ;
    }

    public byte[] BincodeSerialize()
    {
        var serializer = new BincodeSerializer();
        Serialize(serializer);
        return serializer.GetBytes();
    }

    public static Shape BincodeDeserialize(byte[] input)
    {
        if (input is null)
        {
            throw new DeserializationError("Cannot deserialize null array");
        }
        var deserializer = new BincodeDeserializer(input);
        var value = Deserialize(deserializer);
        if (deserializer.GetBufferOffset() < input.Length)
        {
            throw new DeserializationError("Some input bytes were not read");
        }
        return value;
    }
}
```

<!-- generated:csharp:end -->

</details>

## Facet attributes

### Namespaces

Types that are explicitly annotated as belonging to a specific namespace are emitted as separate modules. These can be within the same package, or in a separate package if specified in the config during type generation (using [`ExternalPackage`](https://docs.rs/facet_generate/latest/facet_generate/generation/struct.ExternalPackage.html)).

* In Swift, namespaces become a separate target in the current package
* In Kotlin, they are emitted as a child namespace of the package's namespace
* In TypeScript they are emitted alongside as a separate `.ts` file
* In C#, each namespace becomes a file-scoped `namespace` written to a directory matching the dotted module path (e.g. `Company.Models.Shared`)

Notes:

* Once a namespace is set (via `#[facet(fg::namespace = "my_ns")]`) either at field-level (call-site) or type-level (called site), it will propagate to child types. The latest namespace is in effect until changed or cancelled. Type-level annotations take priority over field-level annotations.
* A namespace context can be unset (via `#[facet(fg::namespace)]`). This is still an explicit annotation, so it cancels any implicit annotations being carried forwards from higher in the graph. It places the type (and any child types) in the ROOT namespace.
* Namespaces are propagated through field level references, including via pointers and collections.
* Any ambiguity (i.e. a type is reached via more than one path, each with a different implicit namespace) will cause the typegen to emit an error, detailing the type involved and the namespaces that clash. The fix is then to either explicitly set (or unset) the type's namespace, or to align the inherited namespaces.


```rust
#[derive(Facet)]
#[facet(fg::namespace = "server_sent_events")]
pub struct SseRequest {
    pub url: String,
}

#[derive(Facet)]
#[facet(fg::namespace = "server_sent_events")]
#[repr(C)]
pub enum SseResponse {
    Chunk(Vec<u8>),
    Done,
}
```

### Renaming

Renaming uses Facet's builtin [`rename`](https://facet.rs/reference/attributes/#field-attributes--rename) and [`rename_all`](https://facet.rs/reference/attributes/#container-attributes--rename-all) attributes.

#### Container rename

Rename a struct or enum in the generated output (the Rust name stays the same):

```rust
#[derive(Facet)]
#[facet(rename = "Effect")]
struct EffectFfi {
    name: String,
    active: bool,
}
```

This also works on enums:

```rust
#[derive(Facet)]
#[facet(rename = "Effect")]
#[repr(C)]
enum EffectFfi {
    One,
    Two,
}
```

When a renamed type is referenced from another struct, the generated code uses
the new name automatically.

#### Field rename

Rename individual struct fields with `#[facet(rename = "...")]`:

```rust
#[derive(Facet)]
struct Request {
    #[facet(rename = "id")]
    request_id: u32,
}
```

This works for all field types — primitives, `Option<T>`, `Vec<T>`, and
user-defined types.

#### Enum variant rename

Rename individual enum variants:

```rust
#[derive(Facet)]
#[repr(C)]
enum Effect {
    #[facet(rename = "Id")]
    RequestId,
}
```

Fields inside struct variants can also be renamed:

```rust
#[derive(Facet)]
#[repr(C)]
enum Message {
    Info {
        #[facet(rename = "msg")]
        message: String,
    },
}
```

#### `rename_all`

Apply a naming convention to all fields in a struct or all variants in an enum:

```rust
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
struct Config {
    request_id: u32,
    user_name: String,
    is_active: bool,
}
```

This also works on enums:

```rust
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
enum Effect {
    RequestId,       // → requestId
    SomeOtherVariant, // → someOtherVariant
}
```

A per-field or per-variant `rename` always takes priority over `rename_all`:

```rust
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
struct Request {
    #[facet(rename = "id")]  // "id", not "requestId"
    request_id: u32,
}
```

Container-level `rename` and field/variant-level `rename` (or `rename_all`) can
be combined freely.

### Skipping struct fields or enum variants

You can annotate fields or variants with `#[facet(skip)]` to prevent them from being emitted in the generated code. (Note: you can also use `#[facet(opaque)]` to prevent Facet from recursing through).

```rust
#[derive(Facet)]
#[repr(C)]
pub enum Event {
    Get,

    #[facet(skip)]
    Set(#[facet(opaque)] HttpResult<HttpResponse<Count>, HttpError>),
}
```

### Transparent

You can skip through (even successive layers) of newtyping by annotating the struct with `#[facet(transparent)]`.

```rust
#[derive(Facet)]
#[facet(transparent)]
struct Inner(i32);

#[derive(Facet)]
struct MyStruct {
    inner: Inner,
}
```

With `#[facet(transparent)]`, `Inner` is unwrapped and `MyStruct.inner` is generated as a plain `Int32` (Swift) / `Int` (Kotlin) / `number` (TypeScript) / `int` (C#) in the target language.

### Bytes

In order to generate byte array types (e.g. `[UInt8]` in Swift, `Bytes` in Kotlin, `Uint8Array` in TypeScript, `byte[]` in C#) for `Vec<u8>` and `&'a [u8]`, use the `#[facet(fg::bytes)]` attribute:

```rust
#[derive(Facet)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<HttpHeader>,
    #[facet(fg::bytes)]
    pub body: Vec<u8>,
}
```
