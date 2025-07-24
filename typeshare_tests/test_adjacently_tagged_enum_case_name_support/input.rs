#[derive(Facet)]
pub struct ItemDetailsFieldValue {}

#[derive(Facet)]
#[facet(rename_all = "camelCase", tag = "type", content = "content")]
pub enum AdvancedColors {
    Str(String),
    Number(i32),
    #[facet(rename = "number-array")]
    NumberArray(Vec<i32>),
    #[facet(rename = "reallyCoolType")]
    ReallyCoolType(ItemDetailsFieldValue),
}
