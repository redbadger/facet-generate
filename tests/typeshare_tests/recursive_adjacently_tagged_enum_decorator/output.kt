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
sealed interface Options {
    @Serializable
    @SerialName("red")
    data class Red(val content: Boolean) : Options

    @Serializable
    @SerialName("banana")
    data class Banana(val content: String) : Options

    @Serializable
    @SerialName("vermont")
    data class Vermont(val content: Options) : Options
}

@Serializable
@JsonClassDiscriminator("type")
sealed interface MoreOptions {
    @Serializable
    @SerialName("news")
    data class News(val content: Boolean) : MoreOptions

    @Serializable(with = Exactly.Serializer::class)
    data class Exactly(val config: String): MoreOptions {
        object Serializer : KSerializer<Exactly> {
            @Serializable
            private data class Content(val config: String)

            override val descriptor = buildClassSerialDescriptor("exactly") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: Exactly) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.config)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): Exactly {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.config)
            }
        }
    }

    @Serializable(with = Built.Serializer::class)
    data class Built(val top: MoreOptions): MoreOptions {
        object Serializer : KSerializer<Built> {
            @Serializable
            private data class Content(val top: MoreOptions)

            override val descriptor = buildClassSerialDescriptor("built") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: Built) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.top)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): Built {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.top)
            }
        }
    }
}

