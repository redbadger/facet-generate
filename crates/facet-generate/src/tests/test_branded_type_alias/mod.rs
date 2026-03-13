use crate as fg;

use facet::Facet;

#[derive(Facet)]
pub struct SimpleAlias1(String);

#[derive(Facet)]
#[facet(rename = "SimpleAlias2")]
pub struct SimpleAlias2(String);

#[derive(Facet)]
#[facet(fg::branded)]
pub struct BrandedStringAlias(String);

#[derive(Facet)]
#[facet(fg::branded)]
pub struct BrandedOptionalStringAlias(Option<String>);

#[derive(Facet)]
#[facet(fg::branded)]
// TODO: re-add language-specific annotation: #[facet(swift = "Equatable, Hashable")]
pub struct BrandedU32Alias(u32);

#[derive(Facet)]
struct MyStruct {
    field: u32,
    other_field: String,
}

#[derive(Facet)]
#[facet(fg::branded)]
pub struct BrandedStructAlias(MyStruct);
