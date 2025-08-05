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

@Deprecated(message = "Use `MySuperAwesomeAlias` instead")
typealias MyLegacyAlias = UInt

@Deprecated(message = "Use `MySuperAwesomeStruct` instead")
@Serializable
data class MyLegacyStruct (
    val field: String
)

@Deprecated(message = "Use `MySuperAwesomeEnum` instead")
@Serializable
enum class MyLegacyEnum {
    @SerialName("VariantA")
    VARIANT_A,

    @SerialName("VariantB")
    VARIANT_B,

    @SerialName("VariantC")
    VARIANT_C;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}

@Serializable
enum class MyUnitEnum {
    @SerialName("VariantA")
    VARIANT_A,

    @SerialName("VariantB")
    VARIANT_B,

    @Deprecated(message = "Use `VariantB` instead")
    @SerialName("LegacyVariant")
    LEGACY_VARIANT;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}

@Serializable
@JsonClassDiscriminator("type")
sealed interface MyInternallyTaggedEnum {
    @Serializable
    @SerialName("VariantA")
    data class VariantA(val field: String) : MyInternallyTaggedEnum

    @Serializable
    @SerialName("VariantB")
    data class VariantB(val field: UInt) : MyInternallyTaggedEnum

    @Deprecated(message = "Use `VariantA` instead")
    @Serializable
    @SerialName("LegacyVariant")
    data class LegacyVariant(val field: Boolean) : MyInternallyTaggedEnum
}

@Serializable(with = MyExternallyTaggedEnum.Serializer::class)
sealed interface MyExternallyTaggedEnum {
    @Serializable
    data class VariantA(val value: String) : MyExternallyTaggedEnum

    @Serializable
    data class VariantB(val value: UInt) : MyExternallyTaggedEnum

    @Deprecated(message = "Use `VariantB` instead")
    @Serializable
    data class LegacyVariant(val value: Boolean) : MyExternallyTaggedEnum

    object Serializer : KSerializer<MyExternallyTaggedEnum> {
        override val descriptor = buildClassSerialDescriptor("MyExternallyTaggedEnum") {
            element<VariantA>("VariantA", isOptional = true)
            element<VariantB>("VariantB", isOptional = true)
            element<LegacyVariant>("LegacyVariant", isOptional = true)
        }

        override fun serialize(encoder: Encoder, value: MyExternallyTaggedEnum) {
            encoder.encodeStructure(descriptor) {
                when (value) { 
                    is VariantA -> encodeSerializableElement(
                        descriptor,
                        0,
                        String.serializer(),
                        value.value
                    )

                    is VariantB -> encodeSerializableElement(
                        descriptor,
                        1,
                        UInt.serializer(),
                        value.value
                    )

                    is LegacyVariant -> encodeSerializableElement(
                        descriptor,
                        2,
                        Boolean.serializer(),
                        value.value
                    )
                }
            }
        }

        override fun deserialize(decoder: Decoder): MyExternallyTaggedEnum {
            return decoder.decodeStructure(descriptor) {
                when (val index = decodeElementIndex(descriptor)) { 
                    0 -> {
                        val value = decodeSerializableElement(descriptor, 0, String.serializer())
                        return@decodeStructure VariantA(value)
                    }

                    1 -> {
                        val value = decodeSerializableElement(descriptor, 1, UInt.serializer())
                        return@decodeStructure VariantB(value)
                    }

                    2 -> {
                        val value = decodeSerializableElement(descriptor, 2, Boolean.serializer())
                        return@decodeStructure LegacyVariant(value)
                    }
                    else -> throw Exception("Unknown enum variant $index for MyExternallyTaggedEnum")
                }
            }
        }
    }
}

