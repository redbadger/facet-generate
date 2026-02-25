func serializeSet<T: Hashable, S: Serializer>(
    value: Set<T>,
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    try serializer.serialize_len(value: value.count)
    for item in value {
        try serializeElement(item, serializer)
    }
}

func deserializeSet<T: Hashable, D: Deserializer>(
    deserializer: D,
    deserializeElement: (D) throws -> T
) throws -> Set<T> {
    let length = try deserializer.deserialize_len()
    var obj: Set<T> = []
    for _ in 0..<length {
        obj.insert(try deserializeElement(deserializer))
    }
    return obj
}
