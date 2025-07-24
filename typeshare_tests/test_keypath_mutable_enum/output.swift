import Foundation

enum InternallyTagged: Codable, KeyPathMutable {
    case unit
    case anonymousStruct(foor: Int, bar: String)
    case emptyStruct
    case tuple(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case unit
        case anonymousStruct
        case emptyStruct
        case tuple
    }

    var type: `Type` {
        switch self {
        case .unit: return .unit
        case .anonymousStruct: return .anonymousStruct
        case .emptyStruct: return .emptyStruct
        case .tuple: return .tuple
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case foor
        case bar
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .unit:
            self = .unit
        case .anonymousStruct:
            let foor = try container.decode(Int.self, forKey: .foor)
            let bar = try container.decode(String.self, forKey: .bar)
            self = .anonymousStruct(foor: foor, bar: bar)
        case .emptyStruct:
            self = .emptyStruct
        case .tuple:
            let content = try String(from: decoder)
            self = .tuple(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .unit:
            try container.encode(`Type`.unit, forKey: .type)
        case .anonymousStruct(let foor, let bar):
            try container.encode(`Type`.anonymousStruct, forKey: .type)
            try container.encode(foor, forKey: .foor)
            try container.encode(bar, forKey: .bar)
        case .emptyStruct:
            try container.encode(`Type`.emptyStruct, forKey: .type)
        case .tuple(let content):
            try container.encode(`Type`.tuple, forKey: .type)
            try content.encode(to: encoder)
        }
    }

    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {
        guard let variant = keyPath.first else {
            return apply(patch)
        }
        switch (self, variant) {
        case (.unit, .variant(key: "unit", tag: _)):
            Logs.error("`InternallyTagged.unit` does not support nested key path patches")
        case var (.anonymousStruct(foor, bar), .variant(key: "anonymousStruct", tag: _)):
            guard keyPath.count >= 2 else {
                return Logs.error("`InternallyTagged.anonymousStruct` expects a field after the variant in the key path")
            }
            let field = keyPath[keyPath.index(keyPath.startIndex, offsetBy:1)]
            switch field {
            case .field("foor"):
                let childIndex = keyPath.index(keyPath.startIndex, offsetBy: 2)
                foor.patch(patch, at: keyPath[childIndex...])
                self = .anonymousStruct(foor: foor, bar: bar)
            case .field("bar"):
                let childIndex = keyPath.index(keyPath.startIndex, offsetBy: 2)
                bar.patch(patch, at: keyPath[childIndex...])
                self = .anonymousStruct(foor: foor, bar: bar)
            default:
                Logs.error("`InternallyTagged.anonymousStruct` has no patchable value at path \"\(field)\"")
            }
        case (.emptyStruct, .variant(key: "emptyStruct", tag: _)):
            Logs.error("`InternallyTagged.emptyStruct` does not support nested key paths")
        case var (.tuple(value0), .variant(key: "tuple", tag: _)):
            guard keyPath.count >= 2 else {
               return Logs.error("`InternallyTagged.tuple` expects a field after the variant in the key path")
            }
            let component = keyPath[keyPath.index(keyPath.startIndex, offsetBy:1)]
            switch component {
            case .field(key: "0"):
                let childIndex = keyPath.index(keyPath.startIndex, offsetBy: 2)
                value0.patch(patch, at: keyPath[childIndex...])
                self = .tuple(value0)
            default:
                return Logs.error("`InternallyTagged.tuple` unexpected key path component: \(component)")
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

enum ExternallyTagged: Codable, KeyPathMutable {
    case anonymousStruct(foor: Int, bar: String)
    case tuple(String)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case anonymousStruct
        case tuple
    }

    var type: `Type` {
        switch self {
        case .anonymousStruct: return .anonymousStruct
        case .tuple: return .tuple
        }
    }

    private enum CodingKeys: String, CodingKey {
        case foor
        case bar
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: `Type`.self)
        
        if container.contains(.anonymousStruct) {
            let container = try container.nestedContainer(keyedBy: CodingKeys.self, forKey: .anonymousStruct)
            let foor = try container.decode(Int.self, forKey: .foor)
            let bar = try container.decode(String.self, forKey: .bar)
            self = .anonymousStruct(foor: foor, bar: bar)
            return
        }
        if container.contains(.tuple) {
            let content = try container.decode(String.self, forKey: .tuple)
            self = .tuple(content)
            return
        }
        throw DecodingError.typeMismatch(ExternallyTagged.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong tag for ExternallyTagged"))
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: `Type`.self)
        switch self {
        case .anonymousStruct(let foor, let bar):
            var container = container.nestedContainer(keyedBy: CodingKeys.self, forKey: .anonymousStruct)
            try container.encode(foor, forKey: .foor)
            try container.encode(bar, forKey: .bar)
        case .tuple(let content):
            try container.encode(content, forKey: .tuple)
        }
    }

    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {
        guard let variant = keyPath.first else {
            return apply(patch)
        }
        switch (self, variant) {
        case var (.anonymousStruct(foor, bar), .variant(key: "anonymousStruct", tag: _)):
            guard keyPath.count >= 2 else {
                return Logs.error("`ExternallyTagged.anonymousStruct` expects a field after the variant in the key path")
            }
            let field = keyPath[keyPath.index(keyPath.startIndex, offsetBy:1)]
            switch field {
            case .field("foor"):
                let childIndex = keyPath.index(keyPath.startIndex, offsetBy: 2)
                foor.patch(patch, at: keyPath[childIndex...])
                self = .anonymousStruct(foor: foor, bar: bar)
            case .field("bar"):
                let childIndex = keyPath.index(keyPath.startIndex, offsetBy: 2)
                bar.patch(patch, at: keyPath[childIndex...])
                self = .anonymousStruct(foor: foor, bar: bar)
            default:
                Logs.error("`ExternallyTagged.anonymousStruct` has no patchable value at path \"\(field)\"")
            }
        case var (.tuple(value0), .variant(key: "tuple", tag: _)):
            guard keyPath.count >= 2 else {
               return Logs.error("`ExternallyTagged.tuple` expects a field after the variant in the key path")
            }
            let component = keyPath[keyPath.index(keyPath.startIndex, offsetBy:1)]
            switch component {
            case .field(key: "0"):
                let childIndex = keyPath.index(keyPath.startIndex, offsetBy: 2)
                value0.patch(patch, at: keyPath[childIndex...])
                self = .tuple(value0)
            default:
                return Logs.error("`ExternallyTagged.tuple` unexpected key path component: \(component)")
            }
        default:
            Logs.error("Trying to apply a patch for the wrong variant: expect `\(self)`, received `\(variant)`")
        }
    }

    private mutating func apply(_ patch: PatchOperation) {
        switch patch {
        case let .update(value):
            if let newValue = value.value as? ExternallyTagged {
                self = newValue
            } else if let newValue = ExternallyTagged.fromAnyCodable(value) {
                self = newValue
            } else {
                Logs.error("Trying to update `ExternallyTagged` with \(value.value)")
            }
        case .splice:
            Logs.error("`ExternallyTagged` does not support splice operations.")
        }
    }
}

enum Unit: String, Codable, KeyPathMutable {
    case foo
    case bar

    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {
        guard keyPath.isEmpty else { return Logs.error("`Unit` does not support child keyPath") }
        apply(patch)
    }

    private mutating func apply(_ patch: PatchOperation) {
        switch patch {
        case let .update(value):
            if let newValue = value.value as? Unit {
                self = newValue
            } else if let newValue = Unit.fromAnyCodable(value) {
                self = newValue
            } else {
                Logs.error("Trying to update `Unit` with \(value.value)")
            }
        case .splice:
            Logs.error("`Unit` does not support splice operations.")
        }
    }
}

enum Generic<T: Codable>: Codable, KeyPathMutable {
    case anonymousStruct(foo: T)
    case tuple(T)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case anonymousStruct
        case tuple
    }

    var type: `Type` {
        switch self {
        case .anonymousStruct: return .anonymousStruct
        case .tuple: return .tuple
        }
    }

    private enum CodingKeys: String, CodingKey {
        case foo
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: `Type`.self)
        
        if container.contains(.anonymousStruct) {
            let container = try container.nestedContainer(keyedBy: CodingKeys.self, forKey: .anonymousStruct)
            let foo = try container.decode(T.self, forKey: .foo)
            self = .anonymousStruct(foo: foo)
            return
        }
        if container.contains(.tuple) {
            let content = try container.decode(T.self, forKey: .tuple)
            self = .tuple(content)
            return
        }
        throw DecodingError.typeMismatch(Generic.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong tag for Generic"))
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: `Type`.self)
        switch self {
        case .anonymousStruct(let foo):
            var container = container.nestedContainer(keyedBy: CodingKeys.self, forKey: .anonymousStruct)
            try container.encode(foo, forKey: .foo)
        case .tuple(let content):
            try container.encode(content, forKey: .tuple)
        }
    }

    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {
        guard let variant = keyPath.first else {
            return apply(patch)
        }
        switch (self, variant) {
        case var (.anonymousStruct(foo), .variant(key: "anonymousStruct", tag: _)):
            guard keyPath.count >= 2 else {
                return Logs.error("`Generic.anonymousStruct` expects a field after the variant in the key path")
            }
            let field = keyPath[keyPath.index(keyPath.startIndex, offsetBy:1)]
            switch field {
            case .field("foo"):
                let childIndex = keyPath.index(keyPath.startIndex, offsetBy: 2)
                foo.patch(patch, at: keyPath[childIndex...])
                self = .anonymousStruct(foo: foo)
            default:
                Logs.error("`Generic.anonymousStruct` has no patchable value at path \"\(field)\"")
            }
        case var (.tuple(value0), .variant(key: "tuple", tag: _)):
            guard keyPath.count >= 2 else {
               return Logs.error("`Generic.tuple` expects a field after the variant in the key path")
            }
            let component = keyPath[keyPath.index(keyPath.startIndex, offsetBy:1)]
            switch component {
            case .field(key: "0"):
                let childIndex = keyPath.index(keyPath.startIndex, offsetBy: 2)
                value0.patch(patch, at: keyPath[childIndex...])
                self = .tuple(value0)
            default:
                return Logs.error("`Generic.tuple` unexpected key path component: \(component)")
            }
        default:
            Logs.error("Trying to apply a patch for the wrong variant: expect `\(self)`, received `\(variant)`")
        }
    }

    private mutating func apply(_ patch: PatchOperation) {
        switch patch {
        case let .update(value):
            if let newValue = value.value as? Generic {
                self = newValue
            } else if let newValue = Generic.fromAnyCodable(value) {
                self = newValue
            } else {
                Logs.error("Trying to update `Generic` with \(value.value)")
            }
        case .splice:
            Logs.error("`Generic` does not support splice operations.")
        }
    }
}
