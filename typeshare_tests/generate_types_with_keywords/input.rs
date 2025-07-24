#[derive(Facet)]
pub struct catch {
    pub default: String,
    pub case: String,
}

#[derive(Facet)]
pub enum throws {
    case,
    default,
}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
pub enum switch {
    default(catch),
}
