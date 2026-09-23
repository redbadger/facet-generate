package com.example.kit

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

fun <T> kotlin.collections.Set<T>.serialize(
    serializer: Serializer,
    serializeElement: Serializer.(T) -> Unit,
) {
    serializer.serialize_len(size.toLong())
    forEach { element ->
        serializer.serializeElement(element)
    }
}

fun <T> Deserializer.deserializeSetOf(deserializeElement: (Deserializer) -> T): kotlin.collections.Set<T> {
    val length = deserialize_len()
    val set = mutableSetOf<T>()
    repeat(length.toInt()) {
        set.add(deserializeElement(this))
    }
    return set
}

data class Set(
    val value: UInt,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        serializer.serialize_u32(value)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Set {
            deserializer.increase_container_depth()
            val value = deserializer.deserialize_u32()
            deserializer.decrease_container_depth()
            return Set(value)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Set {
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

data class Tray(
    val nothing: Unit,
    val ids: kotlin.collections.Set<UInt>,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        serializer.serialize_unit(nothing)
        ids.serialize(serializer) {
            serializer.serialize_u32(it)
        }
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Tray {
            deserializer.increase_container_depth()
            val nothing = deserializer.deserialize_unit()
            val ids =
                deserializer.deserializeSetOf {
                    deserializer.deserialize_u32()
                }
            deserializer.decrease_container_depth()
            return Tray(nothing, ids)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Tray {
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
