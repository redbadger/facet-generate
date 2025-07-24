import Foundation

struct ExplicitlyNamedStruct: Codable {
    var a_field: String
    var another_field: UInt32

    init(a_field: String, another_field: UInt32) {
        self.a_field = a_field
        self.another_field = another_field
    }
}

enum SomeEnum: Codable {
    case a
    case b(field1: String)
    case c(field1: UInt32, field2: Float)
    case d(field3: Bool?)
    case e(ExplicitlyNamedStruct)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case a = "A"
        case b = "B"
        case c = "C"
        case d = "D"
        case e = "E"
    }

    var type: `Type` {
        switch self {
        case .a: return .a
        case .b: return .b
        case .c: return .c
        case .d: return .d
        case .e: return .e
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case field1
        case field2
        case field3
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .a:
            self = .a
        case .b:
            let field1 = try container.decode(String.self, forKey: .field1)
            self = .b(field1: field1)
        case .c:
            let field1 = try container.decode(UInt32.self, forKey: .field1)
            let field2 = try container.decode(Float.self, forKey: .field2)
            self = .c(field1: field1, field2: field2)
        case .d:
            let field3 = try container.decodeIfPresent(Bool?.self, forKey: .field3).flatMap { $0 }
            self = .d(field3: field3)
        case .e:
            let content = try ExplicitlyNamedStruct(from: decoder)
            self = .e(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .a:
            try container.encode(`Type`.a, forKey: .type)
        case .b(let field1):
            try container.encode(`Type`.b, forKey: .type)
            try container.encode(field1, forKey: .field1)
        case .c(let field1, let field2):
            try container.encode(`Type`.c, forKey: .type)
            try container.encode(field1, forKey: .field1)
            try container.encode(field2, forKey: .field2)
        case .d(let field3):
            try container.encode(`Type`.d, forKey: .type)
            try container.encodeIfPresent(field3, forKey: .field3)
        case .e(let content):
            try container.encode(`Type`.e, forKey: .type)
            try content.encode(to: encoder)
        }
    }
}
