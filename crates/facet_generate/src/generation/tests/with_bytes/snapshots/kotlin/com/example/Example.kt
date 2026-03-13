package com.example

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.Bytes
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

data class StructWithBytes(
    val data: Bytes,
    val name: String,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        serializer.serialize_bytes(data)
        serializer.serialize_str(name)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): StructWithBytes {
            deserializer.increase_container_depth()
            val data = deserializer.deserialize_bytes()
            val name = deserializer.deserialize_str()
            deserializer.decrease_container_depth()
            return StructWithBytes(data, name)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): StructWithBytes {
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
