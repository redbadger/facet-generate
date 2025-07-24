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
data object NamedEmptyStruct

@Serializable
@JsonClassDiscriminator("type")
sealed interface Test {
    @Serializable
    @SerialName("NamedEmptyStruct")
    data class NamedEmptyStruct(val content: com.photoroom.engine.NamedEmptyStruct) : Test

    @Serializable(with = AnonymousEmptyStruct.Serializer::class)
    data object AnonymousEmptyStruct : Test {
        object Serializer : KSerializer<AnonymousEmptyStruct> {
            @Serializable
            private data object Content

            override val descriptor = buildClassSerialDescriptor("AnonymousEmptyStruct") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: AnonymousEmptyStruct) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content
                    )
                }
            }

            override fun deserialize(decoder: Decoder): AnonymousEmptyStruct {
                // Even though the structure contains nothing of value, we need to consume it
                decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return AnonymousEmptyStruct
            }
        }
    }

    @Serializable
    @SerialName("NoStruct")
    data object NoStruct : Test
}

