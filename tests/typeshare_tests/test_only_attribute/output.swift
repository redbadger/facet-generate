import Foundation

struct StructOnlyInSwift: Codable {
    var field: String

    init(field: String) {
        self.field = field
    }
}

struct Struct: Codable {
    var only_in_swift: String

    init(only_in_swift: String) {
        self.only_in_swift = only_in_swift
    }
}

enum Enum: Codable {
    case onlyInSwift(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case onlyInSwift = "OnlyInSwift"
    }

    var type: `Type` {
        switch self {
        case .onlyInSwift: return .onlyInSwift
        }
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: `Type`.self)
        
        if container.contains(.onlyInSwift) {
            let content = try container.decode(String.self, forKey: .onlyInSwift)
            self = .onlyInSwift(content)
            return
        }
        throw DecodingError.typeMismatch(Enum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong tag for Enum"))
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: `Type`.self)
        switch self {
        case .onlyInSwift(let content):
            try container.encode(content, forKey: .onlyInSwift)
        }
    }
}
