import Foundation

struct ItemDetailsFieldValue: Codable {
    init() {}
}

enum AdvancedColors: Codable {
    case str(String)
    case number(Int32)
    case numberArray([Int32])
    case reallyCoolType(ItemDetailsFieldValue)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case str
        case number
        case numberArray = "number-array"
        case reallyCoolType
    }

    var type: `Type` {
        switch self {
        case .str: return .str
        case .number: return .number
        case .numberArray: return .numberArray
        case .reallyCoolType: return .reallyCoolType
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .str:
            let content = try container.decode(String.self, forKey: .content)
            self = .str(content)
        case .number:
            let content = try container.decode(Int32.self, forKey: .content)
            self = .number(content)
        case .numberArray:
            let content = try container.decode([Int32].self, forKey: .content)
            self = .numberArray(content)
        case .reallyCoolType:
            let content = try container.decode(ItemDetailsFieldValue.self, forKey: .content)
            self = .reallyCoolType(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .str(let content):
            try container.encode(`Type`.str, forKey: .type)
            try container.encode(content, forKey: .content)
        case .number(let content):
            try container.encode(`Type`.number, forKey: .type)
            try container.encode(content, forKey: .content)
        case .numberArray(let content):
            try container.encode(`Type`.numberArray, forKey: .type)
            try container.encode(content, forKey: .content)
        case .reallyCoolType(let content):
            try container.encode(`Type`.reallyCoolType, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}
