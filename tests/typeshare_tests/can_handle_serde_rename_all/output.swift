import Foundation

/// This is a Person struct with camelCase rename
struct Person: Codable {
    var firstName: String
    var lastName: String
    var age: UInt8
    var extraSpecialField1: Int32
    var extraSpecialField2: [String]?

    init(firstName: String, lastName: String, age: UInt8, extraSpecialField1: Int32, extraSpecialField2: [String]?) {
        self.firstName = firstName
        self.lastName = lastName
        self.age = age
        self.extraSpecialField1 = extraSpecialField1
        self.extraSpecialField2 = extraSpecialField2
    }
}

/// This is a Person2 struct with UPPERCASE rename
struct Person2: Codable {
    var FIRST_NAME: String
    var LAST_NAME: String
    var AGE: UInt8

    init(FIRST_NAME: String, LAST_NAME: String, AGE: UInt8) {
        self.FIRST_NAME = FIRST_NAME
        self.LAST_NAME = LAST_NAME
        self.AGE = AGE
    }
}
