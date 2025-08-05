#![expect(unused)]

use difficient::Diffable;
use facet::Facet;

#[derive(Facet, Clone, Debug, PartialEq, Eq)]
#[facet(rename_all = "camelCase")]
pub struct NotDiffable {}

#[derive(Facet, Clone)]
#[facet(tag = "type")]
#[facet(rename_all = "camelCase")]
#[derive(Diffable)]
#[repr(C)]
pub enum InternallyTagged {
    AnonymousStruct {
        #[diffable(atomic)]
        atomic: NotDiffable,
    },
}
