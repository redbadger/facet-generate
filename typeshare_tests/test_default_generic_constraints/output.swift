import Foundation

struct GenericType<K: Codable & Identifiable & Sendable, V: Codable & Identifiable & Sendable>: Codable {
    var key: K
    var value: V

    init(key: K, value: V) {
        self.key = key
        self.value = value
    }
}

enum GenericEnum<K: Codable & Identifiable & Sendable, V: Codable & Identifiable & Sendable>: Codable {
    case variant(key: K, value: V)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case variant = "Variant"
    }

    var type: `Type` {
        switch self {
        case .variant: return .variant
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    private enum NestedCodingKeys: String, CodingKey {
        case key
        case value
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .variant:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let key = try container.decode(K.self, forKey: .key)
            let value = try container.decode(V.self, forKey: .value)
            self = .variant(key: key, value: value)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .variant(let key, let value):
            try container.encode(`Type`.variant, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(key, forKey: .key)
            try container.encode(value, forKey: .value)
        }
    }
}
