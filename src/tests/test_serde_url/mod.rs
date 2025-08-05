use facet::Facet;

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct Foo {
    pub url: url::Url,
}
