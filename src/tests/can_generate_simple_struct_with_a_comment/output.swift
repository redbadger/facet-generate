import Foundation

struct Location: Codable {
    init() {}
}

/// This is a comment.
struct Person: Codable {
    /// This is another comment
    var name: String
    var age: UInt8
    var info: String?
    var emails: [String]
    var location: Location

    init(name: String, age: UInt8, info: String?, emails: [String], location: Location) {
        self.name = name
        self.age = age
        self.info = info
        self.emails = emails
        self.location = location
    }
}
