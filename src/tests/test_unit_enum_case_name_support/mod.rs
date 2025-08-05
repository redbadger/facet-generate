#![expect(unused)]

use facet::Facet;

/// This is a comment.
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum Colors {
    Red,
    #[facet(rename = "blue-ish")]
    Blue,
    #[facet(rename = "Green")]
    Green,
}
