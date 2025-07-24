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
@JsonClassDiscriminator("type")
sealed interface AdvancedEnum {
    @Serializable
    @SerialName("unitVariant")
    data object UnitVariant : AdvancedEnum

    @Serializable
    @SerialName("A")
    data class AnonymousStruct(val field1: String) : AdvancedEnum

    @Serializable
    @SerialName("otherAnonymousStruct")
    data class OtherAnonymousStruct(val field1: UInt, val field2: Float) : AdvancedEnum

    @Serializable
    @SerialName("B")
    data class Rename(val field3: Boolean? = null) : AdvancedEnum
}

