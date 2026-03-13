use crate as fg;

use facet::Facet;
use serde::{Deserialize, Serialize};

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Serialize, Deserialize, Debug)]
pub(super) struct Context {
    pub urls: Vec<EditItemContextUrl>,
    pub apps: Vec<ItemApp>,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct EditItemViewModelSaveRequest {
    #[facet(fg::serialized_as = "String")]
    pub(super) context: Context,

    pub values: Vec<EditItemSaveValue>,
    pub fill_action: Option<AutoFillItemActionRequest>,
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct EditItemContextUrl;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct ItemApp;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct EditItemSaveValue;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Facet, Serialize, Deserialize, Debug)]
pub struct AutoFillItemActionRequest;
