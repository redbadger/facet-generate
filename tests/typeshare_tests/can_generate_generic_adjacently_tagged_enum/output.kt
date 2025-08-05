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

@JsonClassDiscriminator("type")
sealed interface GenericEnum<A, B> {
    @Serializable
    @SerialName("VariantA")
    data class VariantA<A, B>(val content: A) : GenericEnum<A, B>

    @Serializable
    @SerialName("VariantB")
    data class VariantB<A, B>(val content: B) : GenericEnum<A, B>
}

@Serializable
data class StructUsingGenericEnum (
    val enum_field: GenericEnum<String, Short>
)

@JsonClassDiscriminator("type")
sealed interface GenericEnumUsingGenericEnum<T> {
    @Serializable
    @SerialName("VariantC")
    data class VariantC<T>(val content: GenericEnum<T, T>) : GenericEnumUsingGenericEnum<T>

    @Serializable
    @SerialName("VariantD")
    data class VariantD<T>(val content: GenericEnum<String, Map<String, T>>) : GenericEnumUsingGenericEnum<T>

    @Serializable
    @SerialName("VariantE")
    data class VariantE<T>(val content: GenericEnum<String, UInt>) : GenericEnumUsingGenericEnum<T>
}

@JsonClassDiscriminator("type")
sealed interface GenericEnumsUsingStructVariants<T, U> {
    @Serializable(with = VariantF<T, U>.Serializer::class)
    data class VariantF<T, U>(val action: T): GenericEnumsUsingStructVariants<T, U> {
        object Serializer : KSerializer<VariantF<T, U>> {
            @Serializable
            private data class Content(val action: T)

            override val descriptor = buildClassSerialDescriptor("VariantF") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: VariantF<T, U>) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.action)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): VariantF<T, U> {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.action)
            }
        }
    }

    @Serializable(with = VariantG<T, U>.Serializer::class)
    data class VariantG<T, U>(val action: T, val response: U): GenericEnumsUsingStructVariants<T, U> {
        object Serializer : KSerializer<VariantG<T, U>> {
            @Serializable
            private data class Content(val action: T, val response: U)

            override val descriptor = buildClassSerialDescriptor("VariantG") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: VariantG<T, U>) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.action, value.response)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): VariantG<T, U> {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.action, content.response)
            }
        }
    }

    @Serializable(with = VariantH<T, U>.Serializer::class)
    data class VariantH<T, U>(val non_generic: Int): GenericEnumsUsingStructVariants<T, U> {
        object Serializer : KSerializer<VariantH<T, U>> {
            @Serializable
            private data class Content(val non_generic: Int)

            override val descriptor = buildClassSerialDescriptor("VariantH") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: VariantH<T, U>) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.non_generic)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): VariantH<T, U> {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.non_generic)
            }
        }
    }

    @Serializable(with = VariantI<T, U>.Serializer::class)
    data class VariantI<T, U>(val vec: List<T>, val action: MyType<T, U>): GenericEnumsUsingStructVariants<T, U> {
        object Serializer : KSerializer<VariantI<T, U>> {
            @Serializable
            private data class Content(val vec: List<T>, val action: MyType<T, U>)

            override val descriptor = buildClassSerialDescriptor("VariantI") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: VariantI<T, U>) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.vec, value.action)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): VariantI<T, U> {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.vec, content.action)
            }
        }
    }
}

