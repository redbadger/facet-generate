#![expect(unused)]

use facet::Facet;

#[derive(Facet)]
#[allow(non_camel_case_types)]
pub struct catch {
    pub default: String,
    pub case: String,
}

#[derive(Facet)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum throws {
    case,
    default,
}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum switch {
    default(catch),
}
