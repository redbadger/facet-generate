import Foundation

struct OverrideStruct: Codable {
    var fieldToOverride: Int

    init(fieldToOverride: Int) {
        self.fieldToOverride = fieldToOverride
    }
}

enum OverrideEnum: Codable {
    case unitVariant
    case tupleVariant(String)
    case anonymousStructVariant(fieldToOverride: String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case unitVariant = "UnitVariant"
        case tupleVariant = "TupleVariant"
        case anonymousStructVariant = "AnonymousStructVariant"
    }

    var type: `Type` {
        switch self {
        case .unitVariant: return .unitVariant
        case .tupleVariant: return .tupleVariant
        case .anonymousStructVariant: return .anonymousStructVariant
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    private enum NestedCodingKeys: String, CodingKey {
        case fieldToOverride
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .unitVariant:
            self = .unitVariant
        case .tupleVariant:
            let content = try container.decode(String.self, forKey: .content)
            self = .tupleVariant(content)
        case .anonymousStructVariant:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let fieldToOverride = try container.decode(String.self, forKey: .fieldToOverride)
            self = .anonymousStructVariant(fieldToOverride: fieldToOverride)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .unitVariant:
            try container.encode(`Type`.unitVariant, forKey: .type)
        case .tupleVariant(let content):
            try container.encode(`Type`.tupleVariant, forKey: .type)
            try container.encode(content, forKey: .content)
        case .anonymousStructVariant(let fieldToOverride):
            try container.encode(`Type`.anonymousStructVariant, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(fieldToOverride, forKey: .fieldToOverride)
        }
    }
}
