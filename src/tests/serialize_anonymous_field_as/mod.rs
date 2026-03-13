use crate as fg;

use facet::Facet;
use serde::Serialize;

#[derive(Facet, Serialize, Debug)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum SomeEnum {
    /// The associated String contains some opaque context
    Context(#[facet(fg::serialized_as = "String")] SomeOtherType),
    Other(i32),
}

#[derive(Facet, Serialize, Debug)]
pub struct SomeOtherType;
