#![expect(unused)]
#![expect(clippy::enum_variant_names)]

use facet::Facet;

#[derive(Facet)]
#[facet(swift(skip))]
pub struct NotVisibleInSwift {
    inner: u32,
}

#[derive(Facet)]
#[facet(kotlin(skip))]
pub struct NotVisibleInKotlin {
    inner: u32,
}

#[derive(Facet)]
#[facet(typescript(skip))]
pub struct NotVisibleInTypescript {
    inner: u32,
}

#[derive(Facet)]
#[repr(C)]
pub enum EnumWithVariantsPerLanguage {
    #[facet(swift(skip))]
    NotVisibleInSwift,
    #[facet(kotlin(skip))]
    NotVisibleInKotlin,
    #[facet(typescript(skip))]
    NotVisibleInTypescript,
}
