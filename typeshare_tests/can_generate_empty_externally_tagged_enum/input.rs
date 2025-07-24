#[derive(Facet)]
pub struct NamedEmptyStruct {}

#[derive(Facet)]
pub enum Test {
    NamedEmptyStruct(NamedEmptyStruct),
    AnonymousEmptyStruct {},
}
