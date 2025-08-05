#![expect(unused)]

use std::collections::HashMap;

use facet::Facet;

#[derive(Facet)]
pub struct ItemDetailsFieldValue {
    hello: String,
}

#[derive(Facet)]
#[facet(tag = "t", content = "c")]
#[repr(C)]
pub enum AdvancedColors {
    Str(String),
    Number(i32),
    NumberArray(Vec<i32>),
    ReallyCoolType(ItemDetailsFieldValue),
    ArrayReallyCoolType(Vec<ItemDetailsFieldValue>),
    DictionaryReallyCoolType(HashMap<String, ItemDetailsFieldValue>),
}
