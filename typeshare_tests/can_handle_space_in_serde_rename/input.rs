#[derive(Facet)]
pub enum ExternallyTaggedEnum {
    #[facet(rename = "Some Variant")]
    SomeVariant {
        #[facet(rename = "Variant Field")]
        variant_field: bool,
    },
}

#[derive(Facet)]
#[facet(tag = "type")]
pub enum InternallyTaggedEnum {
    #[facet(rename = "Some Variant")]
    SomeVariant {
        #[facet(rename = "Variant Field")]
        variant_field: bool,
    },
}

#[derive(Facet)]
#[facet(tag = "name", content = "properties")]
pub enum AdjacentlyTaggedEnum {
    #[facet(rename = "Some Variant")]
    SomeVariant {
        #[facet(rename = "Variant Field")]
        variant_field: bool,
    },
}
