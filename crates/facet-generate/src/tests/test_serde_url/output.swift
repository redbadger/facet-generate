import Foundation

struct Foo: Codable {
    var url: String

    init(url: String) {
        self.url = url
    }
}
