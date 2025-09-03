use facet::Facet;

#[derive(Facet)]
pub struct Bar(String);

#[derive(Facet)]
pub struct Foo {
    bar: Bar,
}

// crate::test! {
//     Foo for kotlin
// }
