import Serde

public struct Set: Hashable, Equatable, Codable {
    public var value: UInt32

    public init(value: UInt32) {
        self.value = value
    }

    enum CodingKeys: String, CodingKey {
        case value
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Set {
        return try Serde.jsonDeserialize(Set.self, from: input)
    }
}

public struct Tray: Codable {
    public var nothing: Void
    public var ids: Swift.Set<UInt32>

    public init(nothing: Void, ids: Swift.Set<UInt32>) {
        self.nothing = nothing
        self.ids = ids
    }

    enum CodingKeys: String, CodingKey {
        case nothing
        case ids
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.nothing = try container.decode(Serde.JsonUnit.self, forKey: .nothing).value
        self.ids = try container.decode(Swift.Set<UInt32>.self, forKey: .ids)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(Serde.JsonUnit(), forKey: .nothing)
        try container.encode(self.ids, forKey: .ids)
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Tray {
        return try Serde.jsonDeserialize(Tray.self, from: input)
    }
}
