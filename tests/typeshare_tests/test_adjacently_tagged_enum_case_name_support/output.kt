package com.photoroom.engine

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*
import com.photoroom.engine.photogossip.interfaces.*
import com.photoroom.engine.photogossip.extensions.*
import com.photoroom.engine.misc.EngineSerialization
import com.photoroom.engine.photogossip.PatchOperation

@Serializable
data object ItemDetailsFieldValue

@Serializable
@JsonClassDiscriminator("type")
sealed interface AdvancedColors {
    @Serializable
    @SerialName("str")
    data class Str(val content: String) : AdvancedColors

    @Serializable
    @SerialName("number")
    data class Number(val content: Int) : AdvancedColors

    @Serializable
    @SerialName("number-array")
    data class NumberArray(val content: List<Int>) : AdvancedColors

    @Serializable
    @SerialName("reallyCoolType")
    data class ReallyCoolType(val content: ItemDetailsFieldValue) : AdvancedColors
}

