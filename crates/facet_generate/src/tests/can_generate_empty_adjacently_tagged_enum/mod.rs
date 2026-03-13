#![expect(clippy::enum_variant_names)]

use facet::Facet;

#[derive(Facet)]
pub struct NamedEmptyStruct {}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum Test {
    NamedEmptyStruct(NamedEmptyStruct),
    AnonymousEmptyStruct {},
    NoStruct,
}
