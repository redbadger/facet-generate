import B
import Serde

public struct Row: Hashable, Equatable {
    public var status: B.Status
    public var signal: B.Signal

    public init(status: B.Status, signal: B.Signal) {
        self.status = status
        self.signal = signal
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try self.status.serialize(serializer: serializer)
        try self.signal.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func jsonSerialize() throws -> [UInt8] {
        let serializer = JsonSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Row {
        try deserializer.increase_container_depth()
        let status = try B.Status.deserialize(deserializer: deserializer)
        let signal = try B.Signal.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return Row(status: status, signal: signal)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Row {
        let deserializer = JsonDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
