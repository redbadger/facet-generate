#[derive(Facet)]
pub struct NamedEmptyStruct {}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
pub enum Test {
    NamedEmptyStruct(NamedEmptyStruct),
    AnonymousEmptyStruct {},
    NoStruct,
}
