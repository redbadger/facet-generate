package com.example

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

fun <T> Set<T>.serialize(
    serializer: Serializer,
    serializeElement: Serializer.(T) -> kotlin.Unit,
) {
    serializer.serialize_len(size.toLong())
    forEach { element ->
        serializer.serializeElement(element)
    }
}

fun <T> Deserializer.deserializeSetOf(deserializeElement: (Deserializer) -> T): Set<T> {
    val length = deserialize_len()
    val set = mutableSetOf<T>()
    repeat(length.toInt()) {
        set.add(deserializeElement(this))
    }
    return set
}

data class Shelf(
    val set: com.example.kit.Set,
    val ids: Set<UInt>,
    val unit: com.example.Unit,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        set.serialize(serializer)
        ids.serialize(serializer) {
            serializer.serialize_u32(it)
        }
        unit.serialize(serializer)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Shelf {
            deserializer.increase_container_depth()
            val set = com.example.kit.Set.deserialize(deserializer)
            val ids =
                deserializer.deserializeSetOf {
                    deserializer.deserialize_u32()
                }
            val unit = com.example.Unit.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return Shelf(set, ids, unit)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Shelf {
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

data class Unit(
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
        fun deserialize(deserializer: Deserializer): Unit {
            deserializer.increase_container_depth()
            val value = deserializer.deserialize_u32()
            deserializer.decrease_container_depth()
            return Unit(value)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Unit {
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
