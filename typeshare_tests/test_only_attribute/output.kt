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
data class StructOnlyInKotlin (
    val field: String
)

@Serializable
data class Struct (
    val only_in_kotlin: String
)

@Serializable(with = Enum.Serializer::class)
sealed interface Enum {
    @Serializable
    data class OnlyInKotlin(val value: String) : Enum

    object Serializer : KSerializer<Enum> {
        override val descriptor = buildClassSerialDescriptor("Enum") {
            element<OnlyInKotlin>("OnlyInKotlin", isOptional = true)
        }

        override fun serialize(encoder: Encoder, value: Enum) {
            encoder.encodeStructure(descriptor) {
                when (value) { 
                    is OnlyInKotlin -> encodeSerializableElement(
                        descriptor,
                        0,
                        String.serializer(),
                        value.value
                    )
                }
            }
        }

        override fun deserialize(decoder: Decoder): Enum {
            return decoder.decodeStructure(descriptor) {
                when (val index = decodeElementIndex(descriptor)) { 
                    0 -> {
                        val value = decodeSerializableElement(descriptor, 0, String.serializer())
                        return@decodeStructure OnlyInKotlin(value)
                    }
                    else -> throw Exception("Unknown enum variant $index for Enum")
                }
            }
        }
    }
}

