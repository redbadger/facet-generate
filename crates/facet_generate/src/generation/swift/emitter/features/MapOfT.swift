func serializeMap<K, V, S: Serializer>(
    value: [K: V],
    serializer: S,
    serializeEntry: (K, V, S) throws -> Void
) throws {
    try serializer.serialize_len(value: value.count)
    for (key, value) in value {
        try serializeEntry(key, value, serializer)
    }
}

func deserializeMap<K: Hashable, V, D: Deserializer>(
    deserializer: D,
    deserializeEntry: (D) throws -> (K, V)
) throws -> [K: V] {
    let length = try deserializer.deserialize_len()
    var obj: [K: V] = [:]
    for _ in 0..<length {
        let (key, value) = try deserializeEntry(deserializer)
        obj[key] = value
    }
    return obj
}
