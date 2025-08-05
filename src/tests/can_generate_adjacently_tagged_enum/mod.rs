#![expect(unused)]

use facet::Facet;

/// Struct comment
#[derive(Facet)]
pub struct ExplicitlyNamedStruct {
    /// Field comment
    pub a: u32,
    pub b: u32,
}

/// Enum comment
#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum AdvancedColors {
    Unit,
    /// This is a case comment
    Str(String),
    Number(i32),
    UnsignedNumber(u32),
    NumberArray(Vec<i32>),
    TestWithAnonymousStruct {
        a: u32,
        b: u32,
    },
    /// Comment on the last element
    TestWithExplicitlyNamedStruct(ExplicitlyNamedStruct),
}

#[derive(Facet)]
#[facet(tag = "type", content = "content", rename_all = "kebab-case")]
#[repr(C)]
pub enum AdvancedColors2 {
    /// This is a case comment
    Str(String),
    Number(i32),
    NumberArray(Vec<i32>),
    /// Comment on the last element
    ReallyCoolType(ExplicitlyNamedStruct),
}
