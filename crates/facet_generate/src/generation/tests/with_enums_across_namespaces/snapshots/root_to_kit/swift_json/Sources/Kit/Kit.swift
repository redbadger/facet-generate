import Serde

public struct Badge: Hashable, Equatable, Codable {
    public var presence: Presence
    public var shape: Shape

    public init(presence: Presence, shape: Shape) {
        self.presence = presence
        self.shape = shape
    }

    enum CodingKeys: String, CodingKey {
        case presence
        case shape
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Badge {
        return try Serde.jsonDeserialize(Badge.self, from: input)
    }
}

indirect public enum Presence: Hashable, Equatable, Codable {
    case online
    case offline

    enum CodingKeys: String, CodingKey {
        case online = "Online"
        case offline = "Offline"
    }

    public init(from decoder: Decoder) throws {
        if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
            switch name {
            case "Online":
                self = .online
            case "Offline":
                self = .offline
            default:
                throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for Presence")
            }
            return
        }
        let container = try decoder.container(keyedBy: CodingKeys.self)
        guard container.allKeys.count == 1, let key = container.allKeys.first else {
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of Presence"))
        }
        switch key {
        case .online:
            self = .online
        case .offline:
            self = .offline
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .online:
            var container = encoder.singleValueContainer()
            try container.encode("Online")
        case .offline:
            var container = encoder.singleValueContainer()
            try container.encode("Offline")
        }
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Presence {
        return try Serde.jsonDeserialize(Presence.self, from: input)
    }
}

indirect public enum Shape: Hashable, Equatable, Codable {
    case circle(Double)
    case empty

    enum CodingKeys: String, CodingKey {
        case circle = "Circle"
        case empty = "Empty"
    }

    public init(from decoder: Decoder) throws {
        if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
            switch name {
            case "Empty":
                self = .empty
            default:
                throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for Shape")
            }
            return
        }
        let container = try decoder.container(keyedBy: CodingKeys.self)
        guard container.allKeys.count == 1, let key = container.allKeys.first else {
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of Shape"))
        }
        switch key {
        case .circle:
            self = .circle(
                try container.decode(Double.self, forKey: .circle)
            )
        case .empty:
            self = .empty
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .circle(let payload0):
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(payload0, forKey: .circle)
        case .empty:
            var container = encoder.singleValueContainer()
            try container.encode("Empty")
        }
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Shape {
        return try Serde.jsonDeserialize(Shape.self, from: input)
    }
}
