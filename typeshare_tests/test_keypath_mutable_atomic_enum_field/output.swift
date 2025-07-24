import Foundation

struct NotDiffable: Codable {
    init() {}
}

enum InternallyTagged: Codable, KeyPathMutable {
    case anonymousStruct(atomic: NotDiffable)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case anonymousStruct
    }

    var type: `Type` {
        switch self {
        case .anonymousStruct: return .anonymousStruct
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case atomic
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .anonymousStruct:
            let atomic = try container.decode(NotDiffable.self, forKey: .atomic)
            self = .anonymousStruct(atomic: atomic)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .anonymousStruct(let atomic):
            try container.encode(`Type`.anonymousStruct, forKey: .type)
            try container.encode(atomic, forKey: .atomic)
        }
    }

    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {
        guard let variant = keyPath.first else {
            return apply(patch)
        }
        switch (self, variant) {
        case var (.anonymousStruct(atomic), .variant(key: "anonymousStruct", tag: _)):
            guard keyPath.count >= 2 else {
                return Logs.error("`InternallyTagged.anonymousStruct` expects a field after the variant in the key path")
            }
            let field = keyPath[keyPath.index(keyPath.startIndex, offsetBy:1)]
            switch field {
            case .field("atomic"):
                guard keyPath.count == 2 else {
                    return Logs.error("`InternallyTagged.anonymousStruct.atomic` expects an atomic update")
                }
                guard case let .update(value) = patch else {
                    return Logs.error("`InternallyTagged.anonymousStruct.atomic` is atomic and only support update patches")
                }

                guard let atomic = NotDiffable.fromAnyCodable(value) else {
                    return Logs.error("Trying to update `atomic` with \(value.value)")
                }
                
                self = .anonymousStruct(atomic: atomic)
            default:
                Logs.error("`InternallyTagged.anonymousStruct` has no patchable value at path \"\(field)\"")
            }
        default:
            Logs.error("Trying to apply a patch for the wrong variant: expect `\(self)`, received `\(variant)`")
        }
    }

    private mutating func apply(_ patch: PatchOperation) {
        switch patch {
        case let .update(value):
            if let newValue = value.value as? InternallyTagged {
                self = newValue
            } else if let newValue = InternallyTagged.fromAnyCodable(value) {
                self = newValue
            } else {
                Logs.error("Trying to update `InternallyTagged` with \(value.value)")
            }
        case .splice:
            Logs.error("`InternallyTagged` does not support splice operations.")
        }
    }
}
