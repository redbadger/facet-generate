import Foundation

typealias Bar = String

struct Foo: Codable {
    var bar: Bar

    init(bar: Bar) {
        self.bar = bar
    }
}
