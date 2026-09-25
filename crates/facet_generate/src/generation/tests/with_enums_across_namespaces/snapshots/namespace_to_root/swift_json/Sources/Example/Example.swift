import Serde

indirect public enum Level: Hashable, Equatable, Codable {
    case low
    case high

    enum CodingKeys: String, CodingKey {
        case low = "Low"
        case high = "High"
    }

    public init(from decoder: Decoder) throws {
        if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
            switch name {
            case "Low":
                self = .low
            case "High":
                self = .high
            default:
                throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for Level")
            }
            return
        }
        let container = try decoder.container(keyedBy: CodingKeys.self)
        guard container.allKeys.count == 1, let key = container.allKeys.first else {
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of Level"))
        }
        switch key {
        case .low:
            self = .low
        case .high:
            self = .high
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .low:
            var container = encoder.singleValueContainer()
            try container.encode("Low")
        case .high:
            var container = encoder.singleValueContainer()
            try container.encode("High")
        }
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Level {
        return try Serde.jsonDeserialize(Level.self, from: input)
    }
}

indirect public enum Outcome: Hashable, Equatable, Codable {
    case score(UInt32)
    case missing

    enum CodingKeys: String, CodingKey {
        case score = "Score"
        case missing = "Missing"
    }

    public init(from decoder: Decoder) throws {
        if let container = try? decoder.singleValueContainer(), let name = try? container.decode(String.self) {
            switch name {
            case "Missing":
                self = .missing
            default:
                throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unknown variant \(name) for Outcome")
            }
            return
        }
        let container = try decoder.container(keyedBy: CodingKeys.self)
        guard container.allKeys.count == 1, let key = container.allKeys.first else {
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Expected exactly one variant of Outcome"))
        }
        switch key {
        case .score:
            self = .score(
                try container.decode(UInt32.self, forKey: .score)
            )
        case .missing:
            self = .missing
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .score(let payload0):
            var container = encoder.container(keyedBy: CodingKeys.self)
            try container.encode(payload0, forKey: .score)
        case .missing:
            var container = encoder.singleValueContainer()
            try container.encode("Missing")
        }
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Outcome {
        return try Serde.jsonDeserialize(Outcome.self, from: input)
    }
}
