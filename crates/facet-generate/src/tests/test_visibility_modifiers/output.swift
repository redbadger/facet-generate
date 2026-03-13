import Foundation

public typealias Bar = String

public struct Foo: Codable {
    public var bar: Bar

    public init(bar: Bar) {
        self.bar = bar
    }
}

public enum Baz: String, Codable {
    case bar = "Bar"
    case foo = "Foo"
}
