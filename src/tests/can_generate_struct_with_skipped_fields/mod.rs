use facet::Facet;

#[derive(Facet)]
pub struct MyStruct {
    a: i32,
    #[facet(skip)]
    b: i32,
    #[facet(swift(skip))]
    c: i32,
    #[facet(skip)]
    d: i32,
}

crate::test! {
    MyStruct for java
}
