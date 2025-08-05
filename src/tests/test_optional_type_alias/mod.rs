use facet::Facet;

pub type OptionalU32 = Option<u32>;

#[derive(Facet)]
pub struct OptionalU16(Option<u16>);

#[derive(Facet)]
pub struct FooBar {
    foo: OptionalU32,
    bar: OptionalU16,
}
