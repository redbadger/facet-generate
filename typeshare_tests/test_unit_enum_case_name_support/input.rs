/// This is a comment.
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub enum Colors {
    Red,
    #[facet(rename = "blue-ish")]
    Blue,
    #[facet(rename = "Green")]
    Green,
}
