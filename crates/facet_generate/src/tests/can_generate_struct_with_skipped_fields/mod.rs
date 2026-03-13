use facet::Facet;

#[derive(Facet)]
pub struct MyStruct {
    a: i32,
    #[facet(skip)]
    b: i32,
    // TODO: #[facet(swift(skip))]
    c: i32,
    // TODO: #[facet(skip)]
    d: i32,
}

// TODO: enable swift, typescript (expect files need updating for no-encoding output)
crate::test! {
    MyStruct for kotlin
}
