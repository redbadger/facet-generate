#![expect(unused)]

use facet::Facet;

/// This struct has a unit field
#[derive(Facet)]
#[facet(default, rename_all = "camelCase")]
struct StructHasVoidType {
    this_is_a_unit: (),
}

/// This enum has a variant associated with unit data
#[derive(Facet)]
#[facet(default, rename_all = "camelCase", tag = "type", content = "content")]
#[repr(C)]
enum EnumHasVoidType {
    HasAUnit(()),
}
