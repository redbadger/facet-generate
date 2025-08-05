import Foundation

struct NamedEmptyStruct: Codable {
    init() {}
}

enum Test: Codable {
    case namedEmptyStruct(NamedEmptyStruct)
    case anonymousEmptyStruct

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case namedEmptyStruct = "NamedEmptyStruct"
        case anonymousEmptyStruct = "AnonymousEmptyStruct"
    }

    var type: `Type` {
        switch self {
        case .namedEmptyStruct: return .namedEmptyStruct
        case .anonymousEmptyStruct: return .anonymousEmptyStruct
        }
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: `Type`.self)
        
        if container.contains(.namedEmptyStruct) {
            let content = try container.decode(NamedEmptyStruct.self, forKey: .namedEmptyStruct)
            self = .namedEmptyStruct(content)
            return
        }
        if container.contains(.anonymousEmptyStruct) {
            self = .anonymousEmptyStruct
            return
        }
        throw DecodingError.typeMismatch(Test.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong tag for Test"))
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: `Type`.self)
        switch self {
        case .namedEmptyStruct(let content):
            try container.encode(content, forKey: .namedEmptyStruct)
        case .anonymousEmptyStruct:
            try container.encode(CodableVoid(), forKey: .anonymousEmptyStruct)
        }
    }
}

/// () isn't codable, so we use this instead to represent Rust's unit type
struct CodableVoid: Codable, KeyPathMutable {
    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {}
}
