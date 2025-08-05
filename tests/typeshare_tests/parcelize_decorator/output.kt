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
@Parcelize
data class Struct (
    val field1: String,
    val field2: UInt
): Parcelable

@Parcelize
@Serializable
enum class UnitEnum: Parcelable {
    @SerialName("VariantA")
    VARIANT_A,

    @SerialName("VariantB")
    VARIANT_B,

    @SerialName("VariantC")
    VARIANT_C;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}

@Parcelize
@Serializable(with = ExternallyTaggedEnum.Serializer::class)
sealed interface ExternallyTaggedEnum: Parcelable {
    @Serializable
    data class TupleVariant(val value: String) : ExternallyTaggedEnum

    @Serializable
    data class StructVariant(val field: String) : ExternallyTaggedEnum

    object Serializer : KSerializer<ExternallyTaggedEnum> {
        override val descriptor = buildClassSerialDescriptor("ExternallyTaggedEnum") {
            element<TupleVariant>("TupleVariant", isOptional = true)
            element<StructVariant>("StructVariant", isOptional = true)
        }

        override fun serialize(encoder: Encoder, value: ExternallyTaggedEnum) {
            encoder.encodeStructure(descriptor) {
                when (value) { 
                    is TupleVariant -> encodeSerializableElement(
                        descriptor,
                        0,
                        String.serializer(),
                        value.value
                    )

                    is StructVariant -> encodeSerializableElement(
                        descriptor,
                        1,
                        StructVariant.serializer(),
                        value
                    )
                }
            }
        }

        override fun deserialize(decoder: Decoder): ExternallyTaggedEnum {
            return decoder.decodeStructure(descriptor) {
                when (val index = decodeElementIndex(descriptor)) { 
                    0 -> {
                        val value = decodeSerializableElement(descriptor, 0, String.serializer())
                        return@decodeStructure TupleVariant(value)
                    }

                    1 -> {
                        return@decodeStructure decodeSerializableElement(descriptor, 1, StructVariant.serializer())
                    }
                    else -> throw Exception("Unknown enum variant $index for ExternallyTaggedEnum")
                }
            }
        }
    }
}

@Parcelize
@Serializable(with = InternallyTaggedEnum.Serializer::class)
@JsonClassDiscriminator("type")
sealed interface InternallyTaggedEnum: Parcelable {
    @Serializable
    @SerialName("UnitVariant")
    data object UnitVariant : InternallyTaggedEnum

    @Serializable
    @SerialName("TupleVariant")
    data class TupleVariant(val value: String) : InternallyTaggedEnum

    @Serializable
    @SerialName("StructVariant")
    data class StructVariant(val field: String) : InternallyTaggedEnum

    object Serializer : KSerializer<InternallyTaggedEnum> {
        override val descriptor = buildClassSerialDescriptor("InternallyTaggedEnum") {
            element<UnitVariant>("UnitVariant", isOptional = true)
            element<TupleVariant>("TupleVariant", isOptional = true)
            element<StructVariant>("StructVariant", isOptional = true)
        }

        override fun serialize(encoder: Encoder, value: InternallyTaggedEnum) {
            require(encoder is JsonEncoder)
            when (value) { 
                is UnitVariant -> {
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("UnitVariant"))
                    val element = JsonObject(base)
                    encoder.encodeJsonElement(element)
                }
                is TupleVariant -> {
                    val content = encoder.json.encodeToJsonElement(String.serializer(), value.value) as JsonObject
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("TupleVariant"))
                    val element = JsonObject(base.apply { putAll(content.toMap()) })
                    encoder.encodeJsonElement(element)
                }
                is StructVariant -> {
                    val content = encoder.json.encodeToJsonElement(StructVariant.serializer(), value) as JsonObject
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("StructVariant"))
                    val element = JsonObject(base.apply { putAll(content.toMap()) })
                    encoder.encodeJsonElement(element)
                }
            }
        }

        override fun deserialize(decoder: Decoder): InternallyTaggedEnum {
            require(decoder is JsonDecoder)
            val container = decoder.decodeJsonElement()
            return when (val discriminator = container.jsonObject["type"]?.jsonPrimitive?.content) { 
                "UnitVariant" -> UnitVariant
                "TupleVariant" -> {
                    val content = decoder.json.decodeFromJsonElement<String>(container)
                    TupleVariant(content)
                }
                "StructVariant" -> {
                    decoder.json.decodeFromJsonElement<StructVariant>(container)
                }
                else -> throw Exception("Unknown enum variant $discriminator for InternallyTaggedEnum")
            }
        }
    }
}

@Parcelize
@Serializable
@JsonClassDiscriminator("type")
sealed interface AdjacentlyTaggedEnum: Parcelable {
    @Serializable
    @SerialName("UnitVariant")
    data object UnitVariant : AdjacentlyTaggedEnum

    @Serializable
    @SerialName("TupleVariant")
    data class TupleVariant(val content: String) : AdjacentlyTaggedEnum

    @Serializable(with = StructVariant.Serializer::class)
    data class StructVariant(val field: String): AdjacentlyTaggedEnum {
        object Serializer : KSerializer<StructVariant> {
            @Serializable
            private data class Content(val field: String)

            override val descriptor = buildClassSerialDescriptor("StructVariant") {
                element<Content>("content")
            }

            override fun serialize(encoder: Encoder, value: StructVariant) {
                encoder.encodeStructure(descriptor) {
                    encodeSerializableElement(
                        descriptor,
                        0,
                        Content.serializer(),
                        Content(value.field)
                    )
                }
            }

            override fun deserialize(decoder: Decoder): StructVariant {
                val content = decoder.decodeStructure(descriptor) {
                    assert(decodeElementIndex(descriptor) == 0) // The structure only contains a single index
                    decodeSerializableElement(descriptor, 0, Content.serializer())
                }

                return TestWithAnonymousStruct(content.field)
            }
        }
    }
}

