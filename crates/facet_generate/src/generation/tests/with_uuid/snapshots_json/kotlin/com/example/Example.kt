package com.example

import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

private object UUIDSerializer : KSerializer<java.util.UUID> {
    override val descriptor = PrimitiveSerialDescriptor("UUID", PrimitiveKind.STRING)
    override fun deserialize(decoder: Decoder): java.util.UUID = java.util.UUID.fromString(decoder.decodeString())
    override fun serialize(encoder: Encoder, value: java.util.UUID) = encoder.encodeString(value.toString())
}

typealias UUID = @Serializable(with = UUIDSerializer::class) java.util.UUID

@Serializable
@SerialName("StructWithUuid")
data class StructWithUuid(
    val id: UUID,
    val parentId: UUID? = null,
    val name: String,
)
