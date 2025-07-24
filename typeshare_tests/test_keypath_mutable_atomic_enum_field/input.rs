#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct NotDiffable {}

#[derive(Facet)]
#[facet(tag = "type")]
#[facet(rename_all = "camelCase")]
#[derive(Diffable)]
pub enum InternallyTagged {
    AnonymousStruct {
        #[diffable(atomic)]
        atomic: NotDiffable,
    },
}
