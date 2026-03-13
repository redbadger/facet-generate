import Foundation

enum Test: Codable {
    case anonymousEmptyStruct
    case noStruct

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case anonymousEmptyStruct = "AnonymousEmptyStruct"
        case noStruct = "NoStruct"
    }

    var type: `Type` {
        switch self {
        case .anonymousEmptyStruct: return .anonymousEmptyStruct
        case .noStruct: return .noStruct
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .anonymousEmptyStruct:
            self = .anonymousEmptyStruct
        case .noStruct:
            self = .noStruct
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .anonymousEmptyStruct:
            try container.encode(`Type`.anonymousEmptyStruct, forKey: .type)
        case .noStruct:
            try container.encode(`Type`.noStruct, forKey: .type)
        }
    }
}
