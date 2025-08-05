use facet::Facet;
use serde::{Deserialize, Serialize};

#[derive(Facet, Serialize, Deserialize, Debug)]
pub(super) struct Context {
    pub urls: Vec<EditItemContextUrl>,
    pub apps: Vec<ItemApp>,
}

#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct EditItemViewModelSaveRequest {
    #[facet(serialized_as = "String")]
    pub(super) context: Context,

    pub values: Vec<EditItemSaveValue>,
    pub fill_action: Option<AutoFillItemActionRequest>,
}

#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct EditItemContextUrl;

#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct ItemApp;

#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct EditItemSaveValue;

#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct AutoFillItemActionRequest;
