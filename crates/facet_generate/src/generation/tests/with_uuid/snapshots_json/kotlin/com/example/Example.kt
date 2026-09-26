package com.example

import com.novi.serde.JsonElementSerializer
import com.novi.serde.JsonNewTypeSerializer
import com.novi.serde.JsonPairSerializer
import com.novi.serde.JsonTripleSerializer
import com.novi.serde.JsonUnitSerializer
import kotlinx.serialization.EncodeDefault
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.nullable
import kotlinx.serialization.builtins.serializer
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

@OptIn(ExperimentalSerializationApi::class)
@Serializable
@SerialName("StructWithUuid")
data class StructWithUuid(
    @SerialName("id") val id: UUID,
    @SerialName("parent_id") @EncodeDefault val parentId: UUID? = null,
    @SerialName("name") val name: String,
)
