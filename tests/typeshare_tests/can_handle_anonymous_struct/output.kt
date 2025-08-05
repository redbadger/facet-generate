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

/// Enum keeping track of who autofilled a field
@Serializable
@JsonClassDiscriminator("type")
sealed interface AutofilledBy {
    /// This field was autofilled by us
    @Serializable(with = Us.Serializer::class)
    data class Us(val uuid: String): AutofilledBy {
        object Serializer : KSerializer<Us> {
            @Serializable
            private data class Content(val uuid: String)

            override val descriptor = buildClassSerialDescriptor("Us") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: Us) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.uuid)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): Us {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.uuid)
            }
        }
    }

    /// Something else autofilled this field
    @Serializable(with = SomethingElse.Serializer::class)
    data class SomethingElse(val uuid: String, val thing: Int): AutofilledBy {
        object Serializer : KSerializer<SomethingElse> {
            @Serializable
            private data class Content(val uuid: String, val thing: Int)

            override val descriptor = buildClassSerialDescriptor("SomethingElse") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: SomethingElse) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.uuid, value.thing)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): SomethingElse {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.uuid, content.thing)
            }
        }
    }
}

/// This is a comment (yareek sameek wuz here)
@Serializable
@JsonClassDiscriminator("type")
sealed interface EnumWithManyVariants {
    @Serializable
    @SerialName("UnitVariant")
    data object UnitVariant : EnumWithManyVariants

    @Serializable
    @SerialName("TupleVariantString")
    data class TupleVariantString(val content: String) : EnumWithManyVariants

    @Serializable(with = AnonVariant.Serializer::class)
    data class AnonVariant(val uuid: String): EnumWithManyVariants {
        object Serializer : KSerializer<AnonVariant> {
            @Serializable
            private data class Content(val uuid: String)

            override val descriptor = buildClassSerialDescriptor("AnonVariant") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: AnonVariant) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.uuid)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): AnonVariant {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.uuid)
            }
        }
    }

    @Serializable
    @SerialName("TupleVariantInt")
    data class TupleVariantInt(val content: Int) : EnumWithManyVariants

    @Serializable
    @SerialName("AnotherUnitVariant")
    data object AnotherUnitVariant : EnumWithManyVariants

    @Serializable(with = AnotherAnonVariant.Serializer::class)
    data class AnotherAnonVariant(val uuid: String, val thing: Int): EnumWithManyVariants {
        object Serializer : KSerializer<AnotherAnonVariant> {
            @Serializable
            private data class Content(val uuid: String, val thing: Int)

            override val descriptor = buildClassSerialDescriptor("AnotherAnonVariant") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: AnotherAnonVariant) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.uuid, value.thing)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): AnotherAnonVariant {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.uuid, content.thing)
            }
        }
    }
}

