import Example
import Serde

public struct Entry: Hashable, Equatable {
    public var level: Example.Level
    public var outcome: Example.Outcome

    public init(level: Example.Level, outcome: Example.Outcome) {
        self.level = level
        self.outcome = outcome
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.level.serialize(serializer: serializer)
        try self.outcome.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Entry {
        try deserializer.increase_container_depth()
        let level = try Example.Level.deserialize(deserializer: deserializer)
        let outcome = try Example.Outcome.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return Entry(level: level, outcome: outcome)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Entry {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
