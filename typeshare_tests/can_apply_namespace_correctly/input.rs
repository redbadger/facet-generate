#[derive(Facet)]
pub struct ItemDetailsFieldValue {
    hello: String,
}

#[derive(Facet)]
#[facet(tag = "t", content = "c")]
pub enum AdvancedColors {
    Str(String),
    Number(i32),
    NumberArray(Vec<i32>),
    ReallyCoolType(ItemDetailsFieldValue),
    ArrayReallyCoolType(Vec<ItemDetailsFieldValue>),
    DictionaryReallyCoolType(HashMap<String, ItemDetailsFieldValue>),
}
