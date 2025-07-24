#[derive(Facet)]
#[facet(tag = "type")]
pub enum Test {
    AnonymousEmptyStruct {},
    NoStruct,
}
