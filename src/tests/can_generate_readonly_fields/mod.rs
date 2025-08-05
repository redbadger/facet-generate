use facet::Facet;

#[derive(Facet)]
pub struct SomeStruct {
    #[facet(readonly)]
    field_a: u32,
    #[facet(readonly)]
    field_b: Vec<String>,
}
