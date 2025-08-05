import Foundation

/// This struct has a unit field
struct StructHasVoidType: Codable {
    var thisIsAUnit: CodableVoid

    init(thisIsAUnit: CodableVoid) {
        self.thisIsAUnit = thisIsAUnit
    }
}

/// This enum has a variant associated with unit data
enum EnumHasVoidType: Codable {
    case hasAUnit(CodableVoid)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case hasAUnit
    }

    var type: `Type` {
        switch self {
        case .hasAUnit: return .hasAUnit
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
        case .hasAUnit:
            let content = try container.decode(CodableVoid.self, forKey: .content)
            self = .hasAUnit(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .hasAUnit(let content):
            try container.encode(`Type`.hasAUnit, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

/// () isn't codable, so we use this instead to represent Rust's unit type
struct CodableVoid: Codable, KeyPathMutable {
    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {}
}
