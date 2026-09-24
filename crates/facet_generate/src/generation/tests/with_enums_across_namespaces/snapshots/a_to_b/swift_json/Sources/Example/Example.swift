import A
import Serde

public struct App: Hashable, Equatable, Codable {
    public var row: A.Row

    public init(row: A.Row) {
        self.row = row
    }

    enum CodingKeys: String, CodingKey {
        case row
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> App {
        return try Serde.jsonDeserialize(App.self, from: input)
    }
}
