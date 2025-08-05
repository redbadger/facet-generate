import Foundation

@available(*, deprecated, message: "Use `MySuperAwesomeAlias` instead")
typealias MyLegacyAlias = UInt32

@available(*, deprecated, message: "Use `MySuperAwesomeStruct` instead")
struct MyLegacyStruct: Codable {
    var field: String

    init(field: String) {
        self.field = field
    }
}

@available(*, deprecated, message: "Use `MySuperAwesomeEnum` instead")
enum MyLegacyEnum: String, Codable {
    case variantA = "VariantA"
    case variantB = "VariantB"
    case variantC = "VariantC"
}

enum MyUnitEnum: String, Codable {
    case variantA = "VariantA"
    case variantB = "VariantB"
    @available(*, deprecated, message: "Use `VariantB` instead")
    case legacyVariant = "LegacyVariant"
}

enum MyInternallyTaggedEnum: Codable {
    case variantA(field: String)
    case variantB(field: UInt32)
    @available(*, deprecated, message: "Use `VariantA` instead")
    case legacyVariant(field: Bool)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case variantA = "VariantA"
        case variantB = "VariantB"
        case legacyVariant = "LegacyVariant"
    }

    var type: `Type` {
        switch self {
        case .variantA: return .variantA
        case .variantB: return .variantB
        case .legacyVariant: return .legacyVariant
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case field
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .variantA:
            let field = try container.decode(String.self, forKey: .field)
            self = .variantA(field: field)
        case .variantB:
            let field = try container.decode(UInt32.self, forKey: .field)
            self = .variantB(field: field)
        case .legacyVariant:
            let field = try container.decode(Bool.self, forKey: .field)
            self = .legacyVariant(field: field)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .variantA(let field):
            try container.encode(`Type`.variantA, forKey: .type)
            try container.encode(field, forKey: .field)
        case .variantB(let field):
            try container.encode(`Type`.variantB, forKey: .type)
            try container.encode(field, forKey: .field)
        case .legacyVariant(let field):
            try container.encode(`Type`.legacyVariant, forKey: .type)
            try container.encode(field, forKey: .field)
        }
    }
}

enum MyExternallyTaggedEnum: Codable {
    case variantA(String)
    case variantB(UInt32)
    @available(*, deprecated, message: "Use `VariantB` instead")
    case legacyVariant(Bool)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case variantA = "VariantA"
        case variantB = "VariantB"
        case legacyVariant = "LegacyVariant"
    }

    var type: `Type` {
        switch self {
        case .variantA: return .variantA
        case .variantB: return .variantB
        case .legacyVariant: return .legacyVariant
        }
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: `Type`.self)
        
        if container.contains(.variantA) {
            let content = try container.decode(String.self, forKey: .variantA)
            self = .variantA(content)
            return
        }
        if container.contains(.variantB) {
            let content = try container.decode(UInt32.self, forKey: .variantB)
            self = .variantB(content)
            return
        }
        if container.contains(.legacyVariant) {
            let content = try container.decode(Bool.self, forKey: .legacyVariant)
            self = .legacyVariant(content)
            return
        }
        throw DecodingError.typeMismatch(MyExternallyTaggedEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong tag for MyExternallyTaggedEnum"))
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: `Type`.self)
        switch self {
        case .variantA(let content):
            try container.encode(content, forKey: .variantA)
        case .variantB(let content):
            try container.encode(content, forKey: .variantB)
        case .legacyVariant(let content):
            try container.encode(content, forKey: .legacyVariant)
        }
    }
}
