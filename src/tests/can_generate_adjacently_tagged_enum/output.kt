package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

/// Struct comment
@Serializable
data class ExplicitlyNamedStruct(
        /// Field comment
        val a: UInt,
        val b: UInt
)

/// Enum comment
@Serializable
@JsonClassDiscriminator("type")
sealed interface AdvancedColors {
    @Serializable @SerialName("Unit") data object Unit : AdvancedColors

    /// This is a case comment
    @Serializable @SerialName("Str") data class Str(val content: String) : AdvancedColors

    @Serializable @SerialName("Number") data class Number(val content: Int) : AdvancedColors

    @Serializable
    @SerialName("UnsignedNumber")
    data class UnsignedNumber(val content: UInt) : AdvancedColors

    @Serializable
    @SerialName("NumberArray")
    data class NumberArray(val content: List<Int>) : AdvancedColors

    @Serializable(with = TestWithAnonymousStruct.Serializer::class)
    data class TestWithAnonymousStruct(val a: UInt, val b: UInt) : AdvancedColors {
        object Serializer : KSerializer<TestWithAnonymousStruct> {
            @Serializable private data class Content(val a: UInt, val b: UInt)

            override val descriptor =
                    buildClassSerialDescriptor("TestWithAnonymousStruct") {
                        element<Content>("content")
                    }

            override fun serialize(encoder: Encoder, value: TestWithAnonymousStruct) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                            descriptor,
                            0,
                            Content.serializer(),
                            Content(value.a, value.b)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): TestWithAnonymousStruct {
                val content =
                        decoder.decodeStructure(descriptor) {
                            assert(
                                    decodeElementIndex(descriptor) == 0
                            ) // The structure only contains a single index
                            decodeSerializableElement(descriptor, 0, Content.serializer())
                        }

                return TestWithAnonymousStruct(content.a, content.b)
            }
        }
    }

    /// Comment on the last element
    @Serializable
    @SerialName("TestWithExplicitlyNamedStruct")
    data class TestWithExplicitlyNamedStruct(val content: ExplicitlyNamedStruct) : AdvancedColors
}

@Serializable
@JsonClassDiscriminator("type")
sealed interface AdvancedColors2 {
    /// This is a case comment
    @Serializable @SerialName("str") data class Str(val content: String) : AdvancedColors2

    @Serializable @SerialName("number") data class Number(val content: Int) : AdvancedColors2

    @Serializable
    @SerialName("number-array")
    data class NumberArray(val content: List<Int>) : AdvancedColors2

    /// Comment on the last element
    @Serializable
    @SerialName("really-cool-type")
    data class ReallyCoolType(val content: ExplicitlyNamedStruct) : AdvancedColors2
}
