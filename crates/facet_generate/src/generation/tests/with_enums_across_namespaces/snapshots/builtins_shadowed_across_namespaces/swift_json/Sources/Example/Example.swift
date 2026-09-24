import Kit
import Serde

public struct Shelf: Hashable, Equatable, Codable {
    public var set: Kit.Set
    public var ids: Swift.Set<UInt32>
    public var unit: Unit

    public init(set: Kit.Set, ids: Swift.Set<UInt32>, unit: Unit) {
        self.set = set
        self.ids = ids
        self.unit = unit
    }

    enum CodingKeys: String, CodingKey {
        case set
        case ids
        case unit
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Shelf {
        return try Serde.jsonDeserialize(Shelf.self, from: input)
    }
}

public struct Unit: Hashable, Equatable, Codable {
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

    public static func jsonDeserialize(input: [UInt8]) throws -> Unit {
        return try Serde.jsonDeserialize(Unit.self, from: input)
    }
}
