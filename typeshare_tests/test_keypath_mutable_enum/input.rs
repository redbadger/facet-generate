#[derive(Facet)]
#[facet(rename_all = "camelCase", tag = "type")]
#[derive(Diffable)]
pub enum InternallyTagged {
    Unit,
    AnonymousStruct { foor: Int, bar: String },
    EmptyStruct {},
    Tuple(String),
}

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
#[derive(Diffable)]
pub enum ExternallyTagged {
    AnonymousStruct { foor: Int, bar: String },
    Tuple(String),
}

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
#[derive(Diffable)]
pub enum Unit {
    Foo,
    Bar,
}

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
#[derive(Diffable)]
pub enum Generic<T> {
    AnonymousStruct { foo: T },
    Tuple(T),
}
