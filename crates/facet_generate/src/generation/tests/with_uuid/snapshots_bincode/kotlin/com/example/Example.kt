package com.example

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer
import java.util.UUID

fun <T> T?.serializeOptionOf(
    serializer: Serializer,
    serializeElement: Serializer.(T) -> Unit,
) {
    if (this != null) {
        serializer.serialize_option_tag(true)
        serializer.serializeElement(this)
    } else {
        serializer.serialize_option_tag(false)
    }
}

fun <T> Deserializer.deserializeOptionOf(deserializeElement: (Deserializer) -> T): T? {
    val tag = deserialize_option_tag()
    return if (tag) {
        deserializeElement(this)
    } else {
        null
    }
}

fun UUID.serialize(serializer: Serializer) {
    val bytes = ByteArray(16)
    val msb = mostSignificantBits
    val lsb = leastSignificantBits
    for (i in 0..7) {
        bytes[i]     = (msb ushr (56 - i * 8) and 0xff).toByte()
        bytes[8 + i] = (lsb ushr (56 - i * 8) and 0xff).toByte()
    }
    serializer.serialize_bytes(Bytes(bytes))
}

fun Deserializer.deserializeUuid(): UUID {
    val bytes = deserialize_bytes().content
    if (bytes.size != 16) {
        throw DeserializationError("UUID must be 16 bytes, got ${bytes.size}")
    }
    var msb = 0L
    var lsb = 0L
    for (i in 0..7) {
        msb = (msb shl 8) or (bytes[i].toLong() and 0xff)
        lsb = (lsb shl 8) or (bytes[8 + i].toLong() and 0xff)
    }
    return UUID(msb, lsb)
}

data class StructWithUuid(
    val id: UUID,
    val parentId: UUID? = null,
    val name: String,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        id.serialize(serializer)
        parentId.serializeOptionOf(serializer) {
            it.serialize(serializer)
        }
        serializer.serialize_str(name)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): StructWithUuid {
            deserializer.increase_container_depth()
            val id = deserializer.deserializeUuid()
            val parentId =
                deserializer.deserializeOptionOf {
                    deserializer.deserializeUuid()
                }
            val name = deserializer.deserialize_str()
            deserializer.decrease_container_depth()
            return StructWithUuid(id, parentId, name)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): StructWithUuid {
            if (input == null) {
                throw DeserializationError("Cannot deserialize null array")
            }
            val deserializer = BincodeDeserializer(input)
            val value = deserialize(deserializer)
            if (deserializer.get_buffer_offset() < input.size) {
                throw DeserializationError("Some input bytes were not read")
            }
            return value
        }
    }
}
