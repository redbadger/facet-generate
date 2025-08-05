#![expect(unused)]

use facet::Facet;

#[derive(Facet)]
pub struct NamedEmptyStruct {}

#[derive(Facet)]
#[repr(C)]
pub enum Test {
    NamedEmptyStruct(NamedEmptyStruct),
    AnonymousEmptyStruct {},
}
