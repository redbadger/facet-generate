# `facet-generate` &middot; [![GitHub license](https://img.shields.io/github/license/redbadger/facet-generate?color=blue)](https://github.com/redbadger/facet-generate/blob/master/LICENSE) [![Crate version](https://img.shields.io/crates/v/facet-generate.svg)](https://crates.io/crates/facet-generate) [![Docs](https://img.shields.io/badge/docs.rs-facet_generate-green)](https://docs.rs/facet-generate/) [![Build status](https://img.shields.io/github/actions/workflow/status/redbadger/facet-generate/build.yaml)](https://github.com/redbadger/facet-generate/actions)

Reflect types annotated with [`#[Facet]`](https://crates.io/crates/facet) into Java, Swift, and TypeScript.

### Notes:
1. Currently, struct and enum renaming is not fully implemented and probably requires an upstream PR to facet.

### Usage

```sh
cargo add facet facet-generate
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

Struct and Enum renaming doesn't use `#[facet(rename = "Effect")]`, as facet doesn't seem to pass it through (yet?). So instead, for now, we use an arbitrary `ShapeAttribute` (`name` instead of `rename`), like this:

```rust
#[derive(Facet)]
#[facet(name = "Effect")]
struct EffectFfi {
    name: String,
    active: bool,
}
```

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
