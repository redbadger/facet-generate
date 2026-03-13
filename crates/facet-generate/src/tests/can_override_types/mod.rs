#![expect(unused)]
#![expect(clippy::enum_variant_names)]

use facet::Facet;

#[derive(Facet)]
#[facet(rename_all = "camelCase")]
struct OverrideStruct {
    // TODO: re-implement language-specific type overrides
    // #[facet(
    //     swift(type = "Int"),
    //     typescript(type = "any | undefined"),
    //     kotlin(type = "Int")
    // )]
    field_to_override: String,
}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
#[repr(C)]
enum OverrideEnum {
    UnitVariant,
    TupleVariant(String),
    #[facet(rename_all = "camelCase")]
    AnonymousStructVariant {
        // TODO: re-implement language-specific type overrides
        // #[facet(
        //     swift(type = "Int"),
        //     typescript(type = "any | undefined"),
        //     kotlin(type = "Int")
        // )]
        field_to_override: String,
    },
}
