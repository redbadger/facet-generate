use facet::Facet;

#[derive(Facet)]
pub struct MyEmptyStruct {}

// TODO: enable swift, typescript (expect files need updating for no-encoding output)
crate::test! {
    MyEmptyStruct for kotlin
}
