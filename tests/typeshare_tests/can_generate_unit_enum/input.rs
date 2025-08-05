#![expect(unused)]

use facet::Facet;

/// This is a comment.
/// Continued lovingly here
#[derive(Facet)]
#[repr(C)]
pub enum Colors {
    Red = 0,
    Blue = 1,
    /// Green is a cool color
    Green = 2,
}
