import Foundation

struct Foo: Codable {
    var bar: Bool

    init(bar: Bool) {
        self.bar = bar
    }
}
