#[derive(Facet)]
#[deprecated(since = "0.1.0", note = "Use `MySuperAwesomeStruct` instead")]
pub struct MyLegacyStruct {
    field: String,
}

#[derive(Facet)]
#[deprecated(note = "Use `MySuperAwesomeAlias` instead")]
pub struct MyLegacyAlias(pub u32);

#[derive(Facet)]
#[deprecated(note = "Use `MySuperAwesomeEnum` instead")]
pub enum MyLegacyEnum {
    VariantA,
    VariantB,
    VariantC,
}

#[derive(Facet)]
pub enum MyUnitEnum {
    VariantA,
    VariantB,

    #[deprecated(note = "Use `VariantB` instead")]
    LegacyVariant,
}

#[derive(Facet)]
#[facet(tag = "type")]
pub enum MyInternallyTaggedEnum {
    VariantA {
        field: String,
    },
    VariantB {
        field: u32,
    },

    #[deprecated(note = "Use `VariantA` instead")]
    LegacyVariant {
        field: bool,
    },
}

#[derive(Facet)]
pub enum MyExternallyTaggedEnum {
    VariantA(String),
    VariantB(u32),

    #[deprecated(note = "Use `VariantB` instead")]
    LegacyVariant(bool),
}
