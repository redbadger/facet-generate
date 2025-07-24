import Foundation

indirect enum Options: Codable {
    case red(Bool)
    case banana(String)
    case vermont(Options)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case red
        case banana
        case vermont
    }

    var type: `Type` {
        switch self {
        case .red: return .red
        case .banana: return .banana
        case .vermont: return .vermont
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
        case .red:
            let content = try container.decode(Bool.self, forKey: .content)
            self = .red(content)
        case .banana:
            let content = try container.decode(String.self, forKey: .content)
            self = .banana(content)
        case .vermont:
            let content = try container.decode(Options.self, forKey: .content)
            self = .vermont(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .red(let content):
            try container.encode(`Type`.red, forKey: .type)
            try container.encode(content, forKey: .content)
        case .banana(let content):
            try container.encode(`Type`.banana, forKey: .type)
            try container.encode(content, forKey: .content)
        case .vermont(let content):
            try container.encode(`Type`.vermont, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

indirect enum MoreOptions: Codable {
    case news(Bool)
    case exactly(config: String)
    case built(top: MoreOptions)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case news
        case exactly
        case built
    }

    var type: `Type` {
        switch self {
        case .news: return .news
        case .exactly: return .exactly
        case .built: return .built
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    private enum NestedCodingKeys: String, CodingKey {
        case config
        case top
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .news:
            let content = try container.decode(Bool.self, forKey: .content)
            self = .news(content)
        case .exactly:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let config = try container.decode(String.self, forKey: .config)
            self = .exactly(config: config)
        case .built:
            let container = try container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            let top = try container.decode(MoreOptions.self, forKey: .top)
            self = .built(top: top)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .news(let content):
            try container.encode(`Type`.news, forKey: .type)
            try container.encode(content, forKey: .content)
        case .exactly(let config):
            try container.encode(`Type`.exactly, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(config, forKey: .config)
        case .built(let top):
            try container.encode(`Type`.built, forKey: .type)
            var container = container.nestedContainer(keyedBy: NestedCodingKeys.self, forKey: .content)
            try container.encode(top, forKey: .top)
        }
    }
}
