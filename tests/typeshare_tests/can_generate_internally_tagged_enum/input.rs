#![expect(unused)]

use facet::Facet;

#[derive(Facet)]
pub struct ExplicitlyNamedStruct {
    pub a_field: String,
    pub another_field: u32,
}

#[derive(Facet)]
#[facet(tag = "type")]
#[repr(C)]
pub enum SomeEnum {
    A,
    B { field1: String },
    C { field1: u32, field2: f32 },
    D { field3: Option<bool> },
    E(ExplicitlyNamedStruct),
}
