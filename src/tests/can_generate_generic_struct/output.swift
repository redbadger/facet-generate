import Foundation

struct GenericStruct<A: Codable, B: Codable>: Codable {
    var field_a: A
    var field_b: [B]

    init(field_a: A, field_b: [B]) {
        self.field_a = field_a
        self.field_b = field_b
    }
}

struct GenericStructUsingGenericStruct<T: Codable>: Codable {
    var struct_field: GenericStruct<String, T>
    var second_struct_field: GenericStruct<T, String>
    var third_struct_field: GenericStruct<T, [T]>

    init(struct_field: GenericStruct<String, T>, second_struct_field: GenericStruct<T, String>, third_struct_field: GenericStruct<T, [T]>) {
        self.struct_field = struct_field
        self.second_struct_field = second_struct_field
        self.third_struct_field = third_struct_field
    }
}

enum EnumUsingGenericStruct: Codable {
    case variantA(GenericStruct<String, Float>)
    case variantB(GenericStruct<String, Int32>)
    case variantC(GenericStruct<String, Bool>)
    case variantD(GenericStructUsingGenericStruct<CodableVoid>)

    enum `Type`: String, CodingKey, Codable, CaseIterable {
        case variantA = "VariantA"
        case variantB = "VariantB"
        case variantC = "VariantC"
        case variantD = "VariantD"
    }

    var type: `Type` {
        switch self {
        case .variantA: return .variantA
        case .variantB: return .variantB
        case .variantC: return .variantC
        case .variantD: return .variantD
        }
    }

    private enum CodingKeys: String, CodingKey {
        case type
        case content
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(`Type`.self, forKey: .type)
        switch type {
        case .variantA:
            let content = try container.decode(GenericStruct<String, Float>.self, forKey: .content)
            self = .variantA(content)
        case .variantB:
            let content = try container.decode(GenericStruct<String, Int32>.self, forKey: .content)
            self = .variantB(content)
        case .variantC:
            let content = try container.decode(GenericStruct<String, Bool>.self, forKey: .content)
            self = .variantC(content)
        case .variantD:
            let content = try container.decode(GenericStructUsingGenericStruct<CodableVoid>.self, forKey: .content)
            self = .variantD(content)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .variantA(let content):
            try container.encode(`Type`.variantA, forKey: .type)
            try container.encode(content, forKey: .content)
        case .variantB(let content):
            try container.encode(`Type`.variantB, forKey: .type)
            try container.encode(content, forKey: .content)
        case .variantC(let content):
            try container.encode(`Type`.variantC, forKey: .type)
            try container.encode(content, forKey: .content)
        case .variantD(let content):
            try container.encode(`Type`.variantD, forKey: .type)
            try container.encode(content, forKey: .content)
        }
    }
}

/// () isn't codable, so we use this instead to represent Rust's unit type
struct CodableVoid: Codable, KeyPathMutable {
    public mutating func patch<C: Collection>(_ patch: PatchOperation, at keyPath: C) where C.Element == KeyPathElement {}
}
