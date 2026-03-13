import Foundation

public enum Engine {

struct ItemDetailsFieldValue: Codable {
    var hello: String

    init(hello: String) {
        self.hello = hello
    }
}

enum AdvancedColors: Codable {
    case str(String)
    case number(Int32)
    case numberArray([Int32])
    case reallyCoolType(ItemDetailsFieldValue)
    case arrayReallyCoolType([ItemDetailsFieldValue])
    case dictionaryReallyCoolType([String: ItemDetailsFieldValue])

    enum T: String, CodingKey, Codable, CaseIterable {
        case str = "Str"
        case number = "Number"
        case numberArray = "NumberArray"
        case reallyCoolType = "ReallyCoolType"
        case arrayReallyCoolType = "ArrayReallyCoolType"
        case dictionaryReallyCoolType = "DictionaryReallyCoolType"
    }

    var t: T {
        switch self {
        case .str: return .str
        case .number: return .number
        case .numberArray: return .numberArray
        case .reallyCoolType: return .reallyCoolType
        case .arrayReallyCoolType: return .arrayReallyCoolType
        case .dictionaryReallyCoolType: return .dictionaryReallyCoolType
        }
    }

    private enum CodingKeys: String, CodingKey {
        case t
        case c
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let t = try container.decode(T.self, forKey: .t)
        switch t {
        case .str:
            let content = try container.decode(String.self, forKey: .c)
            self = .str(content)
        case .number:
            let content = try container.decode(Int32.self, forKey: .c)
            self = .number(content)
        case .numberArray:
            let content = try container.decode([Int32].self, forKey: .c)
            self = .numberArray(content)
        case .reallyCoolType:
            let content = try container.decode(ItemDetailsFieldValue.self, forKey: .c)
            self = .reallyCoolType(content)
        case .arrayReallyCoolType:
            let content = try container.decode([ItemDetailsFieldValue].self, forKey: .c)
            self = .arrayReallyCoolType(content)
        case .dictionaryReallyCoolType:
            let content = try container.decode([String: ItemDetailsFieldValue].self, forKey: .c)
            self = .dictionaryReallyCoolType(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .str(let content):
            try container.encode(T.str, forKey: .t)
            try container.encode(content, forKey: .c)
        case .number(let content):
            try container.encode(T.number, forKey: .t)
            try container.encode(content, forKey: .c)
        case .numberArray(let content):
            try container.encode(T.numberArray, forKey: .t)
            try container.encode(content, forKey: .c)
        case .reallyCoolType(let content):
            try container.encode(T.reallyCoolType, forKey: .t)
            try container.encode(content, forKey: .c)
        case .arrayReallyCoolType(let content):
            try container.encode(T.arrayReallyCoolType, forKey: .t)
            try container.encode(content, forKey: .c)
        case .dictionaryReallyCoolType(let content):
            try container.encode(T.dictionaryReallyCoolType, forKey: .t)
            try container.encode(content, forKey: .c)
        }
    }
}
}
