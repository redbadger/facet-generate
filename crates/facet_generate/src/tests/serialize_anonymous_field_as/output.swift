import Foundation

enum SomeEnum: Codable {
    /// The associated String contains some opaque context
    case context(String)
    case other(Int32)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case context = "Context"
        case other = "Other"
    }

    var type: `Type` {
        switch self {
        case .context: return .context
        case .other: return .other
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
        case .context:
            let content = try container.decode(String.self, forKey: .content)
            self = .context(content)
        case .other:
            let content = try container.decode(Int32.self, forKey: .content)
            self = .other(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .context(let content):
            try container.encode(`Type`.context, forKey: .type)
            try container.encode(content, forKey: .content)
        case .other(let content):
            try container.encode(`Type`.other, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}
