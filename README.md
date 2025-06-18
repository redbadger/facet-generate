# `facet-generate` &middot; [![GitHub license](https://img.shields.io/github/license/redbadger/facet-generate?color=blue)](https://github.com/redbadger/facet-generate/blob/master/LICENSE) [![Crate version](https://img.shields.io/crates/v/facet-generate.svg)](https://crates.io/crates/facet-generate) [![Docs](https://img.shields.io/badge/docs.rs-facet_generate-green)](https://docs.rs/facet-generate/) [![Build status](https://img.shields.io/github/actions/workflow/status/redbadger/facet-generate/build.yaml)](https://github.com/redbadger/facet-generate/actions)

An adapter to reflect types annotated with [`#[Facet]`](https://crates.io/crates/facet) into the Intermediate Representation (IR) used by [`serde-generate`](https://crates.io/crates/serde-generate) (which generates code for C++, Java, Python, Rust, Go, C#, Swift, OCaml, and Dart).

### Note:
Currently, struct and enum renaming is not fully implemented and probably requires an upstream PR to facet.


### Usage

```sh
cargo add facet facet-generate
```

```rust
use facet::Facet;

#[derive(Facet)]
#[repr(C)]
pub enum HttpResult {
    Ok(HttpResponse),
    Err(HttpError),
}

#[derive(Facet)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<HttpHeader>,
    #[facet(bytes)]
    pub body: Vec<u8>,
}

#[derive(Facet)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Facet)]
#[repr(C)]
pub enum HttpError {
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

let registry = facet_generate::reflect::<HttpResult>();
insta::assert_yaml_snapshot!(registry.containers, @r"
HttpError:
  ENUM:
    0:
      Url:
        NEWTYPE: STR
    1:
      Io:
        NEWTYPE: STR
    2:
      Timeout: UNIT
HttpHeader:
  STRUCT:
    - name: STR
    - value: STR
HttpResponse:
  STRUCT:
    - status: U16
    - headers:
        SEQ:
          TYPENAME: HttpHeader
    - body: BYTES
HttpResult:
  ENUM:
    0:
      Ok:
        NEWTYPE:
          TYPENAME: HttpResponse
    1:
      Err:
        NEWTYPE:
          TYPENAME: HttpError
");
```
