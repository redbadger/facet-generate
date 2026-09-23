import Serde

indirect public enum Signal: Hashable, Equatable {
    case level(UInt8)
    case silent

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .level(let x):
            try serializer.serialize_variant_index(value: 0)
            try serializer.serialize_u8(value: x)
        case .silent:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Signal {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try deserializer.deserialize_u8()
            try deserializer.decrease_container_depth()
            return .level(x)
        case 1:
            try deserializer.decrease_container_depth()
            return .silent
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Signal: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Signal {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Status: Hashable, Equatable {
    case up
    case down

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .up:
            try serializer.serialize_variant_index(value: 0)
        case .down:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Status {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .up
        case 1:
            try deserializer.decrease_container_depth()
            return .down
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Status: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Status {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
