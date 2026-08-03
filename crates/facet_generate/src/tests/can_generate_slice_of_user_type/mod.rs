use facet::Facet;
use serde::Serialize;

#[derive(Facet, Serialize)]
pub struct Video<'a> {
    pub tags: &'a [Tag],
}

#[derive(Facet, Serialize)]
pub struct Tag;

// TODO: enable swift
crate::test! {
    Video for kotlin, typescript
}
