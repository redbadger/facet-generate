use facet::Facet;

#[derive(Facet)]
pub struct MyStruct {
    pub field: String,

    // TODO: #[facet(swift(wrapper = "@Yolo"))]
    pub wrapped_field: String,

    pub another_field: String,
}
