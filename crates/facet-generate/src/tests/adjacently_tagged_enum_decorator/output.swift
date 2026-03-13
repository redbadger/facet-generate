import Foundation

enum BestHockeyTeams: Codable {
    case pittsburghPenguins
    case lies(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case pittsburghPenguins = "PittsburghPenguins"
        case lies = "Lies"
    }

    var type: `Type` {
        switch self {
        case .pittsburghPenguins: return .pittsburghPenguins
        case .lies: return .lies
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
        case .pittsburghPenguins:
            self = .pittsburghPenguins
        case .lies:
            let content = try container.decode(String.self, forKey: .content)
            self = .lies(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .pittsburghPenguins:
            try container.encode(`Type`.pittsburghPenguins, forKey: .type)
        case .lies(let content):
            try container.encode(`Type`.lies, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

enum BestHockeyTeams1: Codable, Equatable {
    case pittsburghPenguins
    case lies(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case pittsburghPenguins = "PittsburghPenguins"
        case lies = "Lies"
    }

    var type: `Type` {
        switch self {
        case .pittsburghPenguins: return .pittsburghPenguins
        case .lies: return .lies
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
        case .pittsburghPenguins:
            self = .pittsburghPenguins
        case .lies:
            let content = try container.decode(String.self, forKey: .content)
            self = .lies(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .pittsburghPenguins:
            try container.encode(`Type`.pittsburghPenguins, forKey: .type)
        case .lies(let content):
            try container.encode(`Type`.lies, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

enum BestHockeyTeams2: Codable, Comparable, Equatable, Hashable {
    case pittsburghPenguins
    case lies(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case pittsburghPenguins = "PittsburghPenguins"
        case lies = "Lies"
    }

    var type: `Type` {
        switch self {
        case .pittsburghPenguins: return .pittsburghPenguins
        case .lies: return .lies
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
        case .pittsburghPenguins:
            self = .pittsburghPenguins
        case .lies:
            let content = try container.decode(String.self, forKey: .content)
            self = .lies(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .pittsburghPenguins:
            try container.encode(`Type`.pittsburghPenguins, forKey: .type)
        case .lies(let content):
            try container.encode(`Type`.lies, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

enum BestHockeyTeams3: Codable {
    case pittsburghPenguins
    case lies(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case pittsburghPenguins = "PittsburghPenguins"
        case lies = "Lies"
    }

    var type: `Type` {
        switch self {
        case .pittsburghPenguins: return .pittsburghPenguins
        case .lies: return .lies
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
        case .pittsburghPenguins:
            self = .pittsburghPenguins
        case .lies:
            let content = try container.decode(String.self, forKey: .content)
            self = .lies(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .pittsburghPenguins:
            try container.encode(`Type`.pittsburghPenguins, forKey: .type)
        case .lies(let content):
            try container.encode(`Type`.lies, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

enum BestHockeyTeams4: Codable, Equatable, Hashable {
    case pittsburghPenguins
    case lies(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case pittsburghPenguins = "PittsburghPenguins"
        case lies = "Lies"
    }

    var type: `Type` {
        switch self {
        case .pittsburghPenguins: return .pittsburghPenguins
        case .lies: return .lies
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
        case .pittsburghPenguins:
            self = .pittsburghPenguins
        case .lies:
            let content = try container.decode(String.self, forKey: .content)
            self = .lies(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .pittsburghPenguins:
            try container.encode(`Type`.pittsburghPenguins, forKey: .type)
        case .lies(let content):
            try container.encode(`Type`.lies, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}
