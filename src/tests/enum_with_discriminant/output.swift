import Foundation

enum Effect: Codable {
    case colorTemperature(ColorTemperatureAttributes)
    case contrast(ContrastAttributes)
    case exposure(ExposureAttributes)

    enum Name: String, CodingKey, Codable, CaseIterable {
        case colorTemperature = "temperature"
        case contrast
        case exposure
    }

    var name: Name {
        switch self {
        case .colorTemperature: return .colorTemperature
        case .contrast: return .contrast
        case .exposure: return .exposure
        }
    }

    private enum CodingKeys: String, CodingKey {
        case name
        case attributes
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let name = try container.decode(Name.self, forKey: .name)
        switch name {
        case .colorTemperature:
            let content = try container.decode(ColorTemperatureAttributes.self, forKey: .attributes)
            self = .colorTemperature(content)
        case .contrast:
            let content = try container.decode(ContrastAttributes.self, forKey: .attributes)
            self = .contrast(content)
        case .exposure:
            let content = try container.decode(ExposureAttributes.self, forKey: .attributes)
            self = .exposure(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .colorTemperature(let content):
            try container.encode(Name.colorTemperature, forKey: .name)
            try container.encode(content, forKey: .attributes)
        case .contrast(let content):
            try container.encode(Name.contrast, forKey: .name)
            try container.encode(content, forKey: .attributes)
        case .exposure(let content):
            try container.encode(Name.exposure, forKey: .name)
            try container.encode(content, forKey: .attributes)
        }
    }
}

typealias EffectName = Effect.Name;
