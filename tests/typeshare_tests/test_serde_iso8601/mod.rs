use chrono::{DateTime, Utc};
use facet::Facet;

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct Foo {
    pub time: DateTime<Utc>,
}
