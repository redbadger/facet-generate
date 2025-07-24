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
data class ExplicitlyNamedStruct (
    val a_field: String,
    val another_field: UInt
)

@Serializable(with = SomeEnum.Serializer::class)
@JsonClassDiscriminator("type")
sealed interface SomeEnum {
    @Serializable
    @SerialName("A")
    data object A : SomeEnum

    @Serializable
    @SerialName("B")
    data class B(val field1: String) : SomeEnum

    @Serializable
    @SerialName("C")
    data class C(val field1: UInt, val field2: Float) : SomeEnum

    @Serializable
    @SerialName("D")
    data class D(val field3: Boolean? = null) : SomeEnum

    @Serializable
    @SerialName("E")
    data class E(val value: ExplicitlyNamedStruct) : SomeEnum

    object Serializer : KSerializer<SomeEnum> {
        override val descriptor = buildClassSerialDescriptor("SomeEnum") {
            element<A>("A", isOptional = true)
            element<B>("B", isOptional = true)
            element<C>("C", isOptional = true)
            element<D>("D", isOptional = true)
            element<E>("E", isOptional = true)
        }

        override fun serialize(encoder: Encoder, value: SomeEnum) {
            require(encoder is JsonEncoder)
            when (value) { 
                is A -> {
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("A"))
                    val element = JsonObject(base)
                    encoder.encodeJsonElement(element)
                }
                is B -> {
                    val content = encoder.json.encodeToJsonElement(B.serializer(), value) as JsonObject
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("B"))
                    val element = JsonObject(base.apply { putAll(content.toMap()) })
                    encoder.encodeJsonElement(element)
                }
                is C -> {
                    val content = encoder.json.encodeToJsonElement(C.serializer(), value) as JsonObject
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("C"))
                    val element = JsonObject(base.apply { putAll(content.toMap()) })
                    encoder.encodeJsonElement(element)
                }
                is D -> {
                    val content = encoder.json.encodeToJsonElement(D.serializer(), value) as JsonObject
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("D"))
                    val element = JsonObject(base.apply { putAll(content.toMap()) })
                    encoder.encodeJsonElement(element)
                }
                is E -> {
                    val content = encoder.json.encodeToJsonElement(ExplicitlyNamedStruct.serializer(), value.value) as JsonObject
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("E"))
                    val element = JsonObject(base.apply { putAll(content.toMap()) })
                    encoder.encodeJsonElement(element)
                }
            }
        }

        override fun deserialize(decoder: Decoder): SomeEnum {
            require(decoder is JsonDecoder)
            val container = decoder.decodeJsonElement()
            return when (val discriminator = container.jsonObject["type"]?.jsonPrimitive?.content) { 
                "A" -> A
                "B" -> {
                    decoder.json.decodeFromJsonElement<B>(container)
                }
                "C" -> {
                    decoder.json.decodeFromJsonElement<C>(container)
                }
                "D" -> {
                    decoder.json.decodeFromJsonElement<D>(container)
                }
                "E" -> {
                    val content = decoder.json.decodeFromJsonElement<ExplicitlyNamedStruct>(container)
                    E(content)
                }
                else -> throw Exception("Unknown enum variant $discriminator for SomeEnum")
            }
        }
    }
}

