#[derive(Facet, Serialize, Debug)]
#[facet(tag = "type", content = "content")]
pub enum SomeEnum {
    /// The associated String contains some opaque context
    Context(
        #[derive(Facet)]
        #[facet(serialized_as = "String")]
        SomeOtherType,
    ),
    Other(i32),
}
