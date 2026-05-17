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
    let u = value.uuid
    let bytes: [UInt8] = [
        u.0,  u.1,  u.2,  u.3,  u.4,  u.5,  u.6,  u.7,
        u.8,  u.9,  u.10, u.11, u.12, u.13, u.14, u.15,
    ]
    try serializer.serialize_bytes(value: bytes)
}

func deserializeUuid<D: Deserializer>(
    deserializer: D
) throws -> UUID {
    let bytes = try deserializer.deserialize_bytes()
    guard bytes.count == 16 else {
        throw DeserializationError.invalidInput(issue: "UUID must be 16 bytes, got \(bytes.count)")
    }
    return UUID(uuid: (
        bytes[0],  bytes[1],  bytes[2],  bytes[3],
        bytes[4],  bytes[5],  bytes[6],  bytes[7],
        bytes[8],  bytes[9],  bytes[10], bytes[11],
        bytes[12], bytes[13], bytes[14], bytes[15]
    ))
}

public struct StructWithUuid: Hashable {
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

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
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

    public static func bincodeDeserialize(input: [UInt8]) throws -> StructWithUuid {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
