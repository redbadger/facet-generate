use facet::Facet;

#[derive(Facet)]
#[facet(tag = "type")]
#[repr(C)]
pub enum Test {
    AnonymousEmptyStruct {},
    NoStruct,
}
