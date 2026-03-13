use crate as fg;

use facet::Facet;

#[derive(Facet)]
#[facet(fg::serialized_as = "String")]
pub struct ItemId(i64);
