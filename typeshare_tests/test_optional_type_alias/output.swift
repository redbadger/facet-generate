import Foundation

typealias OptionalU32 = UInt32?

typealias OptionalU16 = UInt16?

struct FooBar: Codable {
    var foo: OptionalU32
    var bar: OptionalU16

    init(foo: OptionalU32, bar: OptionalU16) {
        self.foo = foo
        self.bar = bar
    }
}
