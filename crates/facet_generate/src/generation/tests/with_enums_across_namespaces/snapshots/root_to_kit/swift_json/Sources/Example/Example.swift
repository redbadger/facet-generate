import Kit
import Serde

public struct Card: Hashable, Equatable, Codable {
    public var presence: Kit.Presence
    public var shape: Kit.Shape
    public var shapes: [Kit.Shape?]
    public var badge: Kit.Badge

    public init(presence: Kit.Presence, shape: Kit.Shape, shapes: [Kit.Shape?], badge: Kit.Badge) {
        self.presence = presence
        self.shape = shape
        self.shapes = shapes
        self.badge = badge
    }

    enum CodingKeys: String, CodingKey {
        case presence
        case shape
        case shapes
        case badge
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Card {
        return try Serde.jsonDeserialize(Card.self, from: input)
    }
}

public struct Presence: Hashable, Equatable, Codable {
    public var since: UInt64

    public init(since: UInt64) {
        self.since = since
    }

    enum CodingKeys: String, CodingKey {
        case since
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Presence {
        return try Serde.jsonDeserialize(Presence.self, from: input)
    }
}

public struct Sighting: Hashable, Equatable, Codable {
    public var lastSeen: Presence

    public init(lastSeen: Presence) {
        self.lastSeen = lastSeen
    }

    enum CodingKeys: String, CodingKey {
        case lastSeen = "last_seen"
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Sighting {
        return try Serde.jsonDeserialize(Sighting.self, from: input)
    }
}
