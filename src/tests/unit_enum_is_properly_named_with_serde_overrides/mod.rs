#![expect(unused)]

use facet::Facet;

/// This is a comment.
/// Continued lovingly here
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum Colors {
    Red = 0,
    Blue = 1,
    /// Green is a cool color
    #[facet(rename = "green-like")]
    Green = 2,
}
