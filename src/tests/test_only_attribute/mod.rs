#![expect(unused)]
#![expect(clippy::struct_field_names)]
#![expect(clippy::enum_variant_names)]

use facet::Facet;

#[derive(Facet)]
// TODO: #[facet(swift(only))]
pub struct StructOnlyInSwift {
    field: String,
}

#[derive(Facet)]
// TODO: #[facet(kotlin(only))]
pub struct StructOnlyInKotlin {
    field: String,
}

#[derive(Facet)]
// TODO: #[facet(typescript(only))]
pub struct StructOnlyInTypeScript {
    field: String,
}

#[derive(Facet)]
pub struct Struct {
    // TODO: #[facet(swift(only))]
    pub only_in_swift: String,

    // TODO: #[facet(kotlin(only))]
    pub only_in_kotlin: String,

    // TODO: #[facet(typescript(only))]
    pub only_in_typescript: String,
}

#[derive(Facet)]
#[repr(C)]
pub enum Enum {
    // TODO: #[facet(swift(only))]
    OnlyInSwift(String),

    // TODO: #[facet(kotlin(only))]
    OnlyInKotlin(String),

    // TODO: #[facet(typescript(only))]
    OnlyInTypeScript(String),
}
