use facet::Facet;

#[derive(Facet)]
#[facet(public)]
pub struct Bar(String);

#[derive(Facet)]
#[facet(public)]
pub struct Foo {
    bar: Bar,
}

#[derive(Facet)]
#[facet(public)]
#[repr(C)]
pub enum Baz {
    Bar,
    Foo,
}
