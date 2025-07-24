#[derive(Facet)]
#[facet(tag = "type", content = "content")]
pub enum SomeEnum {
    A,
    #[facet(skip)]
    B,
    C(i32),
    #[facet(skip, asdf)]
    D(u32),
    #[facet(skip)]
    E,
}
