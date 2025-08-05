package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable data class OverrideStruct(val fieldToOverride: Int)

@Serializable
@JsonClassDiscriminator("type")
sealed interface OverrideEnum {
    @Serializable @SerialName("UnitVariant") data object UnitVariant : OverrideEnum

    @Serializable
    @SerialName("TupleVariant")
    data class TupleVariant(val content: String) : OverrideEnum

    @Serializable(with = AnonymousStructVariant.Serializer::class)
    data class AnonymousStructVariant(val fieldToOverride: String) : OverrideEnum {
        object Serializer : KSerializer<AnonymousStructVariant> {
            @Serializable private data class Content(val fieldToOverride: String)

            override val descriptor =
                    buildClassSerialDescriptor("AnonymousStructVariant") {
                        element<Content>("content")
                    }

            override fun serialize(encoder: Encoder, value: AnonymousStructVariant) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                            descriptor,
                            0,
                            Content.serializer(),
                            Content(value.fieldToOverride)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): AnonymousStructVariant {
                val content =
                        decoder.decodeStructure(descriptor) {
                            assert(
                                    decodeElementIndex(descriptor) == 0
                            ) // The structure only contains a single index
                            decodeSerializableElement(descriptor, 0, Content.serializer())
                        }

                return TestWithAnonymousStruct(content.fieldToOverride)
            }
        }
    }
}
