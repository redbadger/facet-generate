import Foundation

struct OtherType: Codable {
    init() {}
}

/// This is a comment.
struct PersonTwo: Codable {
    var name: String
    var age: UInt8
    var extraSpecialFieldOne: Int32
    var extraSpecialFieldTwo: [String]?
    var nonStandardDataType: OtherType
    var nonStandardDataTypeInArray: [OtherType]?

    init(name: String, age: UInt8, extraSpecialFieldOne: Int32, extraSpecialFieldTwo: [String]?, nonStandardDataType: OtherType, nonStandardDataTypeInArray: [OtherType]?) {
        self.name = name
        self.age = age
        self.extraSpecialFieldOne = extraSpecialFieldOne
        self.extraSpecialFieldTwo = extraSpecialFieldTwo
        self.nonStandardDataType = nonStandardDataType
        self.nonStandardDataTypeInArray = nonStandardDataTypeInArray
    }
}
