import Example
import Serde

public struct Entry: Hashable, Equatable, Codable {
    public var level: Example.Level
    public var outcome: Example.Outcome

    public init(level: Example.Level, outcome: Example.Outcome) {
        self.level = level
        self.outcome = outcome
    }

    enum CodingKeys: String, CodingKey {
        case level
        case outcome
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Entry {
        return try Serde.jsonDeserialize(Entry.self, from: input)
    }
}
