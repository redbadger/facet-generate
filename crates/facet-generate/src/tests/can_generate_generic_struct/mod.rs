#![expect(unused)]
#![expect(clippy::struct_field_names)]

use facet::Facet;

#[derive(Facet)]
pub struct GenericStruct<A, B> {
    field_a: A,
    field_b: Vec<B>,
}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum EnumUsingGenericStruct {
    VariantA(GenericStruct<String, f32>),
    VariantB(GenericStruct<&'static str, i32>),
    VariantC(GenericStruct<&'static str, bool>),
    VariantD(GenericStructUsingGenericStruct<()>),
}

#[derive(Facet)]
pub struct GenericStructUsingGenericStruct<T> {
    struct_field: GenericStruct<String, T>,
    second_struct_field: GenericStruct<T, String>,
    third_struct_field: GenericStruct<T, Vec<T>>,
}
