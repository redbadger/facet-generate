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
data class GenericStruct<A, B> (
    val field_a: A,
    val field_b: List<B>
)

@Serializable
data class GenericStructUsingGenericStruct<T> (
    val struct_field: GenericStruct<String, T>,
    val second_struct_field: GenericStruct<T, String>,
    val third_struct_field: GenericStruct<T, List<T>>
)

@Serializable
@JsonClassDiscriminator("type")
sealed interface EnumUsingGenericStruct {
    @Serializable
    @SerialName("VariantA")
    data class VariantA(val content: GenericStruct<String, Float>) : EnumUsingGenericStruct

    @Serializable
    @SerialName("VariantB")
    data class VariantB(val content: GenericStruct<String, Int>) : EnumUsingGenericStruct

    @Serializable
    @SerialName("VariantC")
    data class VariantC(val content: GenericStruct<String, Boolean>) : EnumUsingGenericStruct

    @Serializable
    @SerialName("VariantD")
    data class VariantD(val content: GenericStructUsingGenericStruct<Unit>) : EnumUsingGenericStruct
}

