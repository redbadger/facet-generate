package com.example.a

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

data class Row(
    val status: com.example.b.Status,
    val signal: com.example.b.Signal,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        status.serialize(serializer)
        signal.serialize(serializer)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Row {
            deserializer.increase_container_depth()
            val status = com.example.b.Status.deserialize(deserializer)
            val signal = com.example.b.Signal.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return Row(status, signal)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Row {
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
