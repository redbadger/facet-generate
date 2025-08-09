use facet::Facet;
use serde::Serialize;

#[derive(Facet, Serialize)]
pub struct Video<'a> {
    pub tags: &'a [Tag],
}

#[derive(Facet, Serialize)]
pub struct Tag;

crate::test! {
    Video for java, kotlin
}
