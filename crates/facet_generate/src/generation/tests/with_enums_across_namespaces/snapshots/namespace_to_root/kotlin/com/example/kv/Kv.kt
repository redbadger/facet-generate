package com.example.kv

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

data class Entry(
    val level: com.example.kv.Level,
    val outcome: com.example.kv.Outcome,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        level.serialize(serializer)
        outcome.serialize(serializer)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Entry {
            deserializer.increase_container_depth()
            val level = com.example.kv.Level.deserialize(deserializer)
            val outcome = com.example.kv.Outcome.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return Entry(level, outcome)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Entry {
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
