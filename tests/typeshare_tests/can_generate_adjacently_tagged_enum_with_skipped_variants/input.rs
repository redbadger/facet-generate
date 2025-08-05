use facet::Facet;

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
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
