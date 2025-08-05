import Foundation

struct A: Codable {
    var field: UInt32

    init(field: UInt32) {
        self.field = field
    }
}

struct ABC: Codable {
    var field: UInt32

    init(field: UInt32) {
        self.field = field
    }
}

struct AB: Codable {
    var field: UInt32

    init(field: UInt32) {
        self.field = field
    }
}

struct OutsideOfModules: Codable {
    var field: UInt32

    init(field: UInt32) {
        self.field = field
    }
}
