# `facet_generate` · [![GitHub license](https://img.shields.io/github/license/redbadger/facet-generate?color=blue)](https://github.com/redbadger/facet-generate/blob/master/LICENSE) [![Crate version](https://img.shields.io/crates/v/facet_generate.svg)](https://crates.io/crates/facet_generate) [![Docs](https://img.shields.io/badge/docs.rs-facet_generate-green)](https://docs.rs/facet_generate/) [![Build status](https://img.shields.io/github/actions/workflow/status/redbadger/facet-generate/build.yaml)](https://github.com/redbadger/facet-generate/actions)

Reflect types annotated with [`#[derive(Facet)]`](https://crates.io/crates/facet) into Swift, Kotlin, and TypeScript. Optionally generates serialization and deserialization code for [Bincode](https://github.com/bincode-org/bincode) and JSON encodings.

> **Note:** A Java generator is also available but deprecated — use Kotlin for Android targets.

## Usage

```sh
cargo add facet facet_generate
```

```rust
use facet::Facet;

#[derive(Facet)]
#[repr(C)]
enum HttpResult {
    Ok(HttpResponse),
    Err(HttpError),
}

#[derive(Facet)]
struct HttpResponse {
    status: u16,
    headers: Vec<HttpHeader>,
    #[facet(bytes)]
    body: Vec<u8>,
}

#[derive(Facet)]
struct HttpHeader {
    name: String,
    value: String,
}

#[derive(Facet)]
#[repr(C)]
enum HttpError {
    #[facet(skip)]
    Http {
        status: u16,
        message: String,
        body: Option<Vec<u8>>,
    },
    #[facet(skip)]
    Json(String),
    Url(String),
    Io(String),
    Timeout,
}

let registry = RegistryBuilder::new()
    .add_type::<HttpResult>()?
    .build()?;
```

To generate code from the registry, use a language-specific `Installer`. Configure it with an encoding and call `generate()` — the installer takes care of splitting by namespace, installing runtimes, generating each module, and writing the package manifest.

When an encoding like Bincode is configured, the appropriate **runtimes** — small serialization libraries in the target language — are installed automatically alongside the generated code.

```rust
use facet_generate::generation::Encoding;

// Swift
swift::Installer::new("MyPackage", &out_dir)
    .encoding(Encoding::Bincode)
    .generate(&registry)?;

// Kotlin
kotlin::Installer::new("com.example", &out_dir)
    .encoding(Encoding::Bincode)
    .generate(&registry)?;

// TypeScript
typescript::Installer::new("example", &out_dir, InstallTarget::Node)
    .encoding(Encoding::Bincode)
    .generate(&registry)?;
```

To generate type definitions only (without serialization), simply omit the encoding:

```rust
swift::Installer::new("MyPackage", &out_dir)
    .generate(&registry)?;
```

When configured with an encoding such as `Encoding::Bincode`, the generated types include `serialize` and `deserialize` methods alongside the type definitions. Without an encoding (the default), only the type definitions are generated. For the types above with Bincode, this generates the following code (showing `HttpHeader` as a representative example — all types are generated similarly):

### Swift

```swift
public struct HttpHeader: Hashable {
    @Indirect public var name: String
    @Indirect public var value: String

    public init(name: String, value: String) {
        self.name = name
        self.value = value
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.name)
        try serializer.serialize_str(value: self.value)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpHeader {
        try deserializer.increase_container_depth()
        let name = try deserializer.deserialize_str()
        let value = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return HttpHeader(name: name, value: value)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpHeader {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
```

### Kotlin

```kotlin
data class HttpHeader(
    val name: String,
    val value: String,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        serializer.serialize_str(name)
        serializer.serialize_str(value)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): HttpHeader {
            deserializer.increase_container_depth()
            val name = deserializer.deserialize_str()
            val value = deserializer.deserialize_str()
            deserializer.decrease_container_depth()
            return HttpHeader(name, value)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): HttpHeader {
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

### TypeScript

```typescript
export class HttpHeader {
  constructor (public name: str, public value: str) {
  }

  public serialize(serializer: Serializer): void {
    serializer.serializeStr(this.name);
    serializer.serializeStr(this.value);
  }

  static deserialize(deserializer: Deserializer): HttpHeader {
    const name = deserializer.deserializeStr();
    const value = deserializer.deserializeStr();
    return new HttpHeader(name,value);
  }
}
```

## Facet attributes

### Namespaces

Types that are explicitly annotated as belonging to a specific namespace are emitted as separate modules. These can be within the same package, or in a separate package if specified in the config during type generation (using [`ExternalPackage`](https://docs.rs/facet_generate/latest/facet_generate/generation/struct.ExternalPackage.html)).

* In Swift, namespaces become a separate target in the current package
* In Kotlin, they are emitted as a child namespace of the package's namespace
* In TypeScript they are emitted alongside as a separate `.ts` file

Notes:

* Once a namespace is set (via `#[facet(namespace = "my_ns")]`) either at field-level (call-site) or type-level (called site), it will propagate to child types. The latest namespace is in effect until changed or cancelled. Type-level annotations take priority over field-level annotations.
* A namespace context can be unset (via `#[facet(namespace = None)]`). This is still an explicit annotation, so it cancels any implicit annotations being carried forwards from higher in the graph. It places the type (and any child types) in the ROOT namespace.
* Namespaces are propagated through field level references, including via pointers and collections.
* Any ambiguity (i.e. a type is reached via more than one path, each with a different implicit namespace) will cause the typegen to emit an error, detailing the type involved and the namespaces that clash. The fix is then to either explicitly set (or unset) the type's namespace, or to align the inherited namespaces.


```rust
#[derive(Facet)]
#[facet(namespace = "server_sent_events")]
pub struct SseRequest {
    pub url: String,
}

#[derive(Facet)]
#[facet(namespace = "server_sent_events")]
#[repr(C)]
pub enum SseResponse {
    Chunk(Vec<u8>),
    Done,
}
```

### Renaming

Struct and Enum renaming doesn't use `#[facet(rename = "Effect")]`, as facet doesn't seem to pass it through (yet?). So instead, for now, we use an arbitrary `ShapeAttribute` (`name` instead of `rename`), like this:

```rust
#[derive(Facet)]
#[facet(name = "Effect")]
struct EffectFfi {
    name: String,
    active: bool,
}
```

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

With `#[facet(transparent)]`, `Inner` is unwrapped and `MyStruct.inner` is generated as a plain `i32` / `Int32` / `number` in the target language.

### Bytes

In order to generate byte array types (e.g. `[UInt8]` in Swift, `Bytes` in Kotlin, `Uint8Array` in TypeScript) for `Vec<u8>` and `&'a [u8]`, use the `#[facet(bytes)]` attribute:

```rust
#[derive(Facet)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<HttpHeader>,
    #[facet(bytes)]
    pub body: Vec<u8>,
}
```
