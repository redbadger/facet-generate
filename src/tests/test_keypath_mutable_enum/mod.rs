use difficient::Diffable;
use facet::Facet;

#[derive(Facet, Clone)]
#[facet(rename_all = "camelCase", tag = "type")]
#[derive(Diffable)]
#[repr(C)]
pub enum InternallyTagged {
    Unit,
    AnonymousStruct { foor: isize, bar: String },
    EmptyStruct {},
    Tuple(String),
}

#[derive(Facet, Diffable, Clone)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum ExternallyTagged {
    AnonymousStruct { foor: isize, bar: String },
    Tuple(String),
}

#[derive(Facet, Diffable, Clone)]
#[facet(rename_all = "camelCase")]
#[repr(C)]
pub enum Unit {
    Foo,
    Bar,
}

// Diffable does not support generic type parameters
// #[derive(Facet, Diffable)]
// #[facet(rename_all = "camelCase")]
// #[repr(C)]
// pub enum Generic<T> {
//     AnonymousStruct { foo: T },
//     Tuple(T),
// }
