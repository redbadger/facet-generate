use facet::Facet;

#[derive(Facet)]
pub struct ItemDetailsFieldValue {}

#[derive(Facet)]
#[facet(rename_all = "camelCase", tag = "type", content = "content")]
#[repr(C)]
pub enum AdvancedColors {
    Str(String),
    Number(i32),
    #[facet(rename = "number-array")]
    NumberArray(Vec<i32>),
    #[facet(rename = "reallyCoolType")]
    ReallyCoolType(ItemDetailsFieldValue),
}
