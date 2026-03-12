#![expect(unused)]

use facet::Facet;

/// This is a comment.
#[derive(Facet)]
#[repr(C)]
pub enum Colors {
    Red,
    Blue,
    Green,
}

// TODO: enable swift, typescript (expect files need updating for no-encoding output)
crate::test! {
    Colors for kotlin
}
