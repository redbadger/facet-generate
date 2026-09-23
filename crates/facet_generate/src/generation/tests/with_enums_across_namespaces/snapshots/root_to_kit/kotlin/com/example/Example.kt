package com.example

import com.novi.bincode.BincodeDeserializer
import com.novi.bincode.BincodeSerializer
import com.novi.serde.DeserializationError
import com.novi.serde.Deserializer
import com.novi.serde.Serializer

fun <T> List<T>.serialize(
    serializer: Serializer,
    serializeElement: Serializer.(T) -> Unit,
) {
    serializer.serialize_len(size.toLong())
    forEach { element ->
        serializer.serializeElement(element)
    }
}

fun <T> Deserializer.deserializeListOf(deserializeElement: (Deserializer) -> T): List<T> {
    val length = deserialize_len()
    val list = mutableListOf<T>()
    repeat(length.toInt()) {
        list.add(deserializeElement(this))
    }
    return list
}

fun <T> T?.serializeOptionOf(
    serializer: Serializer,
    serializeElement: Serializer.(T) -> Unit,
) {
    if (this != null) {
        serializer.serialize_option_tag(true)
        serializer.serializeElement(this)
    } else {
        serializer.serialize_option_tag(false)
    }
}

fun <T> Deserializer.deserializeOptionOf(deserializeElement: (Deserializer) -> T): T? {
    val tag = deserialize_option_tag()
    return if (tag) {
        deserializeElement(this)
    } else {
        null
    }
}

data class Card(
    val presence: com.example.kit.Presence,
    val shape: com.example.kit.Shape,
    val shapes: List<com.example.kit.Shape?>,
    val badge: com.example.kit.Badge,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        presence.serialize(serializer)
        shape.serialize(serializer)
        shapes.serialize(serializer) { level1 ->
            level1.serializeOptionOf(serializer) {
                it.serialize(serializer)
            }
        }
        badge.serialize(serializer)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Card {
            deserializer.increase_container_depth()
            val presence = com.example.kit.Presence.deserialize(deserializer)
            val shape = com.example.kit.Shape.deserialize(deserializer)
            val shapes =
                deserializer.deserializeListOf {
                    deserializer.deserializeOptionOf {
                        com.example.kit.Shape.deserialize(deserializer)
                    }
                }
            val badge = com.example.kit.Badge.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return Card(presence, shape, shapes, badge)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Card {
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

data class Presence(
    val since: ULong,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        serializer.serialize_u64(since)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Presence {
            deserializer.increase_container_depth()
            val since = deserializer.deserialize_u64()
            deserializer.decrease_container_depth()
            return Presence(since)
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

data class Sighting(
    val lastSeen: com.example.Presence,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        lastSeen.serialize(serializer)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer.get_bytes()
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Sighting {
            deserializer.increase_container_depth()
            val lastSeen = com.example.Presence.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return Sighting(lastSeen)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): Sighting {
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
