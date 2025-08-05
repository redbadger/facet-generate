use facet::Facet;

#[derive(Facet)]
#[repr(C)]
pub enum ExternallyTaggedEnum {
    #[facet(rename = "Some Variant")]
    SomeVariant {
        #[facet(rename = "Variant Field")]
        variant_field: bool,
    },
}

#[derive(Facet)]
#[facet(tag = "type")]
#[repr(C)]
pub enum InternallyTaggedEnum {
    #[facet(rename = "Some Variant")]
    SomeVariant {
        #[facet(rename = "Variant Field")]
        variant_field: bool,
    },
}

#[derive(Facet)]
#[facet(tag = "name", content = "properties")]
#[repr(C)]
pub enum AdjacentlyTaggedEnum {
    #[facet(rename = "Some Variant")]
    SomeVariant {
        #[facet(rename = "Variant Field")]
        variant_field: bool,
    },
}
