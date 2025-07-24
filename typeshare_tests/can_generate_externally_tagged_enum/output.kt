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
data class SomeNamedStruct (
    val a_field: String,
    val another_field: UInt
)

@Serializable(with = SomeResult.Serializer::class)
sealed interface SomeResult {
    @Serializable
    data class Ok(val value: UInt) : SomeResult

    @Serializable
    data class Error(val value: String) : SomeResult

    object Serializer : KSerializer<SomeResult> {
        override val descriptor = buildClassSerialDescriptor("SomeResult") {
            element<Ok>("Ok", isOptional = true)
            element<Error>("Error", isOptional = true)
        }

        override fun serialize(encoder: Encoder, value: SomeResult) {
            encoder.encodeStructure(descriptor) {
                when (value) { 
                    is Ok -> encodeSerializableElement(
                        descriptor,
                        0,
                        UInt.serializer(),
                        value.value
                    )

                    is Error -> encodeSerializableElement(
                        descriptor,
                        1,
                        String.serializer(),
                        value.value
                    )
                }
            }
        }

        override fun deserialize(decoder: Decoder): SomeResult {
            return decoder.decodeStructure(descriptor) {
                when (val index = decodeElementIndex(descriptor)) { 
                    0 -> {
                        val value = decodeSerializableElement(descriptor, 0, UInt.serializer())
                        return@decodeStructure Ok(value)
                    }

                    1 -> {
                        val value = decodeSerializableElement(descriptor, 1, String.serializer())
                        return@decodeStructure Error(value)
                    }
                    else -> throw Exception("Unknown enum variant $index for SomeResult")
                }
            }
        }
    }
}

@Serializable(with = SomeEnum.Serializer::class)
sealed interface SomeEnum {
    @Serializable
    data class A(val field1: String) : SomeEnum

    @Serializable
    data class B(val field1: UInt, val field2: Float) : SomeEnum

    @Serializable
    data class C(val field3: Boolean? = null) : SomeEnum

    @Serializable
    data class D(val value: UInt) : SomeEnum

    @Serializable
    data class E(val value: SomeNamedStruct) : SomeEnum

    @Serializable
    data class F(val value: SomeNamedStruct?) : SomeEnum

    object Serializer : KSerializer<SomeEnum> {
        override val descriptor = buildClassSerialDescriptor("SomeEnum") {
            element<A>("A", isOptional = true)
            element<B>("B", isOptional = true)
            element<C>("C", isOptional = true)
            element<D>("D", isOptional = true)
            element<E>("E", isOptional = true)
            element<F>("F", isOptional = true)
        }

        override fun serialize(encoder: Encoder, value: SomeEnum) {
            encoder.encodeStructure(descriptor) {
                when (value) { 
                    is A -> encodeSerializableElement(
                        descriptor,
                        0,
                        A.serializer(),
                        value
                    )

                    is B -> encodeSerializableElement(
                        descriptor,
                        1,
                        B.serializer(),
                        value
                    )

                    is C -> encodeSerializableElement(
                        descriptor,
                        2,
                        C.serializer(),
                        value
                    )

                    is D -> encodeSerializableElement(
                        descriptor,
                        3,
                        UInt.serializer(),
                        value.value
                    )

                    is E -> encodeSerializableElement(
                        descriptor,
                        4,
                        SomeNamedStruct.serializer(),
                        value.value
                    )

                    is F -> encodeNullableSerializableElement(
                        descriptor,
                        5,
                        SomeNamedStruct.serializer(),
                        value.value
                    )
                }
            }
        }

        override fun deserialize(decoder: Decoder): SomeEnum {
            return decoder.decodeStructure(descriptor) {
                when (val index = decodeElementIndex(descriptor)) { 
                    0 -> {
                        return@decodeStructure decodeSerializableElement(descriptor, 0, A.serializer())
                    }

                    1 -> {
                        return@decodeStructure decodeSerializableElement(descriptor, 1, B.serializer())
                    }

                    2 -> {
                        return@decodeStructure decodeSerializableElement(descriptor, 2, C.serializer())
                    }

                    3 -> {
                        val value = decodeSerializableElement(descriptor, 3, UInt.serializer())
                        return@decodeStructure D(value)
                    }

                    4 -> {
                        val value = decodeSerializableElement(descriptor, 4, SomeNamedStruct.serializer())
                        return@decodeStructure E(value)
                    }

                    5 -> {
                        val value = decodeNullableSerializableElement(descriptor, 5, SomeNamedStruct.serializer())
                        return@decodeStructure F(value)
                    }
                    else -> throw Exception("Unknown enum variant $index for SomeEnum")
                }
            }
        }
    }
}

