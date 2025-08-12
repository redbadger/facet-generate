use facet::Facet;

#[derive(Facet)]
pub struct MyEmptyStruct {}

crate::test! {
    MyEmptyStruct for kotlin
}
