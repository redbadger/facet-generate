use difficient::Diffable;
use facet::Facet;

#[derive(Facet, Diffable)]
pub struct Foo {
    bar: Option<Bar>,
}

#[derive(Facet, Diffable, Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    one: String,
}
