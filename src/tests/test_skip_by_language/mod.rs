#![expect(clippy::enum_variant_names)]

use facet::Facet;

#[derive(Facet)]
// TODO: implement language-specific skip: #[facet(swift(skip))]
#[facet(skip)]
pub struct NotVisibleInSwift {
    inner: u32,
}

#[derive(Facet)]
// TODO: implement language-specific skip: #[facet(kotlin(skip))]
#[facet(skip)]
pub struct NotVisibleInKotlin {
    inner: u32,
}

#[derive(Facet)]
// TODO: implement language-specific skip: #[facet(typescript(skip))]
#[facet(skip)]
pub struct NotVisibleInTypescript {
    inner: u32,
}

#[derive(Facet)]
#[repr(C)]
pub enum EnumWithVariantsPerLanguage {
    // TODO: implement language-specific skip: #[facet(swift(skip))]
    #[facet(skip)]
    NotVisibleInSwift,
    // TODO: implement language-specific skip: #[facet(kotlin(skip))]
    #[facet(skip)]
    NotVisibleInKotlin,
    // TODO: implement language-specific skip: #[facet(typescript(skip))]
    #[facet(skip)]
    NotVisibleInTypescript,
}
