#![expect(unused)]

use facet::Facet;

/// Enum keeping track of who autofilled a field
#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum AutofilledBy {
    /// This field was autofilled by us
    Us {
        /// The UUID for the fill
        uuid: String,
    },
    /// Something else autofilled this field
    SomethingElse {
        /// The UUID for the fill
        uuid: String,
        /// Some other thing
        thing: i32,
    },
}

/// This is a comment (yareek sameek wuz here)
#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
pub enum EnumWithManyVariants {
    UnitVariant,
    TupleVariantString(String),
    AnonVariant { uuid: String },
    TupleVariantInt(i32),
    AnotherUnitVariant,
    AnotherAnonVariant { uuid: String, thing: i32 },
}
