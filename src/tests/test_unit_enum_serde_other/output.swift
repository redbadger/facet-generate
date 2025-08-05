import Foundation

/// This is a comment.
enum Source: String, Codable {
    case embedded = "Embedded"
    case googleFont = "GoogleFont"
    case custom = "Custom"
    case unknown = "Unknown"

    init(from decoder: Decoder) throws {
        self = try Source(rawValue: decoder.singleValueContainer().decode(RawValue.self)) ?? .unknown
    }
}
