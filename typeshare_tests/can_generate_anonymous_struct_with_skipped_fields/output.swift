import Foundation

enum SomeEnum: Codable {
    case anonymousStruct(all: Bool, except_kotlin: Bool, except_ts: Bool)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case anonymousStruct = "AnonymousStruct"
    }

    var type: `Type` {
        switch self {
        case .anonymousStruct: return .anonymousStruct
        }
    }

    private enum CodingKeys: String, CodingKey {
        case all
        case except_kotlin
        case except_ts
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: `Type`.self)
        
        if container.contains(.anonymousStruct) {
            let container = try container.nestedContainer(keyedBy: CodingKeys.self, forKey: .anonymousStruct)
            let all = try container.decode(Bool.self, forKey: .all)
            let except_kotlin = try container.decode(Bool.self, forKey: .except_kotlin)
            let except_ts = try container.decode(Bool.self, forKey: .except_ts)
            self = .anonymousStruct(all: all, except_kotlin: except_kotlin, except_ts: except_ts)
            return
        }
        throw DecodingError.typeMismatch(SomeEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong tag for SomeEnum"))
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: `Type`.self)
        switch self {
        case .anonymousStruct(let all, let except_kotlin, let except_ts):
            var container = container.nestedContainer(keyedBy: CodingKeys.self, forKey: .anonymousStruct)
            try container.encode(all, forKey: .all)
            try container.encode(except_kotlin, forKey: .except_kotlin)
            try container.encode(except_ts, forKey: .except_ts)
        }
    }
}
