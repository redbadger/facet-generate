#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
pub struct Struct {
    field1: String,
    field2: u32,
}

#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
pub enum UnitEnum {
    VariantA,
    VariantB,
    VariantC,
}

#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
pub enum ExternallyTaggedEnum {
    TupleVariant(String),
    StructVariant { field: String },
}

#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
#[facet(tag = "type")]
pub enum InternallyTaggedEnum {
    UnitVariant,
    TupleVariant(String),
    StructVariant { field: String },
}

#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
#[facet(tag = "type", content = "content")]
pub enum AdjacentlyTaggedEnum {
    UnitVariant,
    TupleVariant(String),
    StructVariant { field: String },
}
