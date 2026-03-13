package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable data object NamedEmptyStruct

@Serializable(with = Test.Serializer::class)
sealed interface Test {
    @Serializable data class NamedEmptyStruct(val value: com.example.NamedEmptyStruct) : Test

    @Serializable data object AnonymousEmptyStruct : Test

    object Serializer : KSerializer<Test> {
        override val descriptor =
                buildClassSerialDescriptor("Test") {
                    element<NamedEmptyStruct>("NamedEmptyStruct", isOptional = true)
                    element<AnonymousEmptyStruct>("AnonymousEmptyStruct", isOptional = true)
                }

        override fun serialize(encoder: Encoder, value: Test) {
            encoder.encodeStructure(descriptor) {
                when (value) {
                    is NamedEmptyStruct ->
                            encodeSerializableElement(
                                    descriptor,
                                    0,
                                    com.example.NamedEmptyStruct.serializer(),
                                    value.value
                            )
                    is AnonymousEmptyStruct ->
                            encodeSerializableElement(
                                    descriptor,
                                    1,
                                    AnonymousEmptyStruct.serializer(),
                                    value
                            )
                }
            }
        }

        override fun deserialize(decoder: Decoder): Test {
            return decoder.decodeStructure(descriptor) {
                when (val index = decodeElementIndex(descriptor)) {
                    0 -> {
                        val value =
                                decodeSerializableElement(
                                        descriptor,
                                        0,
                                        com.example.NamedEmptyStruct.serializer()
                                )
                        return@decodeStructure NamedEmptyStruct(value)
                    }
                    1 -> {
                        return@decodeStructure decodeSerializableElement(
                                descriptor,
                                1,
                                AnonymousEmptyStruct.serializer()
                        )
                    }
                    else -> throw Exception("Unknown enum variant $index for Test")
                }
            }
        }
    }
}
