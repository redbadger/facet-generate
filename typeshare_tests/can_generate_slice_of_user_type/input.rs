#[derive(Facet, Serialize)]
pub struct Video<'a> {
    pub tags: &'a [Tag],
}
