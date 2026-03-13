package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
@JsonClassDiscriminator("type")
sealed interface AnonymousStructWithRename {
    @Serializable(with = List.Serializer::class)
    data class List(val list: List<String>): AnonymousStructWithRename {
        object Serializer : KSerializer<List> {
            @Serializable
            private data class Content(val list: List<String>)

            override val descriptor = buildClassSerialDescriptor("list") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: List) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.list)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): List {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.list)
            }
        }
    }

    @Serializable(with = LongFieldNames.Serializer::class)
    data class LongFieldNames(val some_long_field_name: String, val and: Boolean, val but_one_more: List<String>): AnonymousStructWithRename {
        object Serializer : KSerializer<LongFieldNames> {
            @Serializable
            private data class Content(val some_long_field_name: String, val and: Boolean, val but_one_more: List<String>)

            override val descriptor = buildClassSerialDescriptor("longFieldNames") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: LongFieldNames) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.some_long_field_name, value.and, value.but_one_more)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): LongFieldNames {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.some_long_field_name, content.and, content.but_one_more)
            }
        }
    }

    @Serializable(with = KebabCase.Serializer::class)
    data class KebabCase(val another-list: List<String>, val camelCaseStringField: String, val something-else: Boolean): AnonymousStructWithRename {
        object Serializer : KSerializer<KebabCase> {
            @Serializable
            private data class Content(val another-list: List<String>, val camelCaseStringField: String, val something-else: Boolean)

            override val descriptor = buildClassSerialDescriptor("kebabCase") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: KebabCase) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.another-list, value.camelCaseStringField, value.something-else)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): KebabCase {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.another-list, content.camelCaseStringField, content.something-else)
            }
        }
    }
}
