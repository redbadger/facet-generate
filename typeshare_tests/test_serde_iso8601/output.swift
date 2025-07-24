import Foundation

struct Foo: Codable {
    var time: Date

    init(time: Date) {
        self.time = time
    }
}
