package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
@JsonClassDiscriminator("name")
sealed interface Effect {
    @Serializable
    @SerialName("temperature")
    data class ColorTemperature(val attributes: ColorTemperatureAttributes) : Effect

    @Serializable
    @SerialName("contrast")
    data class Contrast(val attributes: ContrastAttributes) : Effect

    @Serializable
    @SerialName("exposure")
    data class Exposure(val attributes: ExposureAttributes) : Effect

    val name: EffectName
        get() =
                when (this) {
                    is ColorTemperature -> EffectName.COLOR_TEMPERATURE
                    is Contrast -> EffectName.CONTRAST
                    is Exposure -> EffectName.EXPOSURE
                }
}

@Serializable
enum class EffectName {
    @SerialName("temperature") COLOR_TEMPERATURE,
    @SerialName("contrast") CONTRAST,
    @SerialName("exposure") EXPOSURE
}
