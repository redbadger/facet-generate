import Kv
import Serde

public struct App: Hashable, Equatable {
    public var entry: Kv.Entry

    public init(entry: Kv.Entry) {
        self.entry = entry
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.entry.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> App {
        try deserializer.increase_container_depth()
        let entry = try Kv.Entry.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return App(entry: entry)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> App {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Level: Hashable, Equatable {
    case low
    case high

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .low:
            try serializer.serialize_variant_index(value: 0)
        case .high:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Level {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .low
        case 1:
            try deserializer.decrease_container_depth()
            return .high
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Level: \(index)")
        }
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Level {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Outcome: Hashable, Equatable {
    case score(UInt32)
    case missing

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .score(let x):
            try serializer.serialize_variant_index(value: 0)
            try serializer.serialize_u32(value: x)
        case .missing:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Outcome {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try deserializer.deserialize_u32()
            try deserializer.decrease_container_depth()
            return .score(x)
        case 1:
            try deserializer.decrease_container_depth()
            return .missing
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Outcome: \(index)")
        }
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Outcome {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
