import External
import Serde

public struct Child: Hashable {
    @Indirect public var external: External.ExternalParent

    public init(external: External.ExternalParent) {
        self.external = external
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.external.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Child {
        try deserializer.increase_container_depth()
        let external = try External.ExternalParent.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return Child.init(external: external)
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
        default:
            throw DeserializationError.invalidInput(
                issue: "Unknown variant index for Parent: \(index)")
        }
    }
}
