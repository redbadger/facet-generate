#![expect(unused)]

use facet::Facet;

#[derive(Facet)]
#[facet(tag = "type", content = "content", rename_all = "camelCase")]
#[repr(C)]
pub enum Options {
    Red(bool),
    Banana(String),
    Vermont(Box<Self>),
}

#[derive(Facet)]
#[facet(tag = "type", content = "content", rename_all = "camelCase")]
#[repr(C)]
pub enum MoreOptions {
    News(bool),
    Exactly { config: String },
    Built { top: Box<Self> },
}
