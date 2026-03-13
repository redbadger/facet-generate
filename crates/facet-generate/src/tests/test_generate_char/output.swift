import Foundation

struct MyType: Codable {
    var field: Unicode.Scalar

    init(field: Unicode.Scalar) {
        self.field = field
    }
}
