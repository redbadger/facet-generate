func serializeTupleArray<T, S: Serializer>(
    value: [T],
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    for item in value {
        try serializeElement(item, serializer)
    }
}

func deserializeTupleArray<T, D: Deserializer>(
    deserializer: D,
    size: Int,
    deserializeElement: (D) throws -> T
) throws -> [T] {
    var obj: [T] = []
    for _ in 0..<size {
        obj.append(try deserializeElement(deserializer))
    }
    return obj
}
