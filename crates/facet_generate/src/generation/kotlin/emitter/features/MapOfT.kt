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
