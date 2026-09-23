package com.example

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

data class App(
    val row: com.example.a.Row,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        row.serialize(serializer)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): App {
            deserializer.increase_container_depth()
            val row = com.example.a.Row.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return App(row)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): App {
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
