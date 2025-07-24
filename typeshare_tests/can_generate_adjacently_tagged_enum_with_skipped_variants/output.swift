import Foundation

enum SomeEnum: Codable {
    case a
    case c(Int32)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case a = "A"
        case c = "C"
    }

    var type: `Type` {
        switch self {
        case .a: return .a
        case .c: return .c
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
        case .a:
            self = .a
        case .c:
            let content = try container.decode(Int32.self, forKey: .content)
            self = .c(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .a:
            try container.encode(`Type`.a, forKey: .type)
        case .c(let content):
            try container.encode(`Type`.c, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}
