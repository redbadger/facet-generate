#[derive(Facet, Diffable)]
pub struct Foo {
    bar: Bar,
}

#[derive(Facet, Diffable)]
pub struct Bar {
    one: String,
}
