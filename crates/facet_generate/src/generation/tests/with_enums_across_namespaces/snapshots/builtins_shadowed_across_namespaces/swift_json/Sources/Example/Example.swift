import Kit
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

public struct Shelf: Hashable, Equatable {
    public var set: Kit.Set
    public var ids: Swift.Set<UInt32>
    public var unit: Unit

    public init(set: Kit.Set, ids: Swift.Set<UInt32>, unit: Unit) {
        self.set = set
        self.ids = ids
        self.unit = unit
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.set.serialize(serializer: serializer)
        try serializeSet(value: self.ids, serializer: serializer) { item, serializer in
            try serializer.serialize_u32(value: item)
        }
        try self.unit.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Shelf {
        try deserializer.increase_container_depth()
        let set = try Kit.Set.deserialize(deserializer: deserializer)
        let ids = try deserializeSet(deserializer: deserializer) { deserializer in
            try deserializer.deserialize_u32()
        }
        let unit = try Unit.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return Shelf(set: set, ids: ids, unit: unit)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Shelf {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct Unit: Hashable, Equatable {
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

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Unit {
        try deserializer.increase_container_depth()
        let value = try deserializer.deserialize_u32()
        try deserializer.decrease_container_depth()
        return Unit(value: value)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Unit {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
