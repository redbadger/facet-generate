use difficient::Diffable;
use facet::Facet;

#[derive(Facet, Diffable, Clone)]
pub struct Foo {
    one: bool,
    two: Option<String>,
}
