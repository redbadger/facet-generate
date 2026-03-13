use facet::Facet;

/// This is a comment.
#[derive(Facet)]
pub struct Foo {
    pub a: i8,
    pub b: i16,
    pub c: i32,
    pub e: u8,
    pub f: u16,
    pub g: u32,
}

// TODO: enable swift, typescript (expect files need updating for no-encoding output)
crate::test! {
    Foo for kotlin
}
