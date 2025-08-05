import Foundation

enum GenericEnum<A: Codable, B: Codable>: Codable {
    case variantA(A)
    case variantB(B)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case variantA = "VariantA"
        case variantB = "VariantB"
    }

    var type: `Type` {
        switch self {
        case .variantA: return .variantA
        case .variantB: return .variantB
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
        case .variantA:
            let content = try container.decode(A.self, forKey: .content)
            self = .variantA(content)
        case .variantB:
            let content = try container.decode(B.self, forKey: .content)
            self = .variantB(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .variantA(let content):
            try container.encode(`Type`.variantA, forKey: .type)
            try container.encode(content, forKey: .content)
        case .variantB(let content):
            try container.encode(`Type`.variantB, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

struct StructUsingGenericEnum: Codable {
    var enum_field: GenericEnum<String, Int16>

    init(enum_field: GenericEnum<String, Int16>) {
        self.enum_field = enum_field
    }
}

enum GenericEnumUsingGenericEnum<T: Codable>: Codable {
    case variantC(GenericEnum<T, T>)
    case variantD(GenericEnum<String, [String: T]>)
    case variantE(GenericEnum<String, UInt32>)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case variantC = "VariantC"
        case variantD = "VariantD"
        case variantE = "VariantE"
    }

    var type: `Type` {
        switch self {
        case .variantC: return .variantC
        case .variantD: return .variantD
        case .variantE: return .variantE
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
        case .variantC:
            let content = try container.decode(GenericEnum<T, T>.self, forKey: .content)
            self = .variantC(content)
        case .variantD:
            let content = try container.decode(GenericEnum<String, [String: T]>.self, forKey: .content)
            self = .variantD(content)
        case .variantE:
            let content = try container.decode(GenericEnum<String, UInt32>.self, forKey: .content)
            self = .variantE(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .variantC(let content):
            try container.encode(`Type`.variantC, forKey: .type)
            try container.encode(content, forKey: .content)
        case .variantD(let content):
            try container.encode(`Type`.variantD, forKey: .type)
            try container.encode(content, forKey: .content)
        case .variantE(let content):
            try container.encode(`Type`.variantE, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

enum GenericEnumsUsingStructVariants<T: Codable, U: Codable>: Codable {
    case variantF(action: T)
    case variantG(action: T, response: U)
    case variantH(non_generic: Int32)
    case variantI(vec: [T], action: MyType<T, U>)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case variantF = "VariantF"
        case variantG = "VariantG"
        case variantH = "VariantH"
        case variantI = "VariantI"
    }

    var type: `Type` {
        switch self {
        case .variantF: return .variantF
        case .variantG: return .variantG
        case .variantH: return .variantH
        case .variantI: return .variantI
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    private enum NestedCodingKeys: String, CodingKey {
        case action
        case response
        case non_generic
        case vec
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .variantF:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let action = try container.decode(T.self, forKey: .action)
            self = .variantF(action: action)
        case .variantG:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let action = try container.decode(T.self, forKey: .action)
            let response = try container.decode(U.self, forKey: .response)
            self = .variantG(action: action, response: response)
        case .variantH:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let non_generic = try container.decode(Int32.self, forKey: .non_generic)
            self = .variantH(non_generic: non_generic)
        case .variantI:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let vec = try container.decode([T].self, forKey: .vec)
            let action = try container.decode(MyType<T, U>.self, forKey: .action)
            self = .variantI(vec: vec, action: action)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .variantF(let action):
            try container.encode(`Type`.variantF, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(action, forKey: .action)
        case .variantG(let action, let response):
            try container.encode(`Type`.variantG, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(action, forKey: .action)
            try container.encode(response, forKey: .response)
        case .variantH(let non_generic):
            try container.encode(`Type`.variantH, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(non_generic, forKey: .non_generic)
        case .variantI(let vec, let action):
            try container.encode(`Type`.variantI, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(vec, forKey: .vec)
            try container.encode(action, forKey: .action)
        }
    }
}
