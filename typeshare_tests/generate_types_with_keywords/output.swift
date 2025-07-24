import Foundation

struct `catch`: Codable {
    var `default`: String
    var `case`: String

    init(default: String, case: String) {
        self.default = `default`
        self.case = `case`
    }
}

enum `throws`: String, Codable {
    case `case`
    case `default`
}

enum `switch`: Codable {
    case `default`(`catch`)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case `default`
    }

    var type: `Type` {
        switch self {
        case .`default`: return .`default`
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
        case .default:
            let content = try container.decode(`catch`.self, forKey: .content)
            self = .default(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .default(let content):
            try container.encode(`Type`.default, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}
