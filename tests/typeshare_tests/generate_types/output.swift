import Foundation

struct CustomType: Codable {
    init() {}
}

struct Types: Codable {
    var s: String
    var static_s: String
    var int8: Int8
    var float: Float
    var double: Double
    var array: [String]
    var fixed_length_array: [String]
    var dictionary: [String: Int32]
    var optional_dictionary: [String: Int32]?
    var custom_type: CustomType

    init(s: String, static_s: String, int8: Int8, float: Float, double: Double, array: [String], fixed_length_array: [String], dictionary: [String: Int32], optional_dictionary: [String: Int32]?, custom_type: CustomType) {
        self.s = s
        self.static_s = static_s
        self.int8 = int8
        self.float = float
        self.double = double
        self.array = array
        self.fixed_length_array = fixed_length_array
        self.dictionary = dictionary
        self.optional_dictionary = optional_dictionary
        self.custom_type = custom_type
    }
}
