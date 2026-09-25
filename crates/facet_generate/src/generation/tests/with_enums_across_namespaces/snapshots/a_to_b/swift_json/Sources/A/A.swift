import B
import Serde

public struct Row: Hashable, Equatable, Codable {
    public var status: B.Status
    public var signal: B.Signal

    public init(status: B.Status, signal: B.Signal) {
        self.status = status
        self.signal = signal
    }

    enum CodingKeys: String, CodingKey {
        case status
        case signal
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> Row {
        return try Serde.jsonDeserialize(Row.self, from: input)
    }
}
