#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct E {
    depends_on: D,
}

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct D {
    depends_on: C,
    also_depends_on: Option<E>,
}

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct C {
    depends_on: B,
}

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct B {
    depends_on: A,
}

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
pub struct A {
    field: u32,
}
