use std::collections::HashMap;

use facet::Facet;

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum GenericEnum<A, B> {
    VariantA(A),
    VariantB(B),
}

#[derive(Facet)]
#[repr(C)]
pub struct StructUsingGenericEnum {
    enum_field: GenericEnum<String, i16>,
}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum GenericEnumUsingGenericEnum<T> {
    VariantC(GenericEnum<T, T>),
    VariantD(GenericEnum<&'static str, HashMap<String, T>>),
    VariantE(GenericEnum<&'static str, u32>),
}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum GenericEnumsUsingStructVariants<T, U> {
    VariantF { action: T },
    VariantG { action: T, response: U },
    VariantH { non_generic: i32 },
    VariantI { vec: Vec<T>, action: MyType<T, U> },
}

#[derive(Facet)]
pub struct MyType<T, U> {
    field1: T,
    field2: U,
}
