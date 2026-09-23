import A
import Serde

public struct App: Hashable, Equatable {
    public var row: A.Row

    public init(row: A.Row) {
        self.row = row
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.row.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> App {
        try deserializer.increase_container_depth()
        let row = try A.Row.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return App(row: row)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> App {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
