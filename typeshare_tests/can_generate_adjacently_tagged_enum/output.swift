import Foundation

/// Struct comment
struct ExplicitlyNamedStruct: Codable {
    /// Field comment
    var a: UInt32
    var b: UInt32

    init(a: UInt32, b: UInt32) {
        self.a = a
        self.b = b
    }
}

/// Enum comment
enum AdvancedColors: Codable {
    case unit
    /// This is a case comment
    case str(String)
    case number(Int32)
    case unsignedNumber(UInt32)
    case numberArray([Int32])
    case testWithAnonymousStruct(a: UInt32, b: UInt32)
    /// Comment on the last element
    case testWithExplicitlyNamedStruct(ExplicitlyNamedStruct)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case unit = "Unit"
        case str = "Str"
        case number = "Number"
        case unsignedNumber = "UnsignedNumber"
        case numberArray = "NumberArray"
        case testWithAnonymousStruct = "TestWithAnonymousStruct"
        case testWithExplicitlyNamedStruct = "TestWithExplicitlyNamedStruct"
    }

    var type: `Type` {
        switch self {
        case .unit: return .unit
        case .str: return .str
        case .number: return .number
        case .unsignedNumber: return .unsignedNumber
        case .numberArray: return .numberArray
        case .testWithAnonymousStruct: return .testWithAnonymousStruct
        case .testWithExplicitlyNamedStruct: return .testWithExplicitlyNamedStruct
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    private enum NestedCodingKeys: String, CodingKey {
        case a
        case b
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .unit:
            self = .unit
        case .str:
            let content = try container.decode(String.self, forKey: .content)
            self = .str(content)
        case .number:
            let content = try container.decode(Int32.self, forKey: .content)
            self = .number(content)
        case .unsignedNumber:
            let content = try container.decode(UInt32.self, forKey: .content)
            self = .unsignedNumber(content)
        case .numberArray:
            let content = try container.decode([Int32].self, forKey: .content)
            self = .numberArray(content)
        case .testWithAnonymousStruct:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let a = try container.decode(UInt32.self, forKey: .a)
            let b = try container.decode(UInt32.self, forKey: .b)
            self = .testWithAnonymousStruct(a: a, b: b)
        case .testWithExplicitlyNamedStruct:
            let content = try container.decode(ExplicitlyNamedStruct.self, forKey: .content)
            self = .testWithExplicitlyNamedStruct(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .unit:
            try container.encode(`Type`.unit, forKey: .type)
        case .str(let content):
            try container.encode(`Type`.str, forKey: .type)
            try container.encode(content, forKey: .content)
        case .number(let content):
            try container.encode(`Type`.number, forKey: .type)
            try container.encode(content, forKey: .content)
        case .unsignedNumber(let content):
            try container.encode(`Type`.unsignedNumber, forKey: .type)
            try container.encode(content, forKey: .content)
        case .numberArray(let content):
            try container.encode(`Type`.numberArray, forKey: .type)
            try container.encode(content, forKey: .content)
        case .testWithAnonymousStruct(let a, let b):
            try container.encode(`Type`.testWithAnonymousStruct, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(a, forKey: .a)
            try container.encode(b, forKey: .b)
        case .testWithExplicitlyNamedStruct(let content):
            try container.encode(`Type`.testWithExplicitlyNamedStruct, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

enum AdvancedColors2: Codable {
    /// This is a case comment
    case str(String)
    case number(Int32)
    case numberArray([Int32])
    /// Comment on the last element
    case reallyCoolType(ExplicitlyNamedStruct)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case str
        case number
        case numberArray = "number-array"
        case reallyCoolType = "really-cool-type"
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
            let content = try container.decode(ExplicitlyNamedStruct.self, forKey: .content)
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
