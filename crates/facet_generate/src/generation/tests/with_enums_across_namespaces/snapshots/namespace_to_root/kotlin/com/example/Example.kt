package com.example

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

data class App(
    val entry: com.example.kv.Entry,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        entry.serialize(serializer)
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
            val entry = com.example.kv.Entry.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return App(entry)
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

enum class Level {
    LOW,
    HIGH;

    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        serializer.serialize_variant_index(ordinal)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        @Throws(DeserializationError::class)
        fun deserialize(deserializer: Deserializer): Level {
            deserializer.increase_container_depth()
            val index = deserializer.deserialize_variant_index()
            deserializer.decrease_container_depth()
            return when (index) {
                0 -> LOW
                1 -> HIGH
                else -> throw DeserializationError("Unknown variant index for Level: $index")
            }
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Level {
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

sealed interface Outcome {
    fun serialize(serializer: Serializer)

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    data class Score(
        val value: UInt,
    ) : Outcome {
        override fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(0)
            serializer.serialize_u32(value)
            serializer.decrease_container_depth()
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Score {
                deserializer.increase_container_depth()
                val value = deserializer.deserialize_u32()
                deserializer.decrease_container_depth()
                return Score(value)
            }
        }
    }

    data object Missing: Outcome {
        override fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(1)
            serializer.decrease_container_depth()
        }

        fun deserialize(deserializer: Deserializer): Missing {
            return Missing
        }
    }

    companion object {
        @Throws(DeserializationError::class)
        fun deserialize(deserializer: Deserializer): Outcome {
            val index = deserializer.deserialize_variant_index()
            return when (index) {
                0 -> Score.deserialize(deserializer)
                1 -> Missing.deserialize(deserializer)
                else -> throw DeserializationError("Unknown variant index for Outcome: $index")
            }
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Outcome {
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
