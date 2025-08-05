import Foundation

struct SomeStruct: Codable {
    let field_a: UInt32
    let field_b: [String]

    init(field_a: UInt32, field_b: [String]) {
        self.field_a = field_a
        self.field_b = field_b
    }
}
