import Serde

indirect public enum Signal: Hashable, Equatable, Codable {
    case level(UInt8)
    case silent

    enum CodingKeys: String, CodingKey {
        case level = "Level"
        case silent = "Silent"
    }

    public init(from decoder: Decoder) throws {
        if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
            switch name {
            case "Silent":
                self = .silent
            default:
                throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for Signal")
            }
            return
        }
        let container = try decoder.container(keyedBy: CodingKeys.self)
        guard container.allKeys.count == 1, let key = container.allKeys.first else {
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of Signal"))
        }
        switch key {
        case .level:
            self = .level(
                try container.decode(UInt8.self, forKey: .level)
            )
        case .silent:
            self = .silent
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .level(let payload0):
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(payload0, forKey: .level)
        case .silent:
            var container = encoder.singleValueContainer()
            try container.encode("Silent")
        }
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Signal {
        return try Serde.jsonDeserialize(Signal.self, from: input)
    }
}

indirect public enum Status: Hashable, Equatable, Codable {
    case up
    case down

    enum CodingKeys: String, CodingKey {
        case up = "Up"
        case down = "Down"
    }

    public init(from decoder: Decoder) throws {
        if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
            switch name {
            case "Up":
                self = .up
            case "Down":
                self = .down
            default:
                throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for Status")
            }
            return
        }
        let container = try decoder.container(keyedBy: CodingKeys.self)
        guard container.allKeys.count == 1, let key = container.allKeys.first else {
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of Status"))
        }
        switch key {
        case .up:
            self = .up
        case .down:
            self = .down
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .up:
            var container = encoder.singleValueContainer()
            try container.encode("Up")
        case .down:
            var container = encoder.singleValueContainer()
            try container.encode("Down")
        }
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Status {
        return try Serde.jsonDeserialize(Status.self, from: input)
    }
}
