package com.example

import serde.BincodeDeserializer
import serde.BincodeSerializer
import serde.DeserializationError
import serde.Deserializer
import serde.Serializer

data class Child(
    val external: com.example2.other.other.OtherParent,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        external.serialize(serializer)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer._bytes
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Child {
            deserializer.increase_container_depth()
            val external = com.example2.other.other.OtherParent.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return Child(external)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Child {
            if (input == null) {
                throw DeserializationError("Cannot deserialize null array")
            }
            val deserializer = BincodeDeserializer(input)
            val value = deserialize(deserializer)
            if (deserializer._buffer_offset < input.size) {
                throw DeserializationError("Some input bytes were not read")
            }
            return value
        }
    }
}

sealed interface Parent {
    fun serialize(serializer: Serializer)

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer._bytes
    }

    data class Child(
        val value: com.example.Child,
    ) : Parent {
        override fun serialize(serializer: Serializer) {
            serializer.increase_container_depth()
            serializer.serialize_variant_index(0)
            value.serialize(serializer)
            serializer.decrease_container_depth()
        }

        companion object {
            fun deserialize(deserializer: Deserializer): Child {
                deserializer.increase_container_depth()
                val value = com.example.Child.deserialize(deserializer)
                deserializer.decrease_container_depth()
                return Child(value)
            }
        }
    }

    companion object {
        @Throws(DeserializationError::class)
        fun deserialize(deserializer: Deserializer): Parent {
            val index = deserializer.deserialize_variant_index()
            return when (index) {
                0 -> Child.deserialize(deserializer)
                else -> throw DeserializationError("Unknown variant index for Parent: $index")
            }
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Parent {
            if (input == null) {
                throw DeserializationError("Cannot deserialize null array")
            }
            val deserializer = BincodeDeserializer(input)
            val value = deserialize(deserializer)
            if (deserializer._buffer_offset < input.size) {
                throw DeserializationError("Some input bytes were not read")
            }
            return value
        }
    }
}
