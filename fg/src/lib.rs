//! Extension attributes for facet-generate code generation.
//!
//! These attributes are used with the `#[facet(fg::...)]` syntax.
//! For example: `#[facet(fg::namespace = "MyNs")]`, `#[facet(fg::bytes)]`.
//!
//! Users of `facet_generate` get this crate transitively and should add
//! `extern crate fg;` or `use fg;` at their crate root so that the
//! `fg::__attr!` macro is resolvable by the `#[derive(Facet)]` expansion.

facet::define_attr_grammar! {
    ns "fg";
    crate_path ::fg;

    /// Extension attributes for facet-generate code generation.
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
