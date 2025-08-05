import Foundation

typealias SimpleAlias1 = String

typealias SimpleAlias2 = String

struct BrandedStringAlias: Branded, Codable {
    var value: String

    init(_ value: String) {
        self.value = value
    }

    func encode(to encoder: Encoder) throws {
        try value.encode(to: encoder)
    }

    init(from decoder: Decoder) throws {
        self.value = try String(from: decoder)
    }
}

struct BrandedOptionalStringAlias: Branded, Codable {
    var value: String?

    init(_ value: String?) {
        self.value = value
    }

    func encode(to encoder: Encoder) throws {
        try value.encode(to: encoder)
    }

    init(from decoder: Decoder) throws {
        self.value = try String?(from: decoder)
    }
}

struct BrandedU32Alias: Branded, Codable, Equatable, Hashable {
    var value: UInt32

    init(_ value: UInt32) {
        self.value = value
    }

    func encode(to encoder: Encoder) throws {
        try value.encode(to: encoder)
    }

    init(from decoder: Decoder) throws {
        self.value = try UInt32(from: decoder)
    }
}

struct MyStruct: Codable {
    var field: UInt32
    var other_field: String

    init(field: UInt32, other_field: String) {
        self.field = field
        self.other_field = other_field
    }
}

struct BrandedStructAlias: Branded, Codable {
    var value: MyStruct

    init(_ value: MyStruct) {
        self.value = value
    }

    func encode(to encoder: Encoder) throws {
        try value.encode(to: encoder)
    }

    init(from decoder: Decoder) throws {
        self.value = try MyStruct(from: decoder)
    }
}
