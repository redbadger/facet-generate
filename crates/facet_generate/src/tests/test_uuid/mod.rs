use facet::Facet;
use uuid::Uuid;

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct Foo {
    pub id: Uuid,
    pub maybe_id: Option<Uuid>,
}

crate::test! { Foo for kotlin, swift, typescript, csharp }
