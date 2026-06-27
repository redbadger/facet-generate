use facet::Facet;

#[derive(Facet)]
pub struct MyEmptyStruct {}

// TODO: enable swift
crate::test! {
    MyEmptyStruct for kotlin, typescript
}
