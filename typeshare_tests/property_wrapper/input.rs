#[derive(Facet)]
pub struct MyStruct {
    pub field: String,

    #[derive(Facet)]
    #[facet(swift(wrapper = "@Yolo"))]
    pub wrapped_field: String,

    pub another_field: String,
}
