import Foundation

struct SomeNamedStruct: Codable {
    var a_field: String
    var another_field: UInt32

    init(a_field: String, another_field: UInt32) {
        self.a_field = a_field
        self.another_field = another_field
    }
}

enum SomeResult: Codable {
    case ok(UInt32)
    case error(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case ok = "Ok"
        case error = "Error"
    }

    var type: `Type` {
        switch self {
        case .ok: return .ok
        case .error: return .error
        }
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: `Type`.self)
        
        if container.contains(.ok) {
            let content = try container.decode(UInt32.self, forKey: .ok)
            self = .ok(content)
            return
        }
        if container.contains(.error) {
            let content = try container.decode(String.self, forKey: .error)
            self = .error(content)
            return
        }
        throw DecodingError.typeMismatch(SomeResult.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong tag for SomeResult"))
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: `Type`.self)
        switch self {
        case .ok(let content):
            try container.encode(content, forKey: .ok)
        case .error(let content):
            try container.encode(content, forKey: .error)
        }
    }
}

enum SomeEnum: Codable {
    case a(field1: String)
    case b(field1: UInt32, field2: Float)
    case c(field3: Bool?)
    case d(UInt32)
    case e(SomeNamedStruct)
    case f(SomeNamedStruct?)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case a = "A"
        case b = "B"
        case c = "C"
        case d = "D"
        case e = "E"
        case f = "F"
    }

    var type: `Type` {
        switch self {
        case .a: return .a
        case .b: return .b
        case .c: return .c
        case .d: return .d
        case .e: return .e
        case .f: return .f
        }
    }

    private enum CodingKeys: String, CodingKey {
        case field1
        case field2
        case field3
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: `Type`.self)
        
        if container.contains(.a) {
            let container = try container.nestedContainer(keyedBy: CodingKeys.self, forKey: .a)
            let field1 = try container.decode(String.self, forKey: .field1)
            self = .a(field1: field1)
            return
        }
        if container.contains(.b) {
            let container = try container.nestedContainer(keyedBy: CodingKeys.self, forKey: .b)
            let field1 = try container.decode(UInt32.self, forKey: .field1)
            let field2 = try container.decode(Float.self, forKey: .field2)
            self = .b(field1: field1, field2: field2)
            return
        }
        if container.contains(.c) {
            let container = try container.nestedContainer(keyedBy: CodingKeys.self, forKey: .c)
            let field3 = try container.decodeIfPresent(Bool?.self, forKey: .field3).flatMap { $0 }
            self = .c(field3: field3)
            return
        }
        if container.contains(.d) {
            let content = try container.decode(UInt32.self, forKey: .d)
            self = .d(content)
            return
        }
        if container.contains(.e) {
            let content = try container.decode(SomeNamedStruct.self, forKey: .e)
            self = .e(content)
            return
        }
        if container.contains(.f) {
            let content = try container.decode(SomeNamedStruct?.self, forKey: .f)
            self = .f(content)
            return
        }
        throw DecodingError.typeMismatch(SomeEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong tag for SomeEnum"))
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: `Type`.self)
        switch self {
        case .a(let field1):
            var container = container.nestedContainer(keyedBy: CodingKeys.self, forKey: .a)
            try container.encode(field1, forKey: .field1)
        case .b(let field1, let field2):
            var container = container.nestedContainer(keyedBy: CodingKeys.self, forKey: .b)
            try container.encode(field1, forKey: .field1)
            try container.encode(field2, forKey: .field2)
        case .c(let field3):
            var container = container.nestedContainer(keyedBy: CodingKeys.self, forKey: .c)
            try container.encodeIfPresent(field3, forKey: .field3)
        case .d(let content):
            try container.encode(content, forKey: .d)
        case .e(let content):
            try container.encode(content, forKey: .e)
        case .f(let content):
            try container.encode(content, forKey: .f)
        }
    }
}
