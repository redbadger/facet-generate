use facet::Facet;

#[derive(Facet)]
pub struct SimpleAlias1(String);

#[derive(Facet)]
#[facet(alias)]
pub struct SimpleAlias2(String);

#[derive(Facet)]
#[facet(branded)]
pub struct BrandedStringAlias(String);

#[derive(Facet)]
#[facet(branded)]
pub struct BrandedOptionalStringAlias(Option<String>);

#[derive(Facet)]
#[facet(branded, swift = "Equatable, Hashable")]
pub struct BrandedU32Alias(u32);

#[derive(Facet)]
struct MyStruct {
    field: u32,
    other_field: String,
}

#[derive(Facet)]
#[facet(branded)]
pub struct BrandedStructAlias(MyStruct);
