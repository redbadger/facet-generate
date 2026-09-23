import Serde

func serializeSet<T: Hashable, S: Serializer>(
    value: Swift.Set<T>,
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
) throws -> Swift.Set<T> {
    let length = try deserializer.deserialize_len()
    var obj: Swift.Set<T> = []
    for _ in 0..<length {
        obj.insert(try deserializeElement(deserializer))
    }
    return obj
}

public struct Set: Hashable, Equatable {
    public var value: UInt32

    public init(value: UInt32) {
        self.value = value
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_u32(value: self.value)
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Set {
        try deserializer.increase_container_depth()
        let value = try deserializer.deserialize_u32()
        try deserializer.decrease_container_depth()
        return Set(value: value)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Set {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct Tray {
    public var nothing: Void
    public var ids: Swift.Set<UInt32>

    public init(nothing: Void, ids: Swift.Set<UInt32>) {
        self.nothing = nothing
        self.ids = ids
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_unit(value: self.nothing)
        try serializeSet(value: self.ids, serializer: serializer) { item, serializer in
            try serializer.serialize_u32(value: item)
        }
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Tray {
        try deserializer.increase_container_depth()
        let nothing = try deserializer.deserialize_unit()
        let ids = try deserializeSet(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u32()
        }
        try deserializer.decrease_container_depth()
        return Tray(nothing: nothing, ids: ids)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Tray {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
