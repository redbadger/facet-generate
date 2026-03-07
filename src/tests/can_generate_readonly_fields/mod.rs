use facet::Facet;

#[derive(Facet)]
pub struct SomeStruct {
    #[facet(fg::readonly)]
    field_a: u32,
    #[facet(fg::readonly)]
    field_b: Vec<String>,
}
