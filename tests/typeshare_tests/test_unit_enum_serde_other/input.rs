#![expect(unused)]

use facet::Facet;

/// This is a comment.
#[derive(Facet)]
#[repr(C)]
pub enum Source {
    Embedded,
    GoogleFont,
    Custom,
    #[facet(other)]
    Unknown,
}
