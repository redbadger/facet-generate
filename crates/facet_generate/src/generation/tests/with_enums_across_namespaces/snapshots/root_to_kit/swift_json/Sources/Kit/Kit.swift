import Serde

public struct Badge: Hashable, Equatable {
    public var presence: Presence
    public var shape: Shape

    public init(presence: Presence, shape: Shape) {
        self.presence = presence
        self.shape = shape
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.presence.serialize(serializer: serializer)
        try self.shape.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Badge {
        try deserializer.increase_container_depth()
        let presence = try Kit.Presence.deserialize(deserializer: deserializer)
        let shape = try Kit.Shape.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return Badge(presence: presence, shape: shape)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Badge {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Presence: Hashable, Equatable {
    case online
    case offline

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .online:
            try serializer.serialize_variant_index(value: 0)
        case .offline:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Presence {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .online
        case 1:
            try deserializer.decrease_container_depth()
            return .offline
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Presence: \(index)")
        }
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Presence {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Shape: Hashable, Equatable {
    case circle(Double)
    case empty

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .circle(let x):
            try serializer.serialize_variant_index(value: 0)
            try serializer.serialize_f64(value: x)
        case .empty:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Shape {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try deserializer.deserialize_f64()
            try deserializer.decrease_container_depth()
            return .circle(x)
        case 1:
            try deserializer.decrease_container_depth()
            return .empty
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Shape: \(index)")
        }
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Shape {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
