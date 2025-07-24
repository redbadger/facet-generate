#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct Foo {
    #[facet(default)]
    pub bar: bool,
}
