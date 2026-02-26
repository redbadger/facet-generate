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

func serializeMap<K, V, S: Serializer>(
    value: [K: V],
    serializer: S,
    serializeEntry: (K, V, S) throws -> Void
) throws {
    try serializer.serialize_len(value: value.count)
    for (key, value) in value {
        try serializeEntry(key, value, serializer)
    }
}

func deserializeMap<K: Hashable, V, D: Deserializer>(
    deserializer: D,
    deserializeEntry: (D) throws -> (K, V)
) throws -> [K: V] {
    let length = try deserializer.deserialize_len()
    var obj: [K: V] = [:]
    for _ in 0..<length {
        let (key, value) = try deserializeEntry(deserializer)
        obj[key] = value
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

public struct Child: Hashable {
    @Indirect public var name: String

    public init(name: String) {
        self.name = name
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.name)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Child {
        try deserializer.increase_container_depth()
        let name = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return Child(name: name)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Child {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct MyStruct: Hashable {
    @Indirect public var stringToInt: [String: Int32]
    @Indirect public var mapToList: [String: [Int32]]
    @Indirect public var optionOfVecOfSet: [Set<String>]?
    @Indirect public var parent: Parent

    public init(stringToInt: [String: Int32], mapToList: [String: [Int32]], optionOfVecOfSet: [Set<String>]?, parent: Parent) {
        self.stringToInt = stringToInt
        self.mapToList = mapToList
        self.optionOfVecOfSet = optionOfVecOfSet
        self.parent = parent
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializeMap(value: self.stringToInt, serializer: serializer) { key, value, serializer in
            try serializer.serialize_str(value: key)
            try serializer.serialize_i32(value: value)
        }
        try serializeMap(value: self.mapToList, serializer: serializer) { key, value, serializer in
            try serializer.serialize_str(value: key)
            try serializeArray(value: value, serializer: serializer) { item, serializer in
                try serializer.serialize_i32(value: item)
            }
        }
        try serializeOption(value: self.optionOfVecOfSet, serializer: serializer) { value, serializer in
            try serializeArray(value: value, serializer: serializer) { item, serializer in
                try serializeSet(value: item, serializer: serializer) { item, serializer in
                    try serializer.serialize_str(value: item)
                }
            }
        }
        try self.parent.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> MyStruct {
        try deserializer.increase_container_depth()
        let stringToInt = try deserializeMap(deserializer: deserializer) { deserializer in
            let key = try deserializer.deserialize_str()
            let value = try deserializer.deserialize_i32()
            return (key, value)
        }
        let mapToList = try deserializeMap(deserializer: deserializer) { deserializer in
            let key = try deserializer.deserialize_str()
            let value = try deserializeArray(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_i32()
            }
            return (key, value)
        }
        let optionOfVecOfSet = try deserializeOption(deserializer: deserializer) { deserializer in
            try deserializeArray(deserializer: deserializer) { deserializer in
                try deserializeSet(deserializer: deserializer) { deserializer in
                    try deserializer.deserialize_str()
                }
            }
        }
        let parent = try Parent.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return MyStruct(stringToInt: stringToInt, mapToList: mapToList, optionOfVecOfSet: optionOfVecOfSet, parent: parent)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> MyStruct {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Parent: Hashable {
    case child(Child)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .child(let x):
            try serializer.serialize_variant_index(value: 0)
            try x.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Parent {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try Child.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .child(x)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Parent: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Parent {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
