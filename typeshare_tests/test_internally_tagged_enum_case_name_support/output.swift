import Foundation

enum AdvancedEnum: Codable {
    case unitVariant
    case anonymousStruct(field1: String)
    case otherAnonymousStruct(field1: UInt32, field2: Float)
    case rename(field3: Bool?)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case unitVariant
        case anonymousStruct = "A"
        case otherAnonymousStruct
        case rename = "B"
    }

    var type: `Type` {
        switch self {
        case .unitVariant: return .unitVariant
        case .anonymousStruct: return .anonymousStruct
        case .otherAnonymousStruct: return .otherAnonymousStruct
        case .rename: return .rename
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case field1
        case field2
        case field3
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .unitVariant:
            self = .unitVariant
        case .anonymousStruct:
            let field1 = try container.decode(String.self, forKey: .field1)
            self = .anonymousStruct(field1: field1)
        case .otherAnonymousStruct:
            let field1 = try container.decode(UInt32.self, forKey: .field1)
            let field2 = try container.decode(Float.self, forKey: .field2)
            self = .otherAnonymousStruct(field1: field1, field2: field2)
        case .rename:
            let field3 = try container.decodeIfPresent(Bool?.self, forKey: .field3).flatMap { $0 }
            self = .rename(field3: field3)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .unitVariant:
            try container.encode(`Type`.unitVariant, forKey: .type)
        case .anonymousStruct(let field1):
            try container.encode(`Type`.anonymousStruct, forKey: .type)
            try container.encode(field1, forKey: .field1)
        case .otherAnonymousStruct(let field1, let field2):
            try container.encode(`Type`.otherAnonymousStruct, forKey: .type)
            try container.encode(field1, forKey: .field1)
            try container.encode(field2, forKey: .field2)
        case .rename(let field3):
            try container.encode(`Type`.rename, forKey: .type)
            try container.encodeIfPresent(field3, forKey: .field3)
        }
    }
}
