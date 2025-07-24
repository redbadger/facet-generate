#[derive(Facet)]
#[facet(rename_all = "camelCase")]
struct OverrideStruct {
    // These annotations are intentionally inconsistent across languages
    #[facet(
        swift(type = "Int"),
        typescript(type = "any | undefined"),
        kotlin(type = "Int")
    )]
    field_to_override: String,
}

#[derive(Facet)]
#[facet(tag = "type", content = "content")]
enum OverrideEnum {
    UnitVariant,
    TupleVariant(String),
    #[facet(rename_all = "camelCase")]
    AnonymousStructVariant {
        #[facet(
            swift(type = "Int"),
            typescript(type = "any | undefined"),
            kotlin(type = "Int")
        )]
        field_to_override: String,
    },
}
