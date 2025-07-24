import Foundation

struct NamedEmptyStruct: Codable {
    init() {}
}

enum Test: Codable {
    case namedEmptyStruct(NamedEmptyStruct)
    case anonymousEmptyStruct
    case noStruct

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case namedEmptyStruct = "NamedEmptyStruct"
        case anonymousEmptyStruct = "AnonymousEmptyStruct"
        case noStruct = "NoStruct"
    }

    var type: `Type` {
        switch self {
        case .namedEmptyStruct: return .namedEmptyStruct
        case .anonymousEmptyStruct: return .anonymousEmptyStruct
        case .noStruct: return .noStruct
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
        case .namedEmptyStruct:
            let content = try container.decode(NamedEmptyStruct.self, forKey: .content)
            self = .namedEmptyStruct(content)
        case .anonymousEmptyStruct:
            self = .anonymousEmptyStruct
        case .noStruct:
            self = .noStruct
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .namedEmptyStruct(let content):
            try container.encode(`Type`.namedEmptyStruct, forKey: .type)
            try container.encode(content, forKey: .content)
        case .anonymousEmptyStruct:
            try container.encode(`Type`.anonymousEmptyStruct, forKey: .type)
            try container.encode(CodableVoid(), forKey: .content)
        case .noStruct:
            try container.encode(`Type`.noStruct, forKey: .type)
        }
    }
}

/// () isn't codable, so we use this instead to represent Rust's unit type
struct CodableVoid: Codable, KeyPathMutable {
    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {}
}
