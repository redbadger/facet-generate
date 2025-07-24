import Foundation

/// This is a comment.
struct ArcyColors: Codable {
    var red: UInt8
    var blue: String
    var green: [String]

    init(red: UInt8, blue: String, green: [String]) {
        self.red = red
        self.blue = blue
        self.green = green
    }
}

/// This is a comment.
struct MutexyColors: Codable {
    var blue: [String]
    var green: String

    init(blue: [String], green: String) {
        self.blue = blue
        self.green = green
    }
}

/// This is a comment.
struct RcyColors: Codable {
    var red: String
    var blue: [String]
    var green: String

    init(red: String, blue: [String], green: String) {
        self.red = red
        self.blue = blue
        self.green = green
    }
}

/// This is a comment.
struct CellyColors: Codable {
    var red: String
    var blue: [String]

    init(red: String, blue: [String]) {
        self.red = red
        self.blue = blue
    }
}

/// This is a comment.
struct LockyColors: Codable {
    var red: String

    init(red: String) {
        self.red = red
    }
}

/// This is a comment.
struct CowyColors: Codable {
    var lifetime: String

    init(lifetime: String) {
        self.lifetime = lifetime
    }
}

/// This is a comment.
enum BoxyColors: Codable {
    case red
    case blue
    case green(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case red = "Red"
        case blue = "Blue"
        case green = "Green"
    }

    var type: `Type` {
        switch self {
        case .red: return .red
        case .blue: return .blue
        case .green: return .green
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
            self = .red
        case .blue:
            self = .blue
        case .green:
            let content = try container.decode(String.self, forKey: .content)
            self = .green(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .red:
            try container.encode(`Type`.red, forKey: .type)
        case .blue:
            try container.encode(`Type`.blue, forKey: .type)
        case .green(let content):
            try container.encode(`Type`.green, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}
