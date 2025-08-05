package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable(with = InternallyTagged.Serializer::class)
@JsonClassDiscriminator("type")
sealed interface InternallyTagged : KeyPathMutable<InternallyTagged> {
    @Serializable @SerialName("unit") data object Unit : InternallyTagged

    @Serializable
    @SerialName("anonymousStruct")
    data class AnonymousStruct(val foor: Int, val bar: String) : InternallyTagged

    @Serializable @SerialName("emptyStruct") data object EmptyStruct : InternallyTagged

    @Serializable @SerialName("tuple") data class Tuple(val value: String) : InternallyTagged

    object Serializer : KSerializer<InternallyTagged> {
        override val descriptor =
                buildClassSerialDescriptor("InternallyTagged") {
                    element<Unit>("unit", isOptional = true)
                    element<AnonymousStruct>("anonymousStruct", isOptional = true)
                    element<EmptyStruct>("emptyStruct", isOptional = true)
                    element<Tuple>("tuple", isOptional = true)
                }

        override fun serialize(encoder: Encoder, value: InternallyTagged) {
            require(encoder is JsonEncoder)
            when (value) {
                is Unit -> {
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("unit"))
                    val element = JsonObject(base)
                    encoder.encodeJsonElement(element)
                }
                is AnonymousStruct -> {
                    val content =
                            encoder.json.encodeToJsonElement(AnonymousStruct.serializer(), value) as
                                    JsonObject
                    val base =
                            mutableMapOf<String, JsonElement>(
                                    "type" to JsonPrimitive("anonymousStruct")
                            )
                    val element = JsonObject(base.apply { putAll(content.toMap()) })
                    encoder.encodeJsonElement(element)
                }
                is EmptyStruct -> {
                    val content =
                            encoder.json.encodeToJsonElement(EmptyStruct.serializer(), value) as
                                    JsonObject
                    val base =
                            mutableMapOf<String, JsonElement>(
                                    "type" to JsonPrimitive("emptyStruct")
                            )
                    val element = JsonObject(base.apply { putAll(content.toMap()) })
                    encoder.encodeJsonElement(element)
                }
                is Tuple -> {
                    val content =
                            encoder.json.encodeToJsonElement(String.serializer(), value.value) as
                                    JsonObject
                    val base = mutableMapOf<String, JsonElement>("type" to JsonPrimitive("tuple"))
                    val element = JsonObject(base.apply { putAll(content.toMap()) })
                    encoder.encodeJsonElement(element)
                }
            }
        }

        override fun deserialize(decoder: Decoder): InternallyTagged {
            require(decoder is JsonDecoder)
            val container = decoder.decodeJsonElement()
            return when (val discriminator = container.jsonObject["type"]?.jsonPrimitive?.content) {
                "unit" -> Unit
                "anonymousStruct" -> {
                    decoder.json.decodeFromJsonElement<AnonymousStruct>(container)
                }
                "emptyStruct" -> {
                    decoder.json.decodeFromJsonElement<EmptyStruct>(container)
                }
                "tuple" -> {
                    val content = decoder.json.decodeFromJsonElement<String>(container)
                    Tuple(content)
                }
                else -> throw Exception("Unknown enum variant $discriminator for InternallyTagged")
            }
        }
    }

    override fun patching(patch: PatchOperation, keyPath: List<KeyPathElement>): InternallyTagged {
        if (keyPath.size < 2) {
            return this.applying(patch)
        }
        val variant = keyPath[0]
        val field = keyPath[1]
        return when {
            this is Unit && variant == KeyPathElement.Variant("unit", VariantTagType.INTERNAL) -> {
                throw IllegalStateException(
                        "InternallyTagged.Unit does not support nested key paths"
                )
            }
            this is AnonymousStruct &&
                    variant ==
                            KeyPathElement.Variant("anonymousStruct", VariantTagType.INTERNAL) -> {
                when (field) {
                    KeyPathElement.Field("foor") -> {
                        val newValue = this.foor.patching(patch, keyPath.drop(2))
                        this.copy(foor = newValue)
                    }
                    KeyPathElement.Field("bar") -> {
                        val newValue = this.bar.patching(patch, keyPath.drop(2))
                        this.copy(bar = newValue)
                    }
                    else ->
                            throw IllegalStateException(
                                    "InternallyTagged.AnonymousStruct does not support $field key path"
                            )
                }
            }
            this is EmptyStruct &&
                    variant == KeyPathElement.Variant("emptyStruct", VariantTagType.INTERNAL) -> {
                throw IllegalStateException(
                        "InternallyTagged.EmptyStruct does not support nested key paths"
                )
            }
            this is Tuple &&
                    variant == KeyPathElement.Variant("tuple", VariantTagType.INTERNAL) -> {
                val newValue = this.value.patching(patch, keyPath.drop(2))
                this.copy(value = newValue)
            }
            else ->
                    throw IllegalStateException(
                            "InternallyTagged has no mutable $variant key path."
                    )
        }
    }

    private fun applying(patch: PatchOperation): InternallyTagged {
        when (patch) {
            is PatchOperation.Update -> return patch.value.asT()
            is PatchOperation.Splice ->
                    throw IllegalStateException(
                            "InternallyTagged does not support splice operations."
                    )
        }
    }
}

@Serializable(with = ExternallyTagged.Serializer::class)
sealed interface ExternallyTagged : KeyPathMutable<ExternallyTagged> {
    @Serializable data class AnonymousStruct(val foor: Int, val bar: String) : ExternallyTagged

    @Serializable data class Tuple(val value: String) : ExternallyTagged

    object Serializer : KSerializer<ExternallyTagged> {
        override val descriptor =
                buildClassSerialDescriptor("ExternallyTagged") {
                    element<AnonymousStruct>("anonymousStruct", isOptional = true)
                    element<Tuple>("tuple", isOptional = true)
                }

        override fun serialize(encoder: Encoder, value: ExternallyTagged) {
            encoder.encodeStructure(descriptor) {
                when (value) {
                    is AnonymousStruct ->
                            encodeSerializableElement(
                                    descriptor,
                                    0,
                                    AnonymousStruct.serializer(),
                                    value
                            )
                    is Tuple ->
                            encodeSerializableElement(
                                    descriptor,
                                    1,
                                    String.serializer(),
                                    value.value
                            )
                }
            }
        }

        override fun deserialize(decoder: Decoder): ExternallyTagged {
            return decoder.decodeStructure(descriptor) {
                when (val index = decodeElementIndex(descriptor)) {
                    0 -> {
                        return@decodeStructure decodeSerializableElement(
                                descriptor,
                                0,
                                AnonymousStruct.serializer()
                        )
                    }
                    1 -> {
                        val value = decodeSerializableElement(descriptor, 1, String.serializer())
                        return@decodeStructure Tuple(value)
                    }
                    else -> throw Exception("Unknown enum variant $index for ExternallyTagged")
                }
            }
        }
    }

    override fun patching(patch: PatchOperation, keyPath: List<KeyPathElement>): ExternallyTagged {
        if (keyPath.size < 2) {
            return this.applying(patch)
        }
        val variant = keyPath[0]
        val field = keyPath[1]
        return when {
            this is AnonymousStruct &&
                    variant ==
                            KeyPathElement.Variant("anonymousStruct", VariantTagType.EXTERNAL) -> {
                when (field) {
                    KeyPathElement.Field("foor") -> {
                        val newValue = this.foor.patching(patch, keyPath.drop(1))
                        this.copy(foor = newValue)
                    }
                    KeyPathElement.Field("bar") -> {
                        val newValue = this.bar.patching(patch, keyPath.drop(1))
                        this.copy(bar = newValue)
                    }
                    else ->
                            throw IllegalStateException(
                                    "ExternallyTagged.AnonymousStruct does not support $field key path"
                            )
                }
            }
            this is Tuple &&
                    variant == KeyPathElement.Variant("tuple", VariantTagType.EXTERNAL) -> {
                when (field) {
                    KeyPathElement.Field("0") -> {
                        val newValue = this.value.patching(patch, keyPath.drop(2))
                        this.copy(value = newValue)
                    }
                    else ->
                            throw IllegalStateException(
                                    "ExternallyTagged.Tuple does not support $field key path"
                            )
                }
            }
            else ->
                    throw IllegalStateException(
                            "ExternallyTagged has no mutable $variant key path."
                    )
        }
    }

    private fun applying(patch: PatchOperation): ExternallyTagged {
        when (patch) {
            is PatchOperation.Update -> return patch.value.asT()
            is PatchOperation.Splice ->
                    throw IllegalStateException(
                            "ExternallyTagged does not support splice operations."
                    )
        }
    }
}

@Serializable
enum class Unit : KeyPathMutable<Unit> {
    @SerialName("foo") FOO,
    @SerialName("bar") BAR;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value

    override fun patching(patch: PatchOperation, keyPath: List<KeyPathElement>): Unit {
        if (keyPath.isNotEmpty()) {
            throw IllegalStateException("Unit does not support child keyPath")
        }
        return this.applying(patch)
    }

    private fun applying(patch: PatchOperation): Unit {
        when (patch) {
            is PatchOperation.Update -> return patch.value.asT()
            is PatchOperation.Splice ->
                    throw IllegalStateException("Unit does not support splice operations.")
        }
    }
}

@Serializable(with = Generic<T>.Serializer::class)
sealed interface Generic<T> : KeyPathMutable<Generic<T>> {
    @Serializable data class AnonymousStruct<T>(val foo: T) : Generic<T>

    @Serializable data class Tuple<T>(val value: T) : Generic<T>

    object Serializer : KSerializer<Generic<T>> {
        override val descriptor =
                buildClassSerialDescriptor("Generic<T>") {
                    element<AnonymousStruct<T>>("anonymousStruct", isOptional = true)
                    element<Tuple<T>>("tuple", isOptional = true)
                }

        override fun serialize(encoder: Encoder, value: Generic<T>) {
            encoder.encodeStructure(descriptor) {
                when (value) {
                    is AnonymousStruct ->
                            encodeSerializableElement(
                                    descriptor,
                                    0,
                                    AnonymousStruct<T>.serializer(),
                                    value
                            )
                    is Tuple ->
                            encodeSerializableElement(descriptor, 1, T.serializer(), value.value)
                }
            }
        }

        override fun deserialize(decoder: Decoder): Generic<T> {
            return decoder.decodeStructure(descriptor) {
                when (val index = decodeElementIndex(descriptor)) {
                    0 -> {
                        return@decodeStructure decodeSerializableElement(
                                descriptor,
                                0,
                                AnonymousStruct<T>.serializer()
                        )
                    }
                    1 -> {
                        val value = decodeSerializableElement(descriptor, 1, T.serializer())
                        return@decodeStructure Tuple<T>(value)
                    }
                    else -> throw Exception("Unknown enum variant $index for Generic<T>")
                }
            }
        }
    }

    override fun patching(patch: PatchOperation, keyPath: List<KeyPathElement>): Generic<T> {
        if (keyPath.size < 2) {
            return this.applying(patch)
        }
        val variant = keyPath[0]
        val field = keyPath[1]
        return when {
            this is AnonymousStruct &&
                    variant ==
                            KeyPathElement.Variant("anonymousStruct", VariantTagType.EXTERNAL) -> {
                when (field) {
                    KeyPathElement.Field("foo") -> {
                        val newValue = this.foo.patching(patch, keyPath.drop(1))
                        this.copy(foo = newValue)
                    }
                    else ->
                            throw IllegalStateException(
                                    "Generic<T>.AnonymousStruct does not support $field key path"
                            )
                }
            }
            this is Tuple &&
                    variant == KeyPathElement.Variant("tuple", VariantTagType.EXTERNAL) -> {
                when (field) {
                    KeyPathElement.Field("0") -> {
                        val newValue = this.value.patching(patch, keyPath.drop(2))
                        this.copy(value = newValue)
                    }
                    else ->
                            throw IllegalStateException(
                                    "Generic<T>.Tuple does not support $field key path"
                            )
                }
            }
            else -> throw IllegalStateException("Generic<T> has no mutable $variant key path.")
        }
    }

    private fun applying(patch: PatchOperation): Generic<T> {
        when (patch) {
            is PatchOperation.Update -> return patch.value.asT()
            is PatchOperation.Splice ->
                    throw IllegalStateException("Generic<T> does not support splice operations.")
        }
    }
}
