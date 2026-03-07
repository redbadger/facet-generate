use crate as fg;

use facet::Facet;

#[derive(Facet)]
#[facet(fg::public)]
pub struct Bar(String);

#[derive(Facet)]
#[facet(fg::public)]
pub struct Foo {
    bar: Bar,
}

#[derive(Facet)]
#[facet(fg::public)]
#[repr(C)]
pub enum Baz {
    Bar,
    Foo,
}
