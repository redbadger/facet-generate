import Foundation

struct Bar: Codable, KeyPathMutable {
    var one: String

    init(one: String) {
        self.one = one
    }

    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {
        guard let first = keyPath.first else {
            return apply(patch)
        }
        switch first {
        case let .field(field):
            switch field {
            case "one":
                let nextIndex = keyPath.index(keyPath.startIndex, offsetBy: 1)
                self.one.patch(patch, at: keyPath[nextIndex...])
            default:
                Logs.error("Invalid field: `Bar` has no patchable field \"\(field)\".")
            }
        default:
            Logs.error("Invalid key path: `Bar` only supports field-based keypath.")
        }
    }

    private mutating func apply(_ patch: PatchOperation) {
        switch patch {
            case let .update(value):
                if let newValue = value.value as? Bar {
                    self = newValue
                } else if let newValue = Bar.fromAnyCodable(value) {
                    self = newValue
                } else {
                    Logs.error("Trying to update `Bar` with \(value.value)")
                }
            case .splice:
                Logs.error("`Bar` does not support splice operations.")
        }
    }
}

struct Foo: Codable, KeyPathMutable {
    var bar: Bar?

    init(bar: Bar?) {
        self.bar = bar
    }

    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {
        guard let first = keyPath.first else {
            return apply(patch)
        }
        switch first {
        case let .field(field):
            switch field {
            case "bar":
                let nextIndex = keyPath.index(keyPath.startIndex, offsetBy: 1)
                self.bar.patch(patch, at: keyPath[nextIndex...])
            default:
                Logs.error("Invalid field: `Foo` has no patchable field \"\(field)\".")
            }
        default:
            Logs.error("Invalid key path: `Foo` only supports field-based keypath.")
        }
    }

    private mutating func apply(_ patch: PatchOperation) {
        switch patch {
            case let .update(value):
                if let newValue = value.value as? Foo {
                    self = newValue
                } else if let newValue = Foo.fromAnyCodable(value) {
                    self = newValue
                } else {
                    Logs.error("Trying to update `Foo` with \(value.value)")
                }
            case .splice:
                Logs.error("`Foo` does not support splice operations.")
        }
    }
}
