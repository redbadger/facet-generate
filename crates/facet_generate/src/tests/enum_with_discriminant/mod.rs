#![expect(unused)]

use facet::Facet;
use strum::EnumDiscriminants;

#[derive(Facet, EnumDiscriminants)]
#[strum_discriminants(name(EffectName))]
#[facet(tag = "name", content = "attributes")]
#[repr(C)]
enum Effect {
    #[facet(rename = "temperature")]
    ColorTemperature(ColorTemperatureAttributes),

    #[facet(rename = "contrast")]
    Contrast(ContrastAttributes),

    #[facet(rename = "exposure")]
    Exposure(ExposureAttributes),
}

#[derive(Facet)]
struct ColorTemperatureAttributes {
    value: f32,
}

#[derive(Facet)]
struct ContrastAttributes {
    value: f32,
}

#[derive(Facet)]
struct ExposureAttributes {
    value: f32,
}
