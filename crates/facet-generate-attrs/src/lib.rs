//! Extension attributes for [`facet_generate`] code generation.
//!
//! You write these attributes with the `#[facet(fg::...)]` syntax, for example
//! `#[facet(fg::namespace = "MyNs")]` or `#[facet(fg::bytes)]`.
//!
//! This crate defines the attribute grammar for `facet_generate`. It is a separate crate
//! because Rust does not let a crate use an absolute path to refer to a `#[macro_export]`
//! macro that a macro expansion in the same crate defines.
//!
//! The attributes expand to paths in `facet_generate_attrs`. A crate that uses them must
//! therefore depend on `facet-generate-attrs` directly, as well as on [`facet_generate`],
//! which re-exports everything from this crate.
//!
//! This crate is `no_std` and needs only `alloc`, so a firmware crate can use the
//! attributes.
//!
//! [`facet_generate`]: https://docs.rs/facet_generate

#![no_std]

facet::define_attr_grammar! {
    ns "fg";
    crate_path ::facet_generate_attrs;

    /// Extension attributes for facet_generate code generation.
    pub enum Attr {
        /// Assign a type or field to a namespace for code generation module organization.
        ///
        /// Usage:
        /// - `#[facet(fg::namespace = "MyNamespace")]` — set namespace
        /// - `#[facet(fg::namespace)]` — explicitly clear namespace (root)
        Namespace(Option<&'static str>),

        /// Treat a `Vec<u8>`, `&[u8]`, `[u8; N]`, or `Bytes` field as binary data.
        ///
        /// Usage: `#[facet(fg::bytes)]`
        Bytes,

        /// Mark a type as branded (a newtype with distinct identity).
        ///
        /// Usage: `#[facet(fg::branded)]`
        Branded,

        /// Mark a field or type as public in the generated code.
        ///
        /// Usage: `#[facet(fg::public)]`
        Public,

        /// Mark a field as read-only in the generated code.
        ///
        /// Usage: `#[facet(fg::readonly)]`
        Readonly,

        /// Specify a serialization proxy type name.
        ///
        /// Usage: `#[facet(fg::serialized_as = "String")]`
        SerializedAs(&'static str),
    }
}
