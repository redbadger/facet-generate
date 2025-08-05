#![expect(clippy::enum_variant_names)]
#![expect(unused)]

use facet::Facet;

#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
pub struct Struct {
    field1: String,
    field2: u32,
}

#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
#[repr(C)]
pub enum UnitEnum {
    VariantA,
    VariantB,
    VariantC,
}

#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
#[repr(C)]
pub enum ExternallyTaggedEnum {
    TupleVariant(String),
    StructVariant { field: String },
}

#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
#[facet(tag = "type")]
#[repr(C)]
pub enum InternallyTaggedEnum {
    UnitVariant,
    TupleVariant(String),
    StructVariant { field: String },
}

#[derive(Facet)]
#[facet(kotlin = "Parcelable")]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum AdjacentlyTaggedEnum {
    UnitVariant,
    TupleVariant(String),
    StructVariant { field: String },
}
