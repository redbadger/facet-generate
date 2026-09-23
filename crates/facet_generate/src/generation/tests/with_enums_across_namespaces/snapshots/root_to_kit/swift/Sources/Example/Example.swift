import Kit
import Serde

func serializeArray<T, S: Serializer>(
    value: [T],
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    try serializer.serialize_len(value: value.count)
    for item in value {
        try serializeElement(item, serializer)
    }
}

func deserializeArray<T, D: Deserializer>(
    deserializer: D,
    deserializeElement: (D) throws -> T
) throws -> [T] {
    let length = try deserializer.deserialize_len()
    var obj: [T] = []
    for _ in 0..<length {
        obj.append(try deserializeElement(deserializer))
    }
    return obj
}

func serializeOption<T, S: Serializer>(
    value: T?,
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    if let value = value {
        try serializer.serialize_option_tag(value: true)
        try serializeElement(value, serializer)
    } else {
        try serializer.serialize_option_tag(value: false)
    }
}

func deserializeOption<T, D: Deserializer>(
    deserializer: D,
    deserializeElement: (D) throws -> T
) throws -> T? {
    let tag = try deserializer.deserialize_option_tag()
    if tag {
        return try deserializeElement(deserializer)
    } else {
        return nil
    }
}

public struct Card: Hashable, Equatable {
    public var presence: Kit.Presence
    public var shape: Kit.Shape
    public var shapes: [Kit.Shape?]
    public var badge: Kit.Badge

    public init(presence: Kit.Presence, shape: Kit.Shape, shapes: [Kit.Shape?], badge: Kit.Badge) {
        self.presence = presence
        self.shape = shape
        self.shapes = shapes
        self.badge = badge
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.presence.serialize(serializer: serializer)
        try self.shape.serialize(serializer: serializer)
        try serializeArray(value: self.shapes, serializer: serializer) { item, serializer in
            try serializeOption(value: item, serializer: serializer) { value, serializer in
                try value.serialize(serializer: serializer)
            }
        }
        try self.badge.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Card {
        try deserializer.increase_container_depth()
        let presence = try Kit.Presence.deserialize(deserializer: deserializer)
        let shape = try Kit.Shape.deserialize(deserializer: deserializer)
        let shapes = try deserializeArray(deserializer: deserializer) { deserializer in
            try deserializeOption(deserializer: deserializer) { deserializer in
                try Kit.Shape.deserialize(deserializer: deserializer)
            }
        }
        let badge = try Kit.Badge.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return Card(presence: presence, shape: shape, shapes: shapes, badge: badge)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Card {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct Presence: Hashable, Equatable {
    public var since: UInt64

    public init(since: UInt64) {
        self.since = since
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_u64(value: self.since)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Presence {
        try deserializer.increase_container_depth()
        let since = try deserializer.deserialize_u64()
        try deserializer.decrease_container_depth()
        return Presence(since: since)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Presence {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct Sighting: Hashable, Equatable {
    public var lastSeen: Presence

    public init(lastSeen: Presence) {
        self.lastSeen = lastSeen
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.lastSeen.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Sighting {
        try deserializer.increase_container_depth()
        let lastSeen = try Presence.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return Sighting(lastSeen: lastSeen)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Sighting {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
