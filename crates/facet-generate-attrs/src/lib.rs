//! Extension attributes for [`facet_generate`] code generation.
//!
//! These attributes are used with the `#[facet(fg::...)]` syntax.
//! For example: `#[facet(fg::namespace = "MyNs")]`, `#[facet(fg::bytes)]`.
//!
//! This crate provides the attribute grammar definitions for `facet_generate`.
//! It exists as a separate crate to work around Rust's restriction on
//! accessing macro-expanded `#[macro_export]` macros by absolute paths
//! within the same crate.
//!
//! Users should depend on [`facet_generate`] directly,
//! which re-exports everything from this crate.
//!
//! [`facet_generate`]: https://docs.rs/facet_generate

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
