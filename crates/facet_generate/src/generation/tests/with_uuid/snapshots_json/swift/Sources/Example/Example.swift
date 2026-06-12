import Foundation
import Serde

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

func serializeUuid<S: Serializer>(
    value: UUID,
    serializer: S
) throws {
    try serializer.serialize_str(value: value.uuidString.lowercased())
}

func deserializeUuid<D: Deserializer>(
    deserializer: D
) throws -> UUID {
    let s = try deserializer.deserialize_str()
    guard let uuid = UUID(uuidString: s) else {
        throw DeserializationError.invalidInput(issue: "Invalid UUID string: \(s)")
    }
    return uuid
}

public struct StructWithUuid: Hashable, Equatable {
    public var id: UUID
    public var parentId: UUID?
    public var name: String

    public init(id: UUID, parentId: UUID?, name: String) {
        self.id = id
        self.parentId = parentId
        self.name = name
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializeUuid(value: self.id, serializer: serializer)
        try serializeOption(value: self.parentId, serializer: serializer) { value, serializer in
            try serializeUuid(value: value, serializer: serializer)
        }
        try serializer.serialize_str(value: self.name)
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> StructWithUuid {
        try deserializer.increase_container_depth()
        let id = try deserializeUuid(deserializer: deserializer)
        let parentId = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeUuid(deserializer: deserializer)
        }
        let name = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return StructWithUuid(id: id, parentId: parentId, name: name)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> StructWithUuid {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
