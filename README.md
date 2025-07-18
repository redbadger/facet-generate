# `facet_generate` &middot; [![GitHub license](https://img.shields.io/github/license/redbadger/facet-generate?color=blue)](https://github.com/redbadger/facet-generate/blob/master/LICENSE) [![Crate version](https://img.shields.io/crates/v/facet_generate.svg)](https://crates.io/crates/facet_generate) [![Docs](https://img.shields.io/badge/docs.rs-facet_generate-green)](https://docs.rs/facet_generate/) [![Build status](https://img.shields.io/github/actions/workflow/status/redbadger/facet-generate/build.yaml)](https://github.com/redbadger/facet-generate/actions)

Reflect types annotated with [`#[Facet]`](https://crates.io/crates/facet) into Java, Swift, and TypeScript.

### Usage

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

let registry = Registry::new().add_type::<HttpResult>().build();
insta::assert_yaml_snapshot!(registry, @r"
? namespace: ROOT
  name: HttpError
: ENUM:
    0:
      Url:
        NEWTYPE: STR
    1:
      Io:
        NEWTYPE: STR
    2:
      Timeout: UNIT
? namespace: ROOT
  name: HttpHeader
: STRUCT:
    - name: STR
    - value: STR
? namespace: ROOT
  name: HttpResponse
: STRUCT:
    - status: U16
    - headers:
        SEQ:
          TYPENAME:
            namespace: ROOT
            name: HttpHeader
    - body: BYTES
? namespace: ROOT
  name: HttpResult
: ENUM:
    0:
      Ok:
        NEWTYPE:
          TYPENAME:
            namespace: ROOT
            name: HttpResponse
    1:
      Err:
        NEWTYPE:
          TYPENAME:
            namespace: ROOT
            name: HttpError
");
```

#### Arbitrary facet attributes

##### Namespaces

Types that are explictly annotated as belonging to a specific namespace are emitted as separate modules. In Swift this means they are a separate target in the current package. In Java, they are emitted as a child namespace of the package's namespace. In TypeScript they are emitted alongside as a separate `.ts` file.

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

##### Renaming

Struct and Enum renaming doesn't use `#[facet(rename = "Effect")]`, as facet doesn't seem to pass it through (yet?). So instead, for now, we use an arbitrary `ShapeAttribute` (`name` instead of `rename`), like this:

```rust
#[derive(Facet)]
#[facet(name = "Effect")]
struct EffectFfi {
    name: String,
    active: bool,
}
```


##### Skipping struct fields or enum variants

You can annotate fields or variants with `#[facet(skip)]` to prevent them from being emitted in the generated code. (Note: you can also use `#[facet(opaque)])` to prevent Facet from recursing through).

```rust
#[derive(Facet)]
#[repr(C)]
pub enum Event {
    Get,

    #[facet(skip)]
    Set(HttpResult<HttpResponse<Count>, HttpError>),
}
```

##### Transparent

You can skip through (even successive layers) of newtyping by annotating the struct with `#[facet(transparent)]`.

```rust
#[test]
fn transparent() {
    #[derive(Facet)]
    #[facet(transparent)]
    struct Inner(i32);

    #[derive(Facet)]
    struct MyStruct {
        inner: Inner,
    }

    let registry = RegistryBuilder::new().add_type::<MyStruct>().build();
    insta::assert_yaml_snapshot!(registry, @r"
    ? namespace: ROOT
      name: MyStruct
    : STRUCT:
        - inner: I32
    ");
}
```

##### Bytes

In order to specify `BYTES` in the IR (for `Vec<u8>` and `&'a [u8]`), we can use the `#[facet(bytes)]` attribute:

```rust
#[derive(Facet)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<HttpHeader>,
    #[facet(bytes)]
    pub body: Vec<u8>,
}
```
