/// This is a comment.
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct Things {
    pub bla: String,
    #[facet(rename = "label")]
    pub some_label: Option<String>,
    #[facet(rename = "label-left")]
    pub label_left: Option<String>,
}
