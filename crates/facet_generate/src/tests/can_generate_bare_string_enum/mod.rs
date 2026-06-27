use facet::Facet;

/// This is a comment.
#[derive(Facet)]
#[repr(C)]
pub enum Colors {
    Red,
    Blue,
    Green,
}

// TODO: enable swift
crate::test! {
    Colors for kotlin, typescript
}
