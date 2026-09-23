package com.example.b

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

sealed interface Signal {
    fun serialize(serializer: Serializer)

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    data class Level(
        val value: UByte,
    ) : Signal {
        override fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(0)
            serializer.serialize_u8(value)
            serializer.decrease_container_depth()
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Level {
                deserializer.increase_container_depth()
                val value = deserializer.deserialize_u8()
                deserializer.decrease_container_depth()
                return Level(value)
            }
        }
    }

    data object Silent: Signal {
        override fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(1)
            serializer.decrease_container_depth()
        }

        fun deserialize(deserializer: Deserializer): Silent {
            return Silent
        }
    }

    companion object {
        @Throws(DeserializationError::class)
        fun deserialize(deserializer: Deserializer): Signal {
            val index = deserializer.deserialize_variant_index()
            return when (index) {
                0 -> Level.deserialize(deserializer)
                1 -> Silent.deserialize(deserializer)
                else -> throw DeserializationError("Unknown variant index for Signal: $index")
            }
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Signal {
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

enum class Status {
    UP,
    DOWN;

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
        fun deserialize(deserializer: Deserializer): Status {
            deserializer.increase_container_depth()
            val index = deserializer.deserialize_variant_index()
            deserializer.decrease_container_depth()
            return when (index) {
                0 -> UP
                1 -> DOWN
                else -> throw DeserializationError("Unknown variant index for Status: $index")
            }
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Status {
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
