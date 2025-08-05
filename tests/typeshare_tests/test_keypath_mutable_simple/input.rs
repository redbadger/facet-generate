use difficient::Diffable;
use facet::Facet;

#[derive(Facet, Diffable)]
pub struct Foo {
    one: bool,
}
