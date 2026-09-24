import Foundation
import Serde

public struct StructWithUuid: Hashable, Equatable, Codable {
    public var id: UUID
    public var parentId: UUID?
    public var name: String

    public init(id: UUID, parentId: UUID?, name: String) {
        self.id = id
        self.parentId = parentId
        self.name = name
    }

    enum CodingKeys: String, CodingKey {
        case id
        case parentId = "parent_id"
        case name
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.id = try container.decode(Serde.JsonUuid.self, forKey: .id).value
        self.parentId = try { () throws -> UUID? in
            guard container.contains(.parentId), try !container.decodeNil(forKey: .parentId) else { return nil }
            return try container.decode(Serde.JsonUuid.self, forKey: .parentId).value
        }()
        self.name = try container.decode(String.self, forKey: .name)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(Serde.JsonUuid(self.id), forKey: .id)
        if let value0 = self.parentId {
            try container.encode(Serde.JsonUuid(value0), forKey: .parentId)
        } else {
            try container.encodeNil(forKey: .parentId)
        }
        try container.encode(self.name, forKey: .name)
    }

    public func jsonSerialize() throws -> [UInt8] {
        return try Serde.jsonSerialize(self)
    }

    public static func jsonDeserialize(input: [UInt8]) throws -> StructWithUuid {
        return try Serde.jsonDeserialize(StructWithUuid.self, from: input)
    }
}
