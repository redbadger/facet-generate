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

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Child {
        try deserializer.increase_container_depth()
        let name = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return Child.init(name: name)
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
}

