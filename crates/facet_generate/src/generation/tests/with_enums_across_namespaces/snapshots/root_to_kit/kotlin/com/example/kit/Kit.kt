package com.example.kit

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

data class Badge(
    val presence: com.example.kit.Presence,
    val shape: com.example.kit.Shape,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        presence.serialize(serializer)
        shape.serialize(serializer)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Badge {
            deserializer.increase_container_depth()
            val presence = com.example.kit.Presence.deserialize(deserializer)
            val shape = com.example.kit.Shape.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return Badge(presence, shape)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Badge {
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

enum class Presence {
    ONLINE,
    OFFLINE;

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
        fun deserialize(deserializer: Deserializer): Presence {
            deserializer.increase_container_depth()
            val index = deserializer.deserialize_variant_index()
            deserializer.decrease_container_depth()
            return when (index) {
                0 -> ONLINE
                1 -> OFFLINE
                else -> throw DeserializationError("Unknown variant index for Presence: $index")
            }
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Presence {
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

sealed interface Shape {
    fun serialize(serializer: Serializer)

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    data class Circle(
        val value: Double,
    ) : Shape {
        override fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(0)
            serializer.serialize_f64(value)
            serializer.decrease_container_depth()
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Circle {
                deserializer.increase_container_depth()
                val value = deserializer.deserialize_f64()
                deserializer.decrease_container_depth()
                return Circle(value)
            }
        }
    }

    data object Empty: Shape {
        override fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(1)
            serializer.decrease_container_depth()
        }

        fun deserialize(deserializer: Deserializer): Empty {
            return Empty
        }
    }

    companion object {
        @Throws(DeserializationError::class)
        fun deserialize(deserializer: Deserializer): Shape {
            val index = deserializer.deserialize_variant_index()
            return when (index) {
                0 -> Circle.deserialize(deserializer)
                1 -> Empty.deserialize(deserializer)
                else -> throw DeserializationError("Unknown variant index for Shape: $index")
            }
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Shape {
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
