package com.example

import serde.BincodeDeserializer
import serde.BincodeSerializer
import serde.DeserializationError
import serde.Deserializer
import serde.Serializer

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

fun <K, V> Map<K, V>.serialize(
    serializer: Serializer,
    serializeEntry: Serializer.(K, V) -> Unit,
) {
    serializer.serialize_len(size.toLong())
    forEach { (key, value) ->
        serializer.serializeEntry(key, value)
    }
}

fun <K, V> Deserializer.deserializeMapOf(deserializeEntry: (Deserializer) -> Pair<K, V>): Map<K, V> {
    val length = deserialize_len()
    val map = mutableMapOf<K, V>()
    repeat(length.toInt()) {
        val (key, value) = deserializeEntry(this)
        map[key] = value
    }
    return map
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

fun <T> Set<T>.serialize(
    serializer: Serializer,
    serializeElement: Serializer.(T) -> Unit,
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
        return serializer._bytes
    }

    companion object {
        fun deserialize(deserializer: Deserializer): Child {
            deserializer.increase_container_depth()
            val name = deserializer.deserialize_str()
            deserializer.decrease_container_depth()
            return Child(name)
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

data class MyStruct(
    val stringToInt: Map<String, Int>,
    val mapToList: Map<String, List<Int>>,
    val optionOfVecOfSet: List<Set<String>>? = null,
    val parent: com.example.Parent,
) {
    fun serialize(serializer: Serializer) {
        serializer.increase_container_depth()
        stringToInt.serialize(serializer) { key, value ->
            serializer.serialize_str(key)
            serializer.serialize_i32(value)
        }
        mapToList.serialize(serializer) { key, value ->
            serializer.serialize_str(key)
            value.serialize(serializer) {
                serializer.serialize_i32(it)
            }
        }
        optionOfVecOfSet.serializeOptionOf(serializer) { level1 ->
            level1.serialize(serializer) { level2 ->
                level2.serialize(serializer) {
                    serializer.serialize_str(it)
                }
            }
        }
        parent.serialize(serializer)
        serializer.decrease_container_depth()
    }

    fun bincodeSerialize(): ByteArray {
        val serializer = BincodeSerializer()
        serialize(serializer)
        return serializer._bytes
    }

    companion object {
        fun deserialize(deserializer: Deserializer): MyStruct {
            deserializer.increase_container_depth()
            val stringToInt =
                deserializer.deserializeMapOf {
                    val key = deserializer.deserialize_str()
                    val value = deserializer.deserialize_i32()
                    Pair(key, value)
                }
            val mapToList =
                deserializer.deserializeMapOf {
                    val key = deserializer.deserialize_str()
                    val value =
                        deserializer.deserializeListOf {
                            deserializer.deserialize_i32()
                        }
                    Pair(key, value)
                }
            val optionOfVecOfSet =
                deserializer.deserializeOptionOf {
                    deserializer.deserializeListOf {
                        deserializer.deserializeSetOf {
                            deserializer.deserialize_str()
                        }
                    }
                }
            val parent = com.example.Parent.deserialize(deserializer)
            deserializer.decrease_container_depth()
            return MyStruct(stringToInt, mapToList, optionOfVecOfSet, parent)
        }

        @Throws(DeserializationError::class)
        fun bincodeDeserialize(input: ByteArray?): MyStruct {
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
