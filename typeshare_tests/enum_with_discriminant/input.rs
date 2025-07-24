#[derive(Facet, EnumDiscriminants)]
#[strum_discriminants(name(EffectName))]
#[facet(tag = "name", content = "attributes")]
enum Effect {
    #[facet(rename = "temperature")]
    ColorTemperature(ColorTemperatureAttributes),

    #[facet(rename = "contrast")]
    Contrast(ContrastAttributes),

    #[facet(rename = "exposure")]
    Exposure(ExposureAttributes),
}
