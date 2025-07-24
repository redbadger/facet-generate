#[derive(Facet)]
pub enum SomeResult {
    Ok(u32),
    Error(String),
}

#[derive(Facet)]
pub struct SomeNamedStruct {
    pub a_field: String,
    pub another_field: u32,
}

#[derive(Facet)]
pub enum SomeEnum {
    A { field1: String },
    B { field1: u32, field2: f32 },
    C { field3: Option<bool> },
    D(u32),
    E(SomeNamedStruct),
    F(Option<SomeNamedStruct>),
}
