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
pub enum EnumWithVariantsPerLanguage {
    #[facet(swift(skip))]
    NotVisibleInSwift,
    #[facet(kotlin(skip))]
    NotVisibleInKotlin,
    #[facet(typescript(skip))]
    NotVisibleInTypescript,
}
