#[derive(Facet)]
#[facet(swift(only))]
pub struct StructOnlyInSwift {
    field: String,
}

#[derive(Facet)]
#[facet(kotlin(only))]
pub struct StructOnlyInKotlin {
    field: String,
}

#[derive(Facet)]
#[facet(typescript(only))]
pub struct StructOnlyInTypeScript {
    field: String,
}

#[derive(Facet)]
pub struct Struct {
    #[facet(swift(only))]
    pub only_in_swift: String,

    #[facet(kotlin(only))]
    pub only_in_kotlin: String,

    #[facet(typescript(only))]
    pub only_in_typescript: String,
}

#[derive(Facet)]
pub enum Enum {
    #[facet(swift(only))]
    OnlyInSwift(String),

    #[facet(kotlin(only))]
    OnlyInKotlin(String),

    #[facet(typescript(only))]
    OnlyInTypeScript(String),
}
