import Foundation

enum AnonymousStructWithRename: Codable {
    case list(list: [String])
    case longFieldNames(some_long_field_name: String, and: Bool, but_one_more: [String])
    case kebabCase(another-list: [String], camelCaseStringField: String, something-else: Bool)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case list
        case longFieldNames
        case kebabCase
    }

    var type: `Type` {
        switch self {
        case .list: return .list
        case .longFieldNames: return .longFieldNames
        case .kebabCase: return .kebabCase
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    private enum NestedCodingKeys: String, CodingKey {
        case list
        case some_long_field_name
        case and
        case but_one_more
        case another-list
        case camelCaseStringField
        case something-else
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .list:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let list = try container.decode([String].self, forKey: .list)
            self = .list(list: list)
        case .longFieldNames:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let some_long_field_name = try container.decode(String.self, forKey: .some_long_field_name)
            let and = try container.decode(Bool.self, forKey: .and)
            let but_one_more = try container.decode([String].self, forKey: .but_one_more)
            self = .longFieldNames(some_long_field_name: some_long_field_name, and: and, but_one_more: but_one_more)
        case .kebabCase:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let another-list = try container.decode([String].self, forKey: .another-list)
            let camelCaseStringField = try container.decode(String.self, forKey: .camelCaseStringField)
            let something-else = try container.decode(Bool.self, forKey: .something-else)
            self = .kebabCase(another-list: another-list, camelCaseStringField: camelCaseStringField, something-else: something-else)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .list(let list):
            try container.encode(`Type`.list, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(list, forKey: .list)
        case .longFieldNames(let some_long_field_name, let and, let but_one_more):
            try container.encode(`Type`.longFieldNames, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(some_long_field_name, forKey: .some_long_field_name)
            try container.encode(and, forKey: .and)
            try container.encode(but_one_more, forKey: .but_one_more)
        case .kebabCase(let another-list, let camelCaseStringField, let something-else):
            try container.encode(`Type`.kebabCase, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(another-list, forKey: .another-list)
            try container.encode(camelCaseStringField, forKey: .camelCaseStringField)
            try container.encode(something-else, forKey: .something-else)
        }
    }
}
