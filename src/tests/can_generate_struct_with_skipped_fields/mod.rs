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

crate::test! {
    MyStruct for java, kotlin
}
