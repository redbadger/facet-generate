import Serde

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
        return Child.init(name: name)
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
    @Indirect public var optionOfVecOfSet: [[String]]?
    @Indirect public var parent: Parent

    public init(stringToInt: [String: Int32], mapToList: [String: [Int32]], optionOfVecOfSet: [[String]]?, parent: Parent) {
        self.stringToInt = stringToInt
        self.mapToList = mapToList
        self.optionOfVecOfSet = optionOfVecOfSet
        self.parent = parent
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serialize_map_str_to_i32(value: self.stringToInt, serializer: serializer)
        try serialize_map_str_to_vector_i32(value: self.mapToList, serializer: serializer)
        try serialize_option_vector_set_str(value: self.optionOfVecOfSet, serializer: serializer)
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
        let stringToInt = try deserialize_map_str_to_i32(deserializer: deserializer)
        let mapToList = try deserialize_map_str_to_vector_i32(deserializer: deserializer)
        let optionOfVecOfSet = try deserialize_option_vector_set_str(deserializer: deserializer)
        let parent = try Parent.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return MyStruct.init(stringToInt: stringToInt, mapToList: mapToList, optionOfVecOfSet: optionOfVecOfSet, parent: parent)
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

func serialize_map_str_to_i32<S: Serializer>(value: [String: Int32], serializer: S) throws {
    try serializer.serialize_len(value: value.count)
    var offsets : [Int]  = []
    for (key, value) in value {
        offsets.append(serializer.get_buffer_offset())
        try serializer.serialize_str(value: key)
        try serializer.serialize_i32(value: value)
    }
    serializer.sort_map_entries(offsets: offsets)
}

func deserialize_map_str_to_i32<D: Deserializer>(deserializer: D) throws -> [String: Int32] {
    let length = try deserializer.deserialize_len()
    var obj : [String: Int32] = [:]
    var previous_slice = Slice(start: 0, end: 0)
    for i in 0..<length {
        var slice = Slice(start: 0, end: 0)
        slice.start = deserializer.get_buffer_offset()
        let key = try deserializer.deserialize_str()
        slice.end = deserializer.get_buffer_offset()
        if i > 0 {
            try deserializer.check_that_key_slices_are_increasing(key1: previous_slice, key2: slice)
        }
        previous_slice = slice
        obj[key] = try deserializer.deserialize_i32()
    }
    return obj
}

func serialize_map_str_to_vector_i32<S: Serializer>(value: [String: [Int32]], serializer: S) throws {
    try serializer.serialize_len(value: value.count)
    var offsets : [Int]  = []
    for (key, value) in value {
        offsets.append(serializer.get_buffer_offset())
        try serializer.serialize_str(value: key)
        try serialize_vector_i32(value: value, serializer: serializer)
    }
    serializer.sort_map_entries(offsets: offsets)
}

func deserialize_map_str_to_vector_i32<D: Deserializer>(deserializer: D) throws -> [String: [Int32]] {
    let length = try deserializer.deserialize_len()
    var obj : [String: [Int32]] = [:]
    var previous_slice = Slice(start: 0, end: 0)
    for i in 0..<length {
        var slice = Slice(start: 0, end: 0)
        slice.start = deserializer.get_buffer_offset()
        let key = try deserializer.deserialize_str()
        slice.end = deserializer.get_buffer_offset()
        if i > 0 {
            try deserializer.check_that_key_slices_are_increasing(key1: previous_slice, key2: slice)
        }
        previous_slice = slice
        obj[key] = try deserialize_vector_i32(deserializer: deserializer)
    }
    return obj
}

func serialize_option_vector_set_str<S: Serializer>(value: [[String]]?, serializer: S) throws {
    if let value = value {
        try serializer.serialize_option_tag(value: true)
        try serialize_vector_set_str(value: value, serializer: serializer)
    } else {
        try serializer.serialize_option_tag(value: false)
    }
}

func deserialize_option_vector_set_str<D: Deserializer>(deserializer: D) throws -> [[String]]? {
    let tag = try deserializer.deserialize_option_tag()
    if tag {
        return try deserialize_vector_set_str(deserializer: deserializer)
    } else {
        return nil
    }
}

func serialize_set_str<S: Serializer>(value: [String], serializer: S) throws {
    try serializer.serialize_len(value: value.count)
    for item in value {
        try serializer.serialize_str(value: item)
    }
}

func deserialize_set_str<D: Deserializer>(deserializer: D) throws -> [String] {
    let length = try deserializer.deserialize_len()
    var obj : [String] = []
    for _ in 0..<length {
        obj.append(try deserializer.deserialize_str())
    }
    return obj
}

func serialize_vector_i32<S: Serializer>(value: [Int32], serializer: S) throws {
    try serializer.serialize_len(value: value.count)
    for item in value {
        try serializer.serialize_i32(value: item)
    }
}

func deserialize_vector_i32<D: Deserializer>(deserializer: D) throws -> [Int32] {
    let length = try deserializer.deserialize_len()
    var obj : [Int32] = []
    for _ in 0..<length {
        obj.append(try deserializer.deserialize_i32())
    }
    return obj
}

func serialize_vector_set_str<S: Serializer>(value: [[String]], serializer: S) throws {
    try serializer.serialize_len(value: value.count)
    for item in value {
        try serialize_set_str(value: item, serializer: serializer)
    }
}

func deserialize_vector_set_str<D: Deserializer>(deserializer: D) throws -> [[String]] {
    let length = try deserializer.deserialize_len()
    var obj : [[String]] = []
    for _ in 0..<length {
        obj.append(try deserialize_set_str(deserializer: deserializer))
    }
    return obj
}

