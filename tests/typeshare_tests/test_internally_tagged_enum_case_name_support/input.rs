use facet::Facet;

#[derive(Facet)]
#[facet(rename_all = "camelCase", tag = "type")]
#[repr(C)]
pub enum AdvancedEnum {
    UnitVariant,
    #[facet(rename = "A")]
    AnonymousStruct {
        field1: String,
    },
    OtherAnonymousStruct {
        field1: u32,
        field2: f32,
    },
    #[facet(rename = "B")]
    Rename {
        field3: Option<bool>,
    },
}
