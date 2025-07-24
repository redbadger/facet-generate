import Foundation

/// Enum keeping track of who autofilled a field
enum AutofilledBy: Codable {
    /// This field was autofilled by us
    case us(uuid: String)
    /// Something else autofilled this field
    case somethingElse(uuid: String, thing: Int32)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case us = "Us"
        case somethingElse = "SomethingElse"
    }

    var type: `Type` {
        switch self {
        case .us: return .us
        case .somethingElse: return .somethingElse
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    private enum NestedCodingKeys: String, CodingKey {
        case uuid
        case thing
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .us:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let uuid = try container.decode(String.self, forKey: .uuid)
            self = .us(uuid: uuid)
        case .somethingElse:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let uuid = try container.decode(String.self, forKey: .uuid)
            let thing = try container.decode(Int32.self, forKey: .thing)
            self = .somethingElse(uuid: uuid, thing: thing)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .us(let uuid):
            try container.encode(`Type`.us, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(uuid, forKey: .uuid)
        case .somethingElse(let uuid, let thing):
            try container.encode(`Type`.somethingElse, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(uuid, forKey: .uuid)
            try container.encode(thing, forKey: .thing)
        }
    }
}

/// This is a comment (yareek sameek wuz here)
enum EnumWithManyVariants: Codable {
    case unitVariant
    case tupleVariantString(String)
    case anonVariant(uuid: String)
    case tupleVariantInt(Int32)
    case anotherUnitVariant
    case anotherAnonVariant(uuid: String, thing: Int32)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case unitVariant = "UnitVariant"
        case tupleVariantString = "TupleVariantString"
        case anonVariant = "AnonVariant"
        case tupleVariantInt = "TupleVariantInt"
        case anotherUnitVariant = "AnotherUnitVariant"
        case anotherAnonVariant = "AnotherAnonVariant"
    }

    var type: `Type` {
        switch self {
        case .unitVariant: return .unitVariant
        case .tupleVariantString: return .tupleVariantString
        case .anonVariant: return .anonVariant
        case .tupleVariantInt: return .tupleVariantInt
        case .anotherUnitVariant: return .anotherUnitVariant
        case .anotherAnonVariant: return .anotherAnonVariant
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    private enum NestedCodingKeys: String, CodingKey {
        case uuid
        case thing
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .unitVariant:
            self = .unitVariant
        case .tupleVariantString:
            let content = try container.decode(String.self, forKey: .content)
            self = .tupleVariantString(content)
        case .anonVariant:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let uuid = try container.decode(String.self, forKey: .uuid)
            self = .anonVariant(uuid: uuid)
        case .tupleVariantInt:
            let content = try container.decode(Int32.self, forKey: .content)
            self = .tupleVariantInt(content)
        case .anotherUnitVariant:
            self = .anotherUnitVariant
        case .anotherAnonVariant:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let uuid = try container.decode(String.self, forKey: .uuid)
            let thing = try container.decode(Int32.self, forKey: .thing)
            self = .anotherAnonVariant(uuid: uuid, thing: thing)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .unitVariant:
            try container.encode(`Type`.unitVariant, forKey: .type)
        case .tupleVariantString(let content):
            try container.encode(`Type`.tupleVariantString, forKey: .type)
            try container.encode(content, forKey: .content)
        case .anonVariant(let uuid):
            try container.encode(`Type`.anonVariant, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(uuid, forKey: .uuid)
        case .tupleVariantInt(let content):
            try container.encode(`Type`.tupleVariantInt, forKey: .type)
            try container.encode(content, forKey: .content)
        case .anotherUnitVariant:
            try container.encode(`Type`.anotherUnitVariant, forKey: .type)
        case .anotherAnonVariant(let uuid, let thing):
            try container.encode(`Type`.anotherAnonVariant, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(uuid, forKey: .uuid)
            try container.encode(thing, forKey: .thing)
        }
    }
}
