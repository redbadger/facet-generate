import Foundation

struct QualifiedTypes: Codable {
    var unqualified: String
    var qualified: String
    var qualified_vec: [String]
    var qualified_hashmap: [String: String]
    var qualified_optional: String?
    var qualfied_optional_hashmap_vec: [String: [String]]?

    init(unqualified: String, qualified: String, qualified_vec: [String], qualified_hashmap: [String: String], qualified_optional: String?, qualfied_optional_hashmap_vec: [String: [String]]?) {
        self.unqualified = unqualified
        self.qualified = qualified
        self.qualified_vec = qualified_vec
        self.qualified_hashmap = qualified_hashmap
        self.qualified_optional = qualified_optional
        self.qualfied_optional_hashmap_vec = qualfied_optional_hashmap_vec
    }
}
