package com.example

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

data class Child(
    val name: String,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        serializer.serialize_str(name)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Child {
            deserializer.increase_container_depth()
            val name = deserializer.deserialize_str()
            deserializer.decrease_container_depth()
            return Child(name)
        }

        fun bincodeDeserialize(input: ByteArray?): Child {
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

sealed interface Parent {
    // Abstract methods that implementations must provide
    fun serialize(serializer: Serializer)

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
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
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Parent {
            val index = deserializer.deserialize_variant_index()
            return when (index) {
                0 -> {
                    deserializer.increase_container_depth()
                    val value = com.example.Child.deserialize(deserializer)
                    deserializer.decrease_container_depth()
                    Child(value)
                }
                else -> throw DeserializationError("Unknown variant index for Parent: $index")
            }
        }

        fun bincodeDeserialize(input: ByteArray?): Parent {
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
